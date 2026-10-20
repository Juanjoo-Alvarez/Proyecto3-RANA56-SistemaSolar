//! Original score, persistent streaming loops and event-driven sound effects.
//! Event detection is independent of the device so gameplay can run without audio.
use crate::{
    comms::{Comms, Portrait},
    expedition::{Expedition, Phase, Zone},
    rescue::{HIGH_FIVE_CONTACT, Status},
    simulation::{World, length},
};
use raylib::prelude::*;
use std::{fs, io, path::Path};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioSettings {
    pub music: f32,
    pub effects: f32,
    pub muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music: 0.45,
            effects: 0.65,
            muted: false,
        }
    }
}

fn level(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback
    }
}

impl AudioSettings {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path).map_or_else(|_| Self::default(), |text| Self::parse(&text))
    }

    fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "music" => {
                    if let Ok(value) = value.trim().parse() {
                        settings.music = level(value, settings.music);
                    }
                }
                "effects" => {
                    if let Ok(value) = value.trim().parse() {
                        settings.effects = level(value, settings.effects);
                    }
                }
                "muted" => {
                    if let Ok(value) = value.trim().parse() {
                        settings.muted = value;
                    }
                }
                _ => {}
            }
        }
        settings
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        fs::write(
            path,
            format!(
                "music={:.2}\neffects={:.2}\nmuted={}\n",
                level(self.music, 0.45),
                level(self.effects, 0.65),
                self.muted
            ),
        )
    }

    fn effects_gain(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            level(self.effects, 0.65)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(usize)]
enum Cue {
    Turbo,
    Brake,
    Land,
    Launch,
    Step,
    Jump,
    FootLand,
    Collector,
    Unlock,
    HighFive,
    RescueSuccess,
    RadioOpen,
    LumaTalk,
    BrasaTalk,
    UiMove,
    UiConfirm,
    HeatWarning,
}
const CUE_FILES: [&str; 17] = [
    "turbo",
    "brake",
    "land",
    "launch",
    "step",
    "jump",
    "foot_land",
    "collector",
    "unlock",
    "high_five",
    "rescue_success",
    "radio_open",
    "luma_talk",
    "brasa_talk",
    "ui_move",
    "ui_confirm",
    "heat_warning",
];
// Quiet voices and footsteps leave room for the score and mission feedback.
const CUE_GAIN: [f32; 17] = [
    0.35, 0.30, 0.50, 0.42, 0.18, 0.32, 0.30, 0.43, 0.46, 0.80, 0.46, 0.16, 0.12, 0.14, 0.22, 0.30,
    0.32,
];

#[derive(Default)]
struct Events {
    cues: [Option<Cue>; 16],
    count: usize,
}
impl Events {
    fn push(&mut self, cue: Cue) {
        if self.count < self.cues.len() {
            self.cues[self.count] = Some(cue);
            self.count += 1;
        }
    }
    fn iter(&self) -> impl Iterator<Item = Cue> + '_ {
        self.cues[..self.count].iter().flatten().copied()
    }
}

#[derive(Clone, Copy)]
struct Snapshot {
    phase: Phase,
    status: Status,
    celebration: f32,
    collectors: [bool; 3],
    beacons: [bool; 3],
    grounded: bool,
    position: Vector3,
    vertical_speed: f32,
    speed: f32,
    boosting: bool,
    overheats: u32,
    messages: u32,
}
impl Snapshot {
    fn new(world: &World, e: &Expedition, comms: &Comms, boosting: bool) -> Self {
        let astronaut = if e.phase == Phase::Cabin {
            &e.cabin.captain
        } else {
            &e.astronaut
        };
        let ship = if matches!(e.phase, Phase::Orbit | Phase::Cabin) {
            world
        } else {
            &e.local
        };
        Self {
            phase: e.phase,
            status: e.rescue.status,
            celebration: e.rescue.celebration,
            collectors: e.rescue.collectors,
            beacons: e.beacons,
            grounded: astronaut.grounded,
            position: astronaut.position,
            vertical_speed: astronaut.velocity.y,
            speed: length(ship.velocity),
            boosting,
            overheats: e.rescue.overheats,
            messages: comms.messages_shown,
        }
    }
}

