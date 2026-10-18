"""Compose the original RANA-56 score and effects using only NumPy + Python.

Run: python tools/compose_audio.py
No samples, recordings, downloaded instruments, or external encoders are used.
The score uses a shared F#-A-E-D motif, arranged independently for each location.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import wave

import numpy as np


RATE = 32000
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "audio"
RNG = np.random.default_rng(24856)
TAU = 2.0 * math.pi
_CACHE: dict[tuple, np.ndarray] = {}


def hz(note: float) -> float:
    return 440.0 * 2.0 ** ((note - 69.0) / 12.0)


def envelope(n: int, attack: float, release: float) -> np.ndarray:
    env = np.ones(n, np.float64)
    a = min(n, max(1, int(attack * RATE)))
    r = min(n, max(1, int(release * RATE)))
    env[:a] *= np.sin(np.linspace(0.0, math.pi / 2.0, a)) ** 2
    env[-r:] *= np.cos(np.linspace(0.0, math.pi / 2.0, r)) ** 2
    return env


def tone(kind: str, note: float, duration: float) -> np.ndarray:
    key = (kind, note, round(duration, 6))
    if key in _CACHE:
        return _CACHE[key]
    n = max(1, round(duration * RATE))
    t = np.arange(n, dtype=np.float64) / RATE
    f = hz(note)
    phase = TAU * f * t
    if kind == "strings":
        # A small ensemble with gentle detuning and bowed attack, not a saw beep.
        vibrato = 0.008 * np.sin(TAU * 4.6 * t) * (1.0 - np.exp(-3.0 * t))
        sound = np.zeros(n)
        for detune, gain in ((0.9976, 0.31), (1.0, 0.40), (1.0025, 0.29)):
            p = phase * detune + vibrato
            for harmonic in range(1, 7):
                sound += gain * (0.49 ** (harmonic - 1)) * np.sin(p * harmonic) / harmonic
        sound *= envelope(n, min(0.48, duration * 0.22), min(0.9, duration * 0.38))
    elif kind == "pad":
        sound = (
            np.sin(phase + 0.05 * np.sin(TAU * 0.17 * t))
            + 0.24 * np.sin(phase * 2.0006 + 0.4)
            + 0.09 * np.sin(phase * 3.0)
        ) * (0.93 + 0.07 * np.sin(TAU * 0.32 * t))
        sound *= envelope(n, min(0.6, duration * 0.25), min(1.1, duration * 0.43))
    elif kind in ("celesta", "harp"):
        if kind == "celesta":
            partials = ((1.0, 1.0, 1.8), (2.0, 0.15, 0.65), (3.0, 0.12, 0.42), (4.02, 0.035, 0.25))
        else:
            partials = ((1.0, 1.0, 1.2), (2.0, 0.22, 0.7), (3.0, 0.1, 0.4), (4.0, 0.035, 0.22))
        sound = sum(gain * np.sin(phase * harmonic) * np.exp(-t / decay)
                    for harmonic, gain, decay in partials)
        sound *= envelope(n, 0.006 if kind == "celesta" else 0.012, min(0.3, duration * 0.3))
    elif kind == "flute":
        breath = 0.035 * filtered_noise(n, 2200, 0.0)
        vib = 0.055 * np.sin(TAU * 5.1 * t) * (1.0 - np.exp(-4.5 * t))
        sound = np.sin(phase + vib) + 0.12 * np.sin(2.0 * phase + vib) + breath
        sound *= envelope(n, min(0.09, duration * 0.2), min(0.22, duration * 0.3))
        sound *= 0.94 + 0.06 * np.sin(TAU * 3.3 * t)
    elif kind == "bass":
        sound = np.sin(phase) + 0.18 * np.sin(2.0 * phase) + 0.045 * np.sin(3.0 * phase)
        sound *= envelope(n, 0.04, min(0.28, duration * 0.4))
    elif kind == "horn":
        sound = np.sin(phase) + 0.34 * np.sin(phase * 2.0) + 0.12 * np.sin(phase * 3.0)
        sound *= envelope(n, 0.07, min(0.3, duration * 0.3))
    else:
        raise ValueError(kind)
    result = sound.astype(np.float32)
    _CACHE[key] = result
    return result


def filtered_noise(n: int, cutoff: float = 1800.0, highpass: float = 0.0) -> np.ndarray:
    spectrum = np.fft.rfft(RNG.standard_normal(n))
    freqs = np.fft.rfftfreq(n, 1.0 / RATE)
    shape = np.exp(-(freqs / cutoff) ** 4)
    if highpass:
        shape *= 1.0 - np.exp(-(freqs / highpass) ** 2)
    noise = np.fft.irfft(spectrum * shape, n)
    return noise / max(0.01, float(np.std(noise)))


def add(buffer: np.ndarray, sound: np.ndarray, start: float, gain: float = 1.0,
        pan: float = 0.0, circular: bool = False) -> None:
    index = int(round(start * RATE))
    weights = np.array([math.cos((pan + 1.0) * math.pi / 4.0),
                        math.sin((pan + 1.0) * math.pi / 4.0)], dtype=np.float32)
    stereo = sound[:, None] * weights[None, :] * gain if sound.ndim == 1 else sound * gain
    if circular:
        index %= len(buffer)
        while len(stereo):
            count = min(len(stereo), len(buffer) - index)
            buffer[index:index + count] += stereo[:count]
            stereo = stereo[count:]
            index = 0
    else:
        count = min(len(stereo), len(buffer) - index)
        if index >= 0 and count > 0:
            buffer[index:index + count] += stereo[:count]


def room(buffer: np.ndarray, wet: float = 0.16, loop: bool = False) -> np.ndarray:
    # Diffuse stereo early reflections and a gently darkened tail.
    result = buffer.copy()
    for i, seconds in enumerate((0.073, 0.109, 0.173, 0.239, 0.347, 0.463, 0.631, 0.829, 1.071, 1.391)):
        delay = int(seconds * RATE)
        gain = wet * math.exp(-seconds * 2.5) * (0.80 if i % 2 else 1.0)
        source = buffer[:, ::-1] if i % 2 else buffer
        if loop:
            result += np.roll(source, delay, axis=0) * gain
        elif delay < len(buffer):
            result[delay:] += source[:-delay] * gain
    # Very short smoothing takes the edge off transients without dulling melody.
    result = result * 0.80 + np.roll(result, 1, axis=0) * 0.15 + np.roll(result, 2, axis=0) * 0.05
    return result


def percussion(kind: str) -> np.ndarray:
    duration = {"brush": 0.17, "shaker": 0.12, "low": 0.42, "bell": 0.9}[kind]
    n = int(duration * RATE)
    t = np.arange(n) / RATE
    if kind == "low":
        phase = TAU * (65.0 * t + 3.0 * (1.0 - np.exp(-t * 24.0)))
        sound = np.sin(phase) * np.exp(-t * 12.0) + filtered_noise(n, 400) * np.exp(-t * 35.0) * 0.05
    elif kind == "bell":
        sound = tone("celesta", 86, duration)
    else:
        sound = filtered_noise(n, 3500 if kind == "shaker" else 1800, 850)
        sound *= np.exp(-t * (34 if kind == "shaker" else 22))
    return sound.astype(np.float32) * envelope(n, 0.003, 0.025)


# All notes are authored below: no existing song, MIDI, or recorded melody.
# F# A E D is the crew's four-note call, heard softly in flight and fully at rescue.
SPACE_A = [
    [(0, 78, 1.6), (2, 81, 0.9), (3, 88, 1.4), (4.5, 86, 1.2)],
    [(0, 85, 1.5), (2, 81, 0.8), (3, 78, 2.2)],
    [(0, 79, 1.2), (1.5, 83, 1.0), (3, 86, 2.3)],
    [(0, 85, 1.4), (2, 83, 0.7), (3, 81, 2.1)],
    [(0, 78, 1.0), (1.5, 81, 1.0), (3, 88, 1.0), (4.5, 86, 1.0)],
    [(0, 83, 1.6), (2, 81, 0.8), (3, 79, 1.8)],
    [(0, 78, 1.0), (1.5, 79, 1.0), (3, 81, 1.1), (4.5, 86, 1.0)],
    [(0, 85, 2.2), (3, 81, 1.8)],
]
SPACE_B = [
    [(0, 83, 1.3), (1.5, 86, 1.0), (3, 90, 2.0)],
    [(0, 88, 1.7), (2.5, 86, 0.8), (4, 83, 1.3)],
    [(0, 81, 1.1), (1.5, 85, 1.0), (3, 88, 2.2)],
    [(0, 86, 2.0), (3, 81, 1.0), (4.5, 78, 0.8)],
    [(0, 79, 1.1), (1.5, 83, 1.2), (3, 86, 2.0)],
    [(0, 88, 1.2), (1.5, 86, 1.0), (3, 83, 1.0), (4.5, 81, 0.9)],
    [(0, 79, 1.0), (1.5, 78, 1.0), (3, 76, 2.0)],
    [(0, 73, 1.2), (1.5, 76, 1.1), (3, 81, 2.0)],
]

# Root + open chord voicing. Suspensions, added 9ths, and major 7ths leave air.
HARMONY_A = [
    (38, [50, 57, 61, 66]), (35, [47, 54, 57, 62]),
    (31, [43, 50, 54, 59]), (33, [45, 52, 59, 61]),
    (30, [42, 50, 57, 62]), (28, [40, 47, 54, 55]),
    (31, [43, 50, 57, 59]), (33, [45, 52, 57, 61]),
]
HARMONY_B = [
    (31, [43, 50, 54, 59]), (35, [47, 54, 57, 62]),
    (33, [45, 52, 57, 61]), (38, [50, 57, 61, 66]),
    (28, [40, 47, 54, 55]), (31, [43, 50, 54, 59]),
    (28, [40, 47, 52, 55]), (33, [45, 52, 59, 61]),
]


def compose(name: str) -> np.ndarray:
    tempo = {"space": 144, "lunar": 132, "mercury": 160, "rescue": 144}[name]
    beat = 60.0 / tempo
    bars = 12 if name == "rescue" else 24
    output = np.zeros((round(bars * 6 * beat * RATE), 2), np.float32)
    is_rescue = name == "rescue"
    for bar in range(bars):
        section = bar // 8
        index = bar % 8
        root, chord = (HARMONY_B if section == 1 and not is_rescue else HARMONY_A)[index]
        line = (SPACE_B if section == 1 and not is_rescue else SPACE_A)[index]
        start = bar * 6 * beat
        # Wider, quieter pads sit behind a close melodic instrument.
        for j, note in enumerate(chord):
            add(output, tone("pad", note, 7.4 * beat), start - 0.30,
                0.030 if name == "mercury" else 0.045, (j - 1.5) * 0.35, True)
            if name != "lunar" or section == 1:
                add(output, tone("strings", note + (12 if is_rescue else 0), 6.8 * beat),
                    start - 0.10, 0.040 if name == "space" else 0.035, (1.5 - j) * 0.25, True)
        add(output, tone("bass", root, 5.8 * beat), start, 0.12, 0, True)
        # A breathing 6/8 harp pattern, with a few intentional rests.
        harp_notes = [chord[0] + 12, chord[2] + 12, chord[1] + 12,
                      chord[3] + 12, chord[1] + 12, chord[2] + 12]
        for step, note in enumerate(harp_notes):
            if name == "lunar" and step in (2, 5):
                continue
            add(output, tone("harp", note, 2.6 * beat), start + step * beat,
                0.047 if step % 3 else 0.057, -0.55 + step * 0.2, True)
        # Space alternates flute and celesta; Luna favors lower, intimate notes.
        lead = "flute" if (name == "space" and section == 1) or is_rescue else "celesta"
        transpose = -12 if name in ("lunar", "mercury") else 0
        if name == "space" and section == 0:
            transpose = -12
        for at, note, length in line:
            sustain = length * beat
            if lead == "celesta":
                sustain += 0.65
            add(output, tone(lead, note + transpose, sustain), start + at * beat,
                0.095 if lead == "flute" else 0.10, -0.12 if lead == "flute" else 0.14, True)
            if is_rescue:
                add(output, tone("horn", note - 12, length * beat), start + at * beat,
                    0.055, -0.15, True)
        # A small answering flute phrase keeps the loop from sounding mechanical.
        if name in ("space", "lunar") and bar % 4 == 3:
            for at, note in ((4.0, chord[2] + 12), (5.0, chord[3] + 12)):
                add(output, tone("flute", note, 0.70 * beat), start + at * beat, 0.042, -0.30, True)
        if name in ("mercury", "rescue") or (name == "space" and section == 2):
            add(output, percussion("low"), start, 0.062 if not is_rescue else 0.09, 0, True)
            add(output, percussion("low"), start + 3 * beat, 0.038, 0.05, True)
            for step in range(6):
                add(output, percussion("shaker"), start + (step + 0.5) * beat,
                    0.012 if step % 3 else 0.020, 0.45, True)
            if name == "mercury":
                add(output, percussion("brush"), start + 3 * beat, 0.024, -0.3, True)
        elif name == "lunar" and bar % 2:
            add(output, percussion("brush"), start + 3 * beat, 0.012, 0.4, True)
    # A soft, spacious room; circular delays preserve the musical loop boundary.
    return room(output, 0.23 if name in ("space", "lunar") else 0.17, True)


def swoop(seconds: float, start_hz: float, end_hz: float, noise: float = 0.0) -> np.ndarray:
    n = round(seconds * RATE)
    t = np.arange(n) / RATE
    ratio = np.clip(t / seconds, 0, 1)
    freq = start_hz + (end_hz - start_hz) * (ratio * ratio * (3 - 2 * ratio))
    phase = TAU * np.cumsum(freq) / RATE
    sound = np.sin(phase) + 0.12 * np.sin(2.0 * phase)
    if noise:
        sound += noise * filtered_noise(n, 1700, 140)
    return sound * envelope(n, min(0.07, seconds * 0.18), seconds * 0.5)


def croak(seconds: float, pitch: float) -> np.ndarray:
    n = round(seconds * RATE)
    t = np.arange(n) / RATE
    f = pitch * (1.0 - 0.22 * t / seconds)
    phase = TAU * np.cumsum(f) / RATE
    # Rounded frog-like pulses, no speech or sampled voice.
    modulation = 0.60 + 0.40 * np.cos(TAU * 39.0 * t)
    sound = (np.sin(phase) + 0.30 * np.sin(phase * 2.0) + 0.14 * np.sin(phase * 3.0)) * modulation
    return sound * envelope(n, 0.018, min(0.1, seconds * 0.4))


def effect(name: str) -> tuple[np.ndarray, bool]:
    lengths = {"engine": 2.0, "turbo": 1.1, "brake": 0.75, "land": 1.4,
               "launch": 1.8, "step": 0.16, "jump": 0.52, "charge": 2.0,
               "foot_land": 0.40, "collector": 1.7, "unlock": 2.2,
               "high_five": 0.85, "rescue_success": 3.6, "radio_open": 0.50,
               "luma_talk": 0.16, "brasa_talk": 0.20, "ui_move": 0.20,
               "ui_confirm": 0.48, "heat_warning": 0.85}
    duration = lengths[name]
    n = round(duration * RATE)
    sound = np.zeros((n, 2), np.float32)
    loop = name in ("engine", "charge")
    t = np.arange(n) / RATE
    if name == "engine":
        # Exact integer cycles at the 2 s boundary, including the modulation.
        source = 0.50 * np.sin(TAU * 55 * t) + 0.16 * np.sin(TAU * 110 * t)
        source += 0.045 * np.sin(TAU * 164.5 * t) + 0.035 * filtered_noise(n, 320)
        source *= 0.93 + 0.07 * np.sin(TAU * 1.5 * t)
        add(sound, source, 0, 1.0, 0)
    elif name == "charge":
        source = (np.sin(TAU * 294 * t) + 0.24 * np.sin(TAU * 440 * t))
        source *= 0.74 + 0.26 * np.cos(TAU * 3 * t)
        add(sound, source, 0, 0.38)
    elif name in ("turbo", "brake", "launch"):
        a, b = {"turbo": (105, 260), "brake": (180, 55), "launch": (65, 230)}[name]
        add(sound, swoop(duration - 0.15, a, b, 0.45), 0, 0.35)
        if name != "brake":
            add(sound, tone("harp", 78, 0.6), duration * 0.5, 0.06, 0.15)
    elif name in ("step", "foot_land", "land"):
        source = filtered_noise(n, 850 if name == "step" else 560) * np.exp(-t * (45 if name == "step" else 17))
        source += 0.6 * np.sin(TAU * 72 * t) * np.exp(-t * 28)
        source *= envelope(n, 0.004, min(duration / 2, 0.2))
        add(sound, source, 0, 0.35)
        if name == "land":
            add(sound, swoop(0.65, 90, 44, 0.1), 0.08, 0.18)
            add(sound, tone("harp", 66, 0.65), 0.4, 0.12, -0.2)
            add(sound, tone("harp", 69, 0.65), 0.53, 0.09, 0.2)
    elif name == "jump":
        add(sound, swoop(0.34, 250, 560, 0.07), 0, 0.17)
        add(sound, tone("harp", 78, 0.4), 0.06, 0.12)
    elif name in ("collector", "unlock", "rescue_success"):
        notes = {"collector": [66, 69, 76, 74], "unlock": [62, 66, 69, 74, 78, 81],
                 "rescue_success": [66, 69, 76, 74, 78, 81, 86]}[name]
        spacing = 0.14 if name != "rescue_success" else 0.25
        for i, note in enumerate(notes):
            add(sound, tone("celesta", note + (12 if name == "collector" else 0),
                            min(1.2, duration - i * spacing)), i * spacing, 0.17,
                -0.3 + 0.6 * i / max(1, len(notes) - 1))
        if name == "rescue_success":
            for note in (50, 57, 62, 66):
                add(sound, tone("strings", note, 2.4), 0.8, 0.10, (note - 58) / 20)
    elif name == "high_five":
        # Three closely spaced soft noise bursts suggest palms rather than a gunshot.
        for offset, amp in ((0.0, 0.21), (0.013, 0.13), (0.027, 0.09)):
            count = int(0.13 * RATE)
            impact = filtered_noise(count, 2200, 350)
            impact *= np.exp(-np.arange(count) / RATE * 42) * envelope(count, 0.002, 0.03)
            add(sound, impact, offset, amp)
        add(sound, croak(0.32, 215), 0.18, 0.14, -0.15)
        add(sound, croak(0.28, 165), 0.28, 0.12, 0.20)
    elif name in ("luma_talk", "brasa_talk"):
        add(sound, croak(duration, 290 if name == "luma_talk" else 185), 0, 0.28)
    elif name == "radio_open":
        add(sound, tone("harp", 78, 0.25), 0, 0.17, -0.12)
        add(sound, tone("harp", 81, 0.28), 0.12, 0.15, 0.12)
    elif name == "ui_move":
        add(sound, tone("harp", 78, duration), 0, 0.19)
    elif name == "ui_confirm":
        add(sound, tone("celesta", 78, 0.40), 0, 0.17, -0.15)
        add(sound, tone("celesta", 81, 0.35), 0.10, 0.14, 0.15)
    elif name == "heat_warning":
        for start in (0.0, 0.30):
            add(sound, tone("flute", 74, 0.20), start, 0.19)
            add(sound, tone("flute", 69, 0.20), start + 0.06, 0.10)
    else:
        raise ValueError(name)
    if name not in ("engine", "charge", "step", "luma_talk", "brasa_talk"):
        sound = room(sound, 0.11, loop)
    if not loop:
        sound *= envelope(n, 0.002, min(0.06, duration / 4))[:, None]
    return sound, loop


SFX = ("engine", "turbo", "brake", "land", "launch", "step", "jump", "charge",
       "foot_land", "collector", "unlock", "high_five", "rescue_success", "radio_open",
       "luma_talk", "brasa_talk", "ui_move", "ui_confirm", "heat_warning")


def save(name: str, samples: np.ndarray, loop: bool, music: bool) -> dict:
    samples = samples.astype(np.float64)
    samples -= np.mean(samples, axis=0)
    peak = float(np.max(np.abs(samples)))
    # Music retains headroom for effects; effects are balanced by class in-game.
    desired_peak = 0.70 if music else (0.40 if name in ("engine", "charge") else 0.60)
    samples *= desired_peak / max(peak, 0.001)
    if loop:
        # Tiny circular equal-power overlap only for a residual non-periodic seam.
        seam = float(np.max(np.abs(samples[0] - samples[-1])))
        if seam > 0.025:
            width = int(0.008 * RATE)
            midpoint = (samples[0] + samples[-1]) * 0.5
            blend = np.linspace(0, 1, width)[:, None]
            samples[:width] = midpoint * (1 - blend) + samples[:width] * blend
            samples[-width:] = samples[-width:] * (1 - blend) + midpoint * blend
    else:
        # Removing DC can move an already faded endpoint away from digital zero.
        samples *= envelope(len(samples), 0.002, 0.012)[:, None]
    pcm = np.round(np.clip(samples, -1, 1) * 32767).astype("<i2")
    path = OUT / f"{name}.wav"
    with wave.open(str(path), "wb") as writer:
        writer.setnchannels(2)
        writer.setsampwidth(2)
        writer.setframerate(RATE)
        writer.writeframes(pcm.tobytes())
    with wave.open(str(path), "rb") as reader:
        decoded = np.frombuffer(reader.readframes(reader.getnframes()), dtype="<i2").reshape(-1, 2) / 32768.0
        assert reader.getframerate() == RATE and reader.getnchannels() == 2
        assert len(decoded) == len(samples)
    rms = float(np.sqrt(np.mean(decoded ** 2)))
    metadata = {
        "file": path.name, "kind": "music" if music else "effect", "loop": loop,
        "duration_seconds": round(len(decoded) / RATE, 5), "sample_rate": RATE,
        "channels": 2, "pcm_bits": 16, "peak_dbfs": round(20 * math.log10(max(float(np.max(np.abs(decoded))), 1e-9)), 2),
        "rms_dbfs": round(20 * math.log10(max(rms, 1e-9)), 2),
        "seam_delta": round(float(np.max(np.abs(decoded[0] - decoded[-1]))), 6),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }
    assert metadata["peak_dbfs"] < -2.0, name
    if loop:
        assert metadata["seam_delta"] < 0.025, (name, metadata)
    else:
        assert np.max(np.abs(decoded[[0, -1]])) < 0.001, name
    print(f"{path.name:20} {metadata['duration_seconds']:7.2f}s  peak {metadata['peak_dbfs']:6.2f} dBFS  RMS {metadata['rms_dbfs']:6.2f} dBFS  seam {metadata['seam_delta']:.6f}")
    return metadata


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--effects-only", action="store_true", help="Regenerate effects only; also resets their deterministic seed.")
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    records = []
    if not args.effects_only:
        for i, name in enumerate(("space", "lunar", "mercury", "rescue")):
            global RNG
            RNG = np.random.default_rng(24856 + i)
            _CACHE.clear()
            records.append(save(name, compose(name), True, True))
    for i, name in enumerate(SFX):
        RNG = np.random.default_rng(248560 + i)
        _CACHE.clear()
        samples, loop = effect(name)
        records.append(save(name, samples, loop, False))
    manifest = OUT / "manifest.json"
    if args.effects_only and manifest.exists():
        records = [r for r in json.loads(manifest.read_text(encoding="utf-8"))["assets"] if r["kind"] == "music"] + records
    manifest.write_text(json.dumps({"generator": "tools/compose_audio.py", "seed": 24856,
                                    "original_motif": ["F#", "A", "E", "D"], "assets": records}, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
