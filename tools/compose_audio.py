"""Compose the original RANA-56 score and effects using only NumPy + Python.

Run: python tools/compose_audio.py
No samples, recordings, downloaded instruments, or external encoders are used.
Local ambient audition: slow harmony, soft piano, filtered water and long rests.
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
    if kind == "piano":
        # Rounded felt-like attack and mostly fundamental energy; no metallic partials.
        sound = sum(gain * np.sin(phase * harmonic) * np.exp(-t / decay)
                    for harmonic, gain, decay in ((1, 1.0, 2.8), (2, 0.18, 1.6), (3, 0.035, 0.8)))
        sound *= envelope(n, 0.035, min(1.0, duration * 0.35))
    elif kind == "air":
        sound = np.sin(phase) + 0.12 * np.sin(phase * 2) + 0.025 * np.sin(phase * 3)
        sound *= envelope(n, min(3.2, duration * 0.28), min(4.5, duration * 0.4))
    elif kind == "strings":
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


# Original, unhurried voicings: Dadd9, Gmaj7, Bm7, Asus2.
HARMONIES = [(50, 57, 64, 66), (43, 54, 57, 62), (47, 54, 57, 62), (45, 52, 59, 64)]
MUSIC = ("space", "lunar", "mercury", "rescue", "cabin", "tension")


def compose(name: str) -> np.ndarray:
    duration = {"space": 144, "lunar": 160, "mercury": 144, "cabin": 152,
                "rescue": 12, "tension": 24}[name]
    n = round(duration * RATE)
    output = np.zeros((n, 2), np.float32)
    if name == "rescue":
        # One short, gentle resolution. No drums, brass, or repeating fanfare.
        for i, note in enumerate((62, 66, 69, 64, 62)):
            add(output, tone("piano", note, 5.0), 0.8 + i * 1.15, 0.085, (i-2)*0.12)
        for j, note in enumerate((50, 57, 62, 66)):
            add(output, tone("air", note, 10.0), 0.0, 0.034, (j-1.5)*0.32)
        return room(output, 0.19, False) * envelope(n, 0.6, 2.5)[:, None]
    if name == "tension":
        # Low, consonant breathing layer; runtime only introduces it during heat danger.
        for note, gain in ((38, 0.045), (45, 0.025), (50, 0.018)):
            for at in (0, 8, 16):
                add(output, tone("air", note, 12.0), at, gain, 0.0, True)
        return room(output, 0.10, True)
    span = duration / 8.0
    t = np.arange(n, dtype=np.float32) / RATE
    # Very faint synthesized flowing water / air, no insect chirps or bird calls.
    for channel in range(2):
        water = filtered_noise(n, 720 if name == "cabin" else 500, 95)
        water *= 0.004 * (0.72 + 0.28 * np.sin(TAU*t/duration*(3+channel)))
        output[:,channel] += water
    themes = {
        "space": [(3.0,62),(9.2,69)], "lunar": [(4.0,57),(12.0,64)],
        "mercury": [(3.8,54),(10.5,62)], "cabin": [(2.0,62),(8.0,66),(13.0,69)],
    }
    for section in range(8):
        chord = HARMONIES[(section + (1 if name == "lunar" else 0)) % 4]
        start = section * span
        # Sections 3 and 7 are ambience-only interludes (18-20 s), not endless melody.
        if section in (3,7):
            continue
        for j, note in enumerate(chord[:3]):
            add(output, tone("air", note, span * 0.94), start + j*0.28,
                0.027 if name == "cabin" else 0.040, (j-1)*0.50, True)
        # The second half changes contour and timing instead of replaying a short phrase.
        if section in (2,6) and name != "cabin":
            continue
        for k, (offset, note) in enumerate(themes[name]):
            if section >= 4:
                note = chord[(k+1)%len(chord)] + (12 if k == 1 else 0)
                offset += 1.4
            add(output, tone("piano", note, 5.8), start + offset,
                0.11 if name == "cabin" else 0.085, -0.2 + k*0.17, True)
        if name == "lunar" and section in (1,5):
            add(output, tone("piano", chord[-1], 6.0), start+span-5.5, 0.035, -0.3, True)
    return room(output, 0.24 if name != "cabin" else 0.16, True)


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
        source = 0.24 * filtered_noise(n, 310, 75) + 0.035 * np.sin(TAU * 110 * t)
        source *= 0.97 + 0.03 * np.sin(TAU * 0.5 * t)
        add(sound, source, 0, 1.0, 0)
    elif name == "charge":
        source = 0.24 * np.sin(TAU * 147 * t) + 0.045 * np.sin(TAU * 220 * t)
        source += 0.12 * filtered_noise(n, 500, 100)
        source *= 0.95 + 0.05 * np.cos(TAU * 0.5 * t)
        add(sound, source, 0, 0.38)
    elif name in ("turbo", "brake", "launch"):
        a, b = {"turbo": (105, 260), "brake": (180, 55), "launch": (65, 230)}[name]
        add(sound, swoop(duration - 0.15, a, b, 0.45), 0, 0.35)
        if name != "brake":
            add(sound, tone("piano", 62, 0.6), duration * 0.5, 0.06, 0.15)
    elif name in ("step", "foot_land", "land"):
        source = filtered_noise(n, 850 if name == "step" else 560) * np.exp(-t * (45 if name == "step" else 17))
        source += 0.6 * np.sin(TAU * 72 * t) * np.exp(-t * 28)
        source *= envelope(n, 0.004, min(duration / 2, 0.2))
        add(sound, source, 0, 0.35)
        if name == "land":
            add(sound, swoop(0.65, 90, 44, 0.1), 0.08, 0.18)
            add(sound, tone("piano", 66, 0.65), 0.4, 0.12, -0.2)
            add(sound, tone("piano", 69, 0.65), 0.53, 0.09, 0.2)
    elif name == "jump":
        add(sound, swoop(0.34, 120, 220, 0.07), 0, 0.17)
        add(sound, tone("piano", 62, 0.4), 0.06, 0.12)
    elif name in ("collector", "unlock", "rescue_success"):
        notes = {"collector": [66, 69, 76, 74], "unlock": [62, 66, 69, 74, 78, 81],
                 "rescue_success": [66, 69, 76, 74, 78, 81, 86]}[name]
        spacing = 0.14 if name != "rescue_success" else 0.25
        for i, note in enumerate(notes):
            add(sound, tone("piano", note - 12,
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
        add(sound, croak(duration, 220 if name == "luma_talk" else 150), 0, 0.28)
    elif name == "radio_open":
        add(sound, tone("piano", 62, 0.25), 0, 0.17, -0.12)
        add(sound, tone("piano", 66, 0.28), 0.12, 0.15, 0.12)
    elif name == "ui_move":
        add(sound, tone("piano", 62, duration), 0, 0.19)
    elif name == "ui_confirm":
        add(sound, tone("piano", 62, 0.40), 0, 0.17, -0.15)
        add(sound, tone("piano", 66, 0.35), 0.10, 0.14, 0.15)
    elif name == "heat_warning":
        for start in (0.0,):
            add(sound, tone("piano", 62, 0.48), start, 0.19)
            add(sound, tone("piano", 57, 0.40), start + 0.06, 0.10)
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
    if music:
        rms = float(np.sqrt(np.mean(samples ** 2)))
        samples *= min(0.40 / max(peak, 0.001), 0.070 / max(rms, 0.001))
    else:
        desired_peak = 0.60 if name == "high_five" else (0.22 if name in ("engine", "charge") else 0.32)
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
        for i, name in enumerate(MUSIC):
            global RNG
            RNG = np.random.default_rng(24856 + i)
            _CACHE.clear()
            records.append(save(name, compose(name), name != "rescue", True))
    for i, name in enumerate(SFX):
        RNG = np.random.default_rng(248560 + i)
        _CACHE.clear()
        samples, loop = effect(name)
        records.append(save(name, samples, loop, False))
    manifest = OUT / "manifest.json"
    if args.effects_only and manifest.exists():
        records = [r for r in json.loads(manifest.read_text(encoding="utf-8"))["assets"] if r["kind"] == "music"] + records
    manifest.write_text(json.dumps({"generator": "tools/compose_audio.py", "seed": 24856,
                                    "direction": "Ambient espacial y naturaleza cosmica - prueba local", "assets": records}, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