#[derive(Default)]
struct EventTracker {
    previous: Option<Snapshot>,
    step_distance: f32,
    heat_cooldown: f32,
    brake_cooldown: f32,
    voice_cooldown: f32,
    voice_count: u8,
    high_five_played: bool,
}

impl EventTracker {
    fn observe(
        &mut self,
        world: &World,
        e: &Expedition,
        comms: &Comms,
        boosting: bool,
        paused: bool,
        dt: f32,
    ) -> Events {
        let now = Snapshot::new(world, e, comms, boosting);
        let old = self.previous.replace(now);
        let mut events = Events::default();
        if paused {
            // Advance the snapshot while menus are open: a held button or old event
            // must never replay when the player closes the menu.
            return events;
        }
        self.heat_cooldown = (self.heat_cooldown - dt).max(0.0);
        self.brake_cooldown = (self.brake_cooldown - dt).max(0.0);
        self.voice_cooldown = (self.voice_cooldown - dt).max(0.0);
        if let Some(old) = old {
            if now.phase != old.phase {
                match now.phase {
                    Phase::Launch => events.push(Cue::Launch),
                    Phase::Disembark if old.phase == Phase::Landing => events.push(Cue::Land),
                    _ => {}
                }
                self.step_distance = 0.0;
            }
            let flying = matches!(now.phase, Phase::Orbit | Phase::Surface);
            if flying && now.boosting && now.speed > 1.0 && !(old.boosting && old.speed > 1.0) {
                events.push(Cue::Turbo);
            }
            if flying
                && now.phase == old.phase
                && old.speed > 7.0
                && old.speed - now.speed > dt.max(0.001) * 28.0
                && self.brake_cooldown == 0.0
            {
                events.push(Cue::Brake);
                self.brake_cooldown = 0.8;
            }
            if matches!(now.phase, Phase::OnFoot | Phase::Cabin)
                && old.phase == now.phase
                && now.overheats == old.overheats
            {
                if old.grounded && !now.grounded && now.vertical_speed > 0.5 {
                    events.push(Cue::Jump);
                    self.step_distance = 0.0;
                } else if !old.grounded && now.grounded && old.vertical_speed < -1.0 {
                    events.push(Cue::FootLand);
                    self.step_distance = 0.0;
                } else if old.grounded && now.grounded {
                    let delta = now.position - old.position;
                    let distance = (delta.x * delta.x + delta.z * delta.z).sqrt();
                    if distance < 3.0 {
                        self.step_distance += distance;
                        if self.step_distance > 1.55 {
                            events.push(Cue::Step);
                            self.step_distance %= 1.55;
                        }
                    } else {
                        self.step_distance = 0.0;
                    }
                }
            }
            let collectors_changed = now
                .collectors
                .iter()
                .zip(old.collectors)
                .any(|(&n, o)| n && !o);
            if collectors_changed {
                // The third collector resolves the musical phrase with its own cue.
                events.push(if now.collectors.iter().all(|b| *b) {
                    Cue::Unlock
                } else {
                    Cue::Collector
                });
            }
            if now.beacons.iter().zip(old.beacons).any(|(&n, o)| n && !o) {
                events.push(Cue::Collector);
            }
            if old.celebration < HIGH_FIVE_CONTACT
                && now.celebration >= HIGH_FIVE_CONTACT
                && matches!(old.status, Status::Celebration)
                && matches!(now.status, Status::Celebration | Status::Escort)
                && !self.high_five_played
            {
                events.push(Cue::HighFive);
                self.high_five_played = true;
            }
            if now.status == Status::Waiting && old.status != Status::Waiting {
                self.high_five_played = false;
            }
            if now.status == Status::Complete && old.status != Status::Complete {
                events.push(Cue::RescueSuccess);
            }
            if now.messages != old.messages {
                events.push(Cue::RadioOpen);
                self.voice_count = 0;
                self.voice_cooldown = 0.22;
            }
        } else if comms.active.is_some() {
            events.push(Cue::RadioOpen);
            self.voice_cooldown = 0.22;
        }
        if e.zone == Zone::Mercury
            && e.phase == Phase::OnFoot
            && e.rescue.heat >= 72.0
            && !e.rescue.unlocked()
            && !comms.blocking()
            && self.heat_cooldown == 0.0
        {
            events.push(Cue::HeatWarning);
            self.heat_cooldown = 12.0;
        }
        if let Some(active) = &comms.active
            && !active.complete()
            && self.voice_cooldown == 0.0
            && self.voice_count < 1
        {
            match active.message.speaker.portrait {
                Portrait::Luma => events.push(Cue::LumaTalk),
                Portrait::Brasa => events.push(Cue::BrasaTalk),
                Portrait::Terminal => {}
            }
            self.voice_count += 1;
            self.voice_cooldown = 1.8;
        }
        events
    }
}

struct Track<'a> {
    music: Music<'a>,
    gain: f32,
    started: bool,
    playing: bool,
}
fn approach(current: &mut f32, target: f32, rate: f32, dt: f32) {
    *current += (target - *current) * (1.0 - (-rate * dt).exp());
}
impl<'a> Track<'a> {
    fn load(device: &'a RaylibAudio, path: &Path, looping: bool) -> Result<Self, String> {
        let mut music = device
            .new_music(&path.to_string_lossy())
            .map_err(|e| e.to_string())?;
        music.set_looping(looping);
        music.set_volume(0.0);
        Ok(Self {
            music,
            gain: 0.0,
            started: false,
            playing: false,
        })
    }
    fn restart(&mut self) {
        self.music.stop_stream();
        self.music.set_volume(0.0);
        self.music.play_stream();
        self.started = true;
        self.playing = true;
        self.gain = 0.0;
    }
    fn update(&mut self, target: f32, volume: f32, rate: f32, dt: f32) {
        approach(&mut self.gain, target, rate, dt);
        if target > 0.0 && !self.playing {
            if self.started {
                self.music.resume_stream();
            } else {
                self.music.play_stream();
            }
            self.started = true;
            self.playing = true;
        }
        // A muted track keeps streaming and keeps its place in the composition.
        if self.playing {
            self.music.set_volume(self.gain * volume);
            self.music.update_stream();
            if target == 0.0 && self.gain < 0.001 {
                self.music.pause_stream();
                self.playing = false;
                self.gain = 0.0;
            }
        }
    }
}

fn environment_track(e: &Expedition) -> usize {
    if e.phase == Phase::Cabin {
        4
    } else if e.phase == Phase::Orbit {
        0
    } else if e.zone == Zone::Moon {
        1
    } else {
        2
    }
}

fn danger_gain(e: &Expedition) -> f32 {
    if e.phase == Phase::OnFoot && e.zone == Zone::Mercury && !e.rescue.unlocked() {
        let t = ((e.rescue.heat - 55.0) / 45.0).clamp(0.0, 1.0);
        0.18 * t * t * (3.0 - 2.0 * t)
    } else {
        0.0
    }
}

pub struct Soundscape<'a> {
    tracks: [Track<'a>; 6],
    engine: Track<'a>,
    charge: Track<'a>,
    sounds: Vec<Sound<'a>>,
    tracker: EventTracker,
    music_level: f32,
    engine_pitch: f32,
    rescue_left: f32,
    was_paused: bool,
    counts: [u32; 17],
    track_changes: u32,
    current_track: usize,
}

impl<'a> Soundscape<'a> {
    pub fn load(device: &'a RaylibAudio, root: &Path) -> Result<Self, String> {
        let directory = root.join("assets/audio");
        let music = |name: &str, looping| {
            Track::load(device, &directory.join(format!("{name}.wav")), looping)
        };
        let tracks = [
            music("space", true)?,
            music("lunar", true)?,
            music("mercury", true)?,
            music("rescue", false)?,
            music("cabin", true)?,
            music("tension", true)?,
        ];
        let engine = music("engine", true)?;
        let charge = music("charge", true)?;
        let sounds = CUE_FILES
            .iter()
            .map(|name| {
                device
                    .new_sound(&directory.join(format!("{name}.wav")).to_string_lossy())
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            tracks,
            engine,
            charge,
            sounds,
            tracker: EventTracker::default(),
            music_level: 0.0,
            engine_pitch: 0.75,
            rescue_left: 0.0,
            was_paused: false,
            counts: [0; 17],
            track_changes: 0,
            current_track: usize::MAX,
        })
    }

    fn play(&mut self, cue: Cue, settings: &AudioSettings) {
        self.counts[cue as usize] += 1;
        let sound = &self.sounds[cue as usize];
        sound.set_volume(settings.effects_gain() * CUE_GAIN[cue as usize]);
        // Small deterministic variations keep repeated footsteps organic without
        // introducing nondeterminism to the simulation or loading extra samples.
        let variation = if matches!(cue, Cue::Step | Cue::LumaTalk | Cue::BrasaTalk) {
            [0.96, 1.02, 1.0, 1.04][self.counts[cue as usize] as usize % 4]
        } else {
            1.0
        };
        sound.set_pitch(variation);
        if settings.effects_gain() > 0.0 {
            sound.play();
        }
    }

    pub fn ui_move(&mut self, settings: &AudioSettings) {
        self.play(Cue::UiMove, settings);
    }
    pub fn ui_confirm(&mut self, settings: &AudioSettings) {
        self.play(Cue::UiConfirm, settings);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        world: &World,
        e: &Expedition,
        comms: &Comms,
        settings: &AudioSettings,
        boosting: bool,
        paused: bool,
        dt: f32,
    ) {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.25)
        } else {
            0.0
        };
        if paused && !self.was_paused {
            for (i, sound) in self.sounds.iter().enumerate() {
                if i != Cue::UiMove as usize && i != Cue::UiConfirm as usize {
                    sound.stop();
                }
            }
        }
        self.was_paused = paused;
        for (i, sound) in self.sounds.iter().enumerate() {
            sound.set_volume(settings.effects_gain() * CUE_GAIN[i]);
        }
        let events = self.tracker.observe(world, e, comms, boosting, paused, dt);
        for cue in events.iter() {
            self.play(cue, settings);
            if cue == Cue::HighFive {
                // The score is the celebration fanfare; the effect is only the
                // synchronized palm contact, so two melodies never compete.
                self.tracks[3].restart();
                self.rescue_left = self.tracks[3].music.get_time_length().max(1.0);
            }
        }
        if self.rescue_left > 0.0 {
            self.rescue_left = (self.rescue_left - dt).max(0.0);
        }
        let track = if self.rescue_left > 1.5 {
            3
        } else {
            environment_track(e)
        };
        if track != self.current_track {
            self.current_track = track;
            self.track_changes += 1;
        }
        let duck = if paused {
            0.30
        } else if comms.active.is_some() {
            0.48
        } else {
            1.0
        };
        let ship = if matches!(e.phase, Phase::Orbit | Phase::Cabin) {
            world
        } else {
            &e.local
        };
        let travel_lift = if track == 0 {
            0.86 + 0.14 * (length(ship.velocity) / 65.0).min(1.0)
        } else {
            1.0
        };
        approach(
            &mut self.music_level,
            level(settings.music, 0.45) * duck * travel_lift,
            4.0,
            dt,
        );
        for (i, track_audio) in self.tracks.iter_mut().enumerate() {
            track_audio.update(
                if i == track {
                    1.0
                } else if i == 5 && !paused && !comms.blocking() {
                    danger_gain(e)
                } else {
                    0.0
                },
                if settings.muted {
                    0.0
                } else {
                    self.music_level
                },
                0.85,
                dt,
            );
        }
        let flying = matches!(
            e.phase,
            Phase::Orbit | Phase::Surface | Phase::Landing | Phase::Launch
        );
        let power = (length(ship.velocity) / 65.0)
            .max(ship.thrust * 0.7)
            .clamp(0.0, 1.0);
        let engine_target = if flying && !paused && !comms.blocking() {
            0.12 * power
        } else {
            0.0
        };
        approach(
            &mut self.engine_pitch,
            0.85 + power * 0.18 + if boosting { 0.04 } else { 0.0 },
            5.0,
            dt,
        );
        self.engine.music.set_pitch(self.engine_pitch);
        self.engine
            .update(engine_target, settings.effects_gain(), 9.0, dt);
        let charge_target =
            if e.phase == Phase::OnFoot && e.astronaut.charging && !paused && !comms.blocking() {
                0.08 + e.astronaut.charge * 0.10
            } else {
                0.0
            };
        self.charge.music.set_pitch(0.9 + e.astronaut.charge * 0.15);
        self.charge
            .update(charge_target, settings.effects_gain(), 14.0, dt);
    }

    pub fn report(&self) -> String {
        format!(
            "Audio: {} cambios musicales; pasos={}; saltos={}; colectores={}; apertura={}; palmadas={}; rescates={}; radio={}",
            self.track_changes,
            self.counts[Cue::Step as usize],
            self.counts[Cue::Jump as usize],
            self.counts[Cue::Collector as usize],
            self.counts[Cue::Unlock as usize],
            self.counts[Cue::HighFive as usize],
            self.counts[Cue::RescueSuccess as usize],
            self.counts[Cue::RadioOpen as usize]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialogue_croaks_once_and_heat_warning_waits_twelve_seconds() {
        let world = World::default();
        let mut e = Expedition {
            phase: Phase::OnFoot,
            zone: Zone::Mercury,
            ..Default::default()
        };
        e.rescue.heat = 90.0;
        let mut comms = Comms::default();
        comms.radio_once(
            "quiet_voice",
            "Una frase larga que permanece visible durante esta prueba de audio.",
        );
        let mut tracker = EventTracker::default();
        let (mut croaks, mut warnings) = (0, 0);
        for frame in 0..300 {
            for cue in tracker
                .observe(&world, &e, &comms, false, false, 0.05)
                .iter()
            {
                croaks += usize::from(cue == Cue::LumaTalk);
                warnings += usize::from(cue == Cue::HeatWarning);
            }
            if frame == 120 {
                assert_eq!(warnings, 1);
            }
        }
        assert_eq!(croaks, 1);
        assert_eq!(warnings, 2);
    }
    #[test]
    fn danger_layer_is_gradual_bounded_and_only_for_active_heat() {
        let mut e = Expedition {
            phase: Phase::OnFoot,
            zone: Zone::Mercury,
            ..Default::default()
        };
        let mut previous = 0.0;
        for heat in 0..=100 {
            e.rescue.heat = heat as f32;
            let gain = danger_gain(&e);
            assert!(gain >= previous && gain <= 0.18);
            if heat <= 55 {
                assert_eq!(gain, 0.0);
            }
            previous = gain;
        }
        e.rescue.collectors = [true; 3];
        assert_eq!(danger_gain(&e), 0.0);
        e.rescue.collectors = [false; 3];
        e.phase = Phase::Cabin;
        assert_eq!(danger_gain(&e), 0.0);
    }
    #[test]
    fn cabin_uses_its_own_music_and_footsteps_without_surface_or_flight_cues() {
        let mut e = Expedition {
            phase: Phase::Cabin,
            zone: Zone::Mercury,
            ..Default::default()
        };
        let world = World::default();
        let comms = Comms::default();
        e.rescue.heat = 99.0;
        assert_eq!(environment_track(&e), 4);
        assert_eq!(danger_gain(&e), 0.0);
        let mut tracker = EventTracker::default();
        tracker.observe(&world, &e, &comms, false, false, 1.0 / 60.0);
        e.cabin.captain.position.x += 1.6;
        let events = tracker.observe(&world, &e, &comms, true, false, 1.0 / 60.0);
        assert_eq!(events.iter().collect::<Vec<_>>(), vec![Cue::Step]);
        e.phase = Phase::Orbit;
        assert_eq!(
            tracker
                .observe(&world, &e, &comms, false, false, 1.0 / 60.0)
                .count,
            0
        );
    }
    fn observe(tracker: &mut EventTracker, e: &Expedition, paused: bool) -> Vec<Cue> {
        tracker
            .observe(&World::default(), e, &Comms::default(), false, paused, 0.05)
            .iter()
            .collect()
    }
    #[test]
    fn settings_recover_invalid_values_and_bound_gains() {
        let s = AudioSettings::parse("music=NaN\neffects=8\nmuted=true\nunknown=x");
        assert_eq!(
            s,
            AudioSettings {
                music: 0.45,
                effects: 1.0,
                muted: true
            }
        );
        assert_eq!(AudioSettings::parse("music=-5\neffects=bad").music, 0.0);
        assert_eq!(AudioSettings::parse("effects=bad").effects, 0.65);
    }
    #[test]
    fn high_five_crossing_large_frame_is_once_and_pause_never_replays_it() {
        let mut t = EventTracker::default();
        let mut e = Expedition {
            phase: Phase::Rescue,
            ..Default::default()
        };
        e.rescue.status = Status::Celebration;
        e.rescue.celebration = HIGH_FIVE_CONTACT - 0.1;
        assert!(observe(&mut t, &e, false).is_empty());
        e.rescue.celebration = HIGH_FIVE_CONTACT + 0.2;
        assert_eq!(observe(&mut t, &e, false), vec![Cue::HighFive]);
        assert!(observe(&mut t, &e, true).is_empty());
        assert!(observe(&mut t, &e, false).is_empty());
        e.rescue.status = Status::Escort;
        e.rescue.celebration = 1.0;
        assert!(observe(&mut t, &e, false).is_empty());
    }
    #[test]
    fn collector_resolution_and_success_are_not_repeated_by_menus() {
        let mut t = EventTracker::default();
        let mut e = Expedition::default();
        observe(&mut t, &e, false);
        e.rescue.collectors[0] = true;
        assert_eq!(observe(&mut t, &e, false), vec![Cue::Collector]);
        observe(&mut t, &e, true);
        assert!(observe(&mut t, &e, false).is_empty());
        e.rescue.collectors = [true; 3];
        assert_eq!(observe(&mut t, &e, false), vec![Cue::Unlock]);
        e.rescue.status = Status::Complete;
        assert_eq!(observe(&mut t, &e, false), vec![Cue::RescueSuccess]);
        assert!(observe(&mut t, &e, false).is_empty());
    }
    #[test]
    fn jumping_and_landing_have_no_airborne_steps_or_teleport_footfalls() {
        let mut t = EventTracker::default();
        let mut e = Expedition {
            phase: Phase::OnFoot,
            ..Default::default()
        };
        observe(&mut t, &e, false);
        e.astronaut.grounded = false;
        e.astronaut.velocity.y = 7.0;
        assert_eq!(observe(&mut t, &e, false), vec![Cue::Jump]);
        e.astronaut.position.x += 2.0;
        e.astronaut.velocity.y = -4.0;
        assert!(observe(&mut t, &e, false).is_empty());
        e.astronaut.grounded = true;
        assert_eq!(observe(&mut t, &e, false), vec![Cue::FootLand]);
        e.astronaut.position.x += 30.0;
        assert!(observe(&mut t, &e, false).is_empty());
        e.astronaut.position.x += 1.7;
        assert_eq!(observe(&mut t, &e, false), vec![Cue::Step]);
    }
    #[test]
    fn music_follows_destination_and_fades_without_overshoot() {
        let mut e = Expedition::default();
        assert_eq!(environment_track(&e), 0);
        e.phase = Phase::Landing;
        assert_eq!(environment_track(&e), 1);
        e.zone = Zone::Mercury;
        assert_eq!(environment_track(&e), 2);
        let mut gain = 0.0;
        for _ in 0..60 {
            approach(&mut gain, 1.0, 2.2, 1.0 / 60.0);
        }
        assert!(gain > 0.8 && gain < 1.0);
        approach(&mut gain, 0.0, 2.2, 60.0);
        assert!((0.0..0.001).contains(&gain));
    }

    #[test]
    fn holding_turbo_from_rest_fires_when_ship_starts_moving_once() {
        let mut tracker = EventTracker::default();
        let mut world = World::default();
        let e = Expedition::default();
        let comms = Comms::default();
        assert!(
            tracker
                .observe(&world, &e, &comms, true, false, 0.05)
                .iter()
                .next()
                .is_none()
        );
        world.velocity.x = 2.0;
        let events = tracker.observe(&world, &e, &comms, true, false, 0.05);
        assert_eq!(events.iter().collect::<Vec<_>>(), vec![Cue::Turbo]);
        assert!(
            tracker
                .observe(&world, &e, &comms, true, false, 0.05)
                .iter()
                .next()
                .is_none()
        );
        assert!(
            tracker
                .observe(&world, &e, &comms, true, true, 0.05)
                .iter()
                .next()
                .is_none()
        );
        assert!(
            tracker
                .observe(&world, &e, &comms, true, false, 0.05)
                .iter()
                .next()
                .is_none()
        );
    }
}
