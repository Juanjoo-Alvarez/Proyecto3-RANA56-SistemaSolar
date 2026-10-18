use crate::astronaut::{Astronaut, SPAWN, WalkInput};
use crate::simulation::*;
use raylib::prelude::*;

pub const MOON: usize = 8;
pub const PAD: Vector3 = Vector3 {
    x: 0.0,
    y: 2.8,
    z: 28.0,
};
pub const BEACONS: [Vector3; 3] = [
    Vector3 {
        x: -22.0,
        y: 4.0,
        z: 4.0,
    },
    Vector3 {
        x: 0.0,
        y: 7.0,
        z: -25.0,
    },
    Vector3 {
        x: 26.0,
        y: 4.0,
        z: -5.0,
    },
];
pub const TRANSITION_SECONDS: f32 = 3.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Orbit,
    Cabin,
    Landing,
    Surface,
    Disembark,
    OnFoot,
    Boarding,
    Launch,
    Rescue,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Zone {
    #[default]
    Moon,
    Mercury,
}
pub struct Expedition {
    pub cabin: crate::cabin::Cabin,
    pub zone: Zone,
    pub rescue: crate::rescue::Rescue,
    pub phase: Phase,
    pub progress: f32,
    pub local: World,
    pub beacons: [bool; 3],
    pub completed: bool,
    pub astronaut: Astronaut,
    pub(crate) boarding_start: Vector3,
}
impl Default for Expedition {
    fn default() -> Self {
        Self {
            cabin: crate::cabin::Cabin::default(),
            zone: Zone::Moon,
            rescue: crate::rescue::Rescue::default(),
            phase: Phase::Orbit,
            progress: 0.0,
            local: surface_ship(),
            beacons: [false; 3],
            completed: false,
            astronaut: Astronaut::default(),
            boarding_start: SPAWN,
        }
    }
}
fn surface_ship() -> World {
    World {
        position: PAD,
        yaw: std::f32::consts::PI,
        ..Default::default()
    }
}
impl Expedition {
    pub fn toggle_cabin(&mut self, orbit: &mut World) -> bool {
        match self.phase {
            Phase::Orbit => {
                orbit.velocity = Vector3::zero();
                orbit.steering = Vector2::zero();
                orbit.thrust = 0.0;
                orbit.autopilot = None;
                if self.rescue.thermal_shield() {
                    self.cabin.recruit(0);
                }
                self.phase = Phase::Cabin;
                true
            }
            Phase::Cabin => {
                self.phase = Phase::Orbit;
                true
            }
            _ => false,
        }
    }
    fn reset_astronaut(&mut self) {
        self.astronaut = Astronaut::default();
        self.astronaut.mercury = self.zone == Zone::Mercury;
        self.rescue.heat = 0.0;
    }
    pub fn landing_zone(&self, orbit: &World) -> Option<Zone> {
        if length(orbit.position - moon_position(orbit.time)) < 13.0 {
            Some(Zone::Moon)
        } else if length(orbit.position - PLANETS[0].position(orbit.time)) < 17.0 {
            Some(Zone::Mercury)
        } else {
            None
        }
    }

    pub fn astronaut_visible(&self) -> bool {
        matches!(
            self.phase,
            Phase::OnFoot | Phase::Disembark | Phase::Boarding | Phase::Rescue
        )
    }
    pub fn disembark(&mut self) {
        if self.phase == Phase::Surface && self.at_pad() {
            self.local = surface_ship();
            self.local.position.y = crate::ship::GROUND_Y;
            self.phase = Phase::Disembark;
            self.progress = 0.0;
            self.reset_astronaut();
        }
    }
    pub fn can_land(&self, orbit: &World) -> bool {
        self.phase == Phase::Orbit
            && self.landing_zone(orbit).is_some()
            && length(orbit.velocity) < 8.0
    }
    pub fn at_pad(&self) -> bool {
        length(self.local.position - PAD) < 8.0 && length(self.local.velocity) < 4.0
    }
    pub fn interact(&mut self, orbit: &mut World) {
        match self.phase {
            Phase::Orbit if self.can_land(orbit) => {
                self.zone = self.landing_zone(orbit).unwrap();
                orbit.autopilot = None;
                orbit.velocity = v(0.0, 0.0, 0.0);
                self.local = surface_ship();
                self.phase = Phase::Landing;
                self.progress = 0.0;
            }
            Phase::Surface if self.at_pad() => {
                self.phase = Phase::Launch;
                self.progress = 0.0;
            }
            Phase::OnFoot
                if self.zone == Zone::Mercury && self.rescue.interact(&self.astronaut) =>
            {
                if self.rescue.status == crate::rescue::Status::Celebration {
                    self.phase = Phase::Rescue;
                }
            }
            Phase::OnFoot if self.astronaut.near_ship() => {
                if self.zone == Zone::Mercury && self.rescue.status == crate::rescue::Status::Escort
                {
                    if !self.rescue.brasa.near_ship() {
                        return;
                    }
                    self.rescue.boarding_start = self.rescue.brasa.position;
                    self.rescue.status = crate::rescue::Status::Aboard;
                }
                self.phase = Phase::Boarding;
                self.progress = 0.0;
                self.boarding_start = self.astronaut.position;
            }
            _ => {}
        }
    }
    pub fn restart(&mut self, orbit: &mut World) {
        if self.phase == Phase::Cabin {
            self.cabin.reset_position();
            return;
        }
        if self.phase == Phase::Rescue {
            return;
        }
        if self.phase == Phase::Orbit {
            orbit.reset();
            orbit.selected = if self.zone == Zone::Moon { MOON } else { 0 };
        } else if self.astronaut_visible() {
            self.reset_astronaut();
            self.phase = Phase::OnFoot;
        } else {
            self.local = surface_ship();
            self.phase = Phase::Surface;
            self.progress = 0.0;
        }
    }
    pub fn update(&mut self, orbit: &mut World, input: Input, walk: WalkInput, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        if orbit.paused {
            return;
        }
        match self.phase {
            Phase::Orbit => orbit.update(input, dt),
            Phase::Cabin => self.cabin.update(walk, dt),
            Phase::Landing | Phase::Launch => {
                self.progress = (self.progress + dt / TRANSITION_SECONDS).min(1.0);
                let t = self.progress * self.progress * (3.0 - 2.0 * self.progress);
                let height = if self.phase == Phase::Landing {
                    1.0 - t
                } else {
                    t
                };
                self.local.position = PAD + v(0.0, height * 38.0, height * 12.0);
                self.local.thrust = 0.65;
                self.local.time += dt;
                if self.progress >= 1.0 {
                    if self.phase == Phase::Landing {
                        self.phase = Phase::Surface;
                        self.disembark();
                    } else {
                        self.phase = Phase::Orbit;
                        orbit.position = destination_position(
                            if self.zone == Zone::Moon { MOON } else { 0 },
                            orbit.time,
                        ) + v(0.0, 5.0, 10.0);
                        if self.zone == Zone::Mercury
                            && self.rescue.status == crate::rescue::Status::Aboard
                        {
                            self.rescue.status = crate::rescue::Status::Complete;
                            self.cabin.recruit(0);
                        }
                        orbit.velocity = v(0.0, 0.0, 0.0);
                        orbit.yaw = 0.0;
                        orbit.pitch = 0.1;
                        orbit.notify("Ascenso completado. De vuelta en órbita.".into());
                        if self.zone == Zone::Moon && self.beacons.iter().all(|b| *b) {
                            self.completed = true;
                        }
                    }
                }
            }
            Phase::Disembark | Phase::Boarding => {
                self.local.time += dt;
                self.local.thrust = 0.05;
                self.progress = (self.progress + dt / 1.3).min(1.0);
                let t = self.progress * self.progress * (3.0 - 2.0 * self.progress);
                let door = crate::ship::DOOR;
                let (from, to) = if self.phase == Phase::Disembark {
                    (door, SPAWN)
                } else {
                    (self.boarding_start, door)
                };
                self.astronaut.position = from + (to - from) * t;
                if self.phase == Phase::Boarding
                    && self.zone == Zone::Mercury
                    && self.rescue.status == crate::rescue::Status::Aboard
                {
                    self.rescue.brasa.position =
                        self.rescue.boarding_start + (door - self.rescue.boarding_start) * t;
                    self.rescue.brasa.time += dt;
                    self.rescue.brasa.animation += dt * 10.0;
                    self.rescue.brasa.velocity = v(-2.0, 0.0, 0.0);
                    self.rescue.brasa.yaw = -std::f32::consts::FRAC_PI_2;
                }
                self.astronaut.time += dt;
                self.astronaut.animation += dt * 10.0;
                self.astronaut.velocity = v(2.0, 0.0, 0.0);
                self.astronaut.yaw = if self.phase == Phase::Disembark {
                    std::f32::consts::FRAC_PI_2
                } else {
                    -std::f32::consts::FRAC_PI_2
                };
                if self.progress >= 1.0 {
                    if self.phase == Phase::Disembark {
                        self.phase = Phase::OnFoot;
                        self.reset_astronaut();
                    } else {
                        self.phase = Phase::Surface;
                        self.local = surface_ship();
                        self.local.thrust = 0.12;
                    }
                }
            }
            Phase::OnFoot => {
                self.local.time += dt;
                self.local.thrust = 0.05;
                self.astronaut.update(walk, dt);
                if self.zone == Zone::Mercury {
                    if !self.rescue.unlocked() && (self.astronaut.position.x - 8.0).abs() < 6.5 {
                        self.astronaut.position.z = self.astronaut.position.z.max(-28.2);
                    }
                    self.rescue.update(&mut self.astronaut, dt);
                }
                for (i, beacon) in BEACONS
                    .iter()
                    .enumerate()
                    .filter(|_| self.zone == Zone::Moon)
                {
                    if length(self.astronaut.position + v(0.0, 1.4, 0.0) - *beacon) < 2.7 {
                        self.beacons[i] = true;
                    }
                }
            }
            Phase::Rescue => {
                self.local.time += dt;
                self.rescue.update(&mut self.astronaut, dt);
                if self.rescue.status == crate::rescue::Status::Escort {
                    self.phase = Phase::OnFoot;
                }
            }
            Phase::Surface => {
                let ship = &mut self.local;
                ship.time += dt;
                ship.steer(input, dt, 1.7, 0.6);
                let mut direction = ship.forward() * input.thrust
                    + ship.right() * input.strafe
                    + v(0.0, input.vertical, 0.0);
                if length(direction) > 1.0 {
                    direction = unit(direction);
                }
                let desired = if input.brake {
                    v(0.0, 0.0, 0.0)
                } else {
                    direction * if input.boost { 22.0 } else { 12.0 }
                };
                ship.velocity += (desired - ship.velocity)
                    * (1.0 - (-if input.brake { 10.0 } else { 4.0 } * dt).exp());
                ship.position += ship.velocity * dt;
                // Volumen de vuelo de la zona de entrenamiento, con suelo y techo seguros.
                let bounded = v(
                    ship.position.x.clamp(-52.0, 52.0),
                    ship.position.y.clamp(2.8, 24.0),
                    ship.position.z.clamp(-52.0, 52.0),
                );
                if bounded.x != ship.position.x {
                    ship.velocity.x = 0.0;
                }
                if bounded.y != ship.position.y {
                    ship.velocity.y = 0.0;
                }
                if bounded.z != ship.position.z {
                    ship.velocity.z = 0.0;
                }
                ship.position = bounded;
                ship.thrust = (length(ship.velocity) / 22.0).clamp(0.12, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cabin_stops_flight_preserves_orbit_and_crew_and_respects_pause() {
        let mut world = World::default();
        world.travel(0);
        world.velocity = v(20.0, 4.0, 6.0);
        let original = (world.position, world.yaw, world.pitch, world.time);
        let mut e = Expedition::default();
        assert!(e.toggle_cabin(&mut world));
        assert_eq!(e.cabin.count(), 0);
        assert!(world.autopilot.is_none());
        for _ in 0..120 {
            e.update(
                &mut world,
                Input {
                    thrust: 1.0,
                    ..Default::default()
                },
                WalkInput {
                    forward: 1.0,
                    ..Default::default()
                },
                1.0 / 60.0,
            );
        }
        assert_eq!(
            (world.position, world.yaw, world.pitch, world.time),
            original
        );
        assert_eq!(world.velocity, Vector3::zero());
        let saved = e.cabin.captain.position;
        world.paused = true;
        e.update(
            &mut world,
            Input::default(),
            WalkInput {
                horizontal: 1.0,
                ..Default::default()
            },
            0.05,
        );
        assert_eq!(e.cabin.captain.position, saved);
        world.paused = false;
        assert!(e.toggle_cabin(&mut world));
        assert_eq!(e.phase, Phase::Orbit);
        e.rescue.status = crate::rescue::Status::Complete;
        assert!(e.toggle_cabin(&mut world));
        assert_eq!(e.cabin.count(), 1);
        e.restart(&mut world);
        assert_eq!(e.cabin.count(), 1);
        e.phase = Phase::OnFoot;
        assert!(!e.toggle_cabin(&mut world));
    }
    #[test]
    fn first_rescue_plays_from_orbit_through_dialogue_and_returns_with_brasa() {
        let mut world = World::default();
        world.travel(0);
        let mut e = Expedition::default();
        let mut pilot = crate::rescue::RescuePilot::default();
        let mut comms = crate::comms::Comms::default();
        let mut celebrated = false;
        for frame in 0..9000 {
            let mut walk = WalkInput::default();
            if comms.blocking() {
                comms.reveal_all();
                comms.advance();
            } else {
                match e.phase {
                    Phase::Orbit if e.can_land(&world) => e.interact(&mut world),
                    Phase::OnFoot => {
                        walk = pilot.input(&e.astronaut, &e.rescue);
                        if pilot.interact(&e.astronaut, &e.rescue) {
                            e.interact(&mut world);
                        }
                    }
                    Phase::Surface => e.interact(&mut world),
                    _ => {}
                }
                e.update(&mut world, Input::default(), walk, 1.0 / 60.0);
            }
            comms.observe(&e, e.beacons);
            comms.update(1.0 / 60.0);
            celebrated |= e.rescue.high_five() > 0.9;
            if e.rescue.thermal_shield() {
                break;
            }
            assert!(
                frame < 8999,
                "Rescate incompleto {:?}, paso {}, posicion {:?}",
                e.rescue.status,
                pilot.waypoint,
                e.astronaut.position
            );
        }
        assert!(celebrated);
        assert_eq!(e.phase, Phase::Orbit);
        assert_eq!(e.zone, Zone::Mercury);
        assert!(e.rescue.thermal_shield());
        assert_eq!(e.cabin.count(), 1);
        assert_eq!(e.rescue.overheats, 0);
        assert_eq!(comms.conversations_finished, 1);
        assert!(length(world.position - PLANETS[0].position(world.time)) < 17.0);
        assert!(!e.completed);
    }
    #[test]
    fn landing_requires_proximity_and_slow_speed() {
        let mut world = World::default();
        let mut e = Expedition::default();
        e.interact(&mut world);
        assert_eq!(e.phase, Phase::Orbit);
        world.position = moon_position(0.0) + v(0.0, 4.0, 8.0);
        world.velocity = v(0.0, 0.0, 10.0);
        assert!(!e.can_land(&world));
        world.velocity *= 0.0;
        assert!(e.can_land(&world));
    }
    #[test]
    fn full_round_trip_preserves_exploration_and_freezes_when_paused() {
        let mut world = World {
            position: moon_position(0.0) + v(0.0, 5.0, 9.0),
            ..Default::default()
        };
        world.visited[2] = true;
        let mut e = Expedition::default();
        e.interact(&mut world);
        world.paused = true;
        e.update(&mut world, Input::default(), WalkInput::default(), 0.05);
        assert_eq!(e.progress, 0.0);
        world.paused = false;
        for _ in 0..120 {
            e.update(&mut world, Input::default(), WalkInput::default(), 0.05);
        }
        assert_eq!(e.phase, Phase::OnFoot);
        world.paused = true;
        let saved = e.astronaut.position;
        let clock = e.astronaut.time;
        e.update(
            &mut world,
            Input::default(),
            WalkInput {
                forward: 1.0,
                jump_pressed: true,
                jump_held: true,
                ..Default::default()
            },
            0.05,
        );
        assert_eq!(saved, e.astronaut.position);
        assert_eq!(clock, e.astronaut.time);
        world.paused = false;
        for (i, beacon) in BEACONS.into_iter().enumerate() {
            e.astronaut.position = v(beacon.x, if i == 1 { 4.2 } else { 1.5 }, beacon.z);
            e.update(&mut world, Input::default(), WalkInput::default(), 0.01);
        }
        e.interact(&mut world);
        assert_eq!(e.phase, Phase::OnFoot);
        e.astronaut = Astronaut::default();
        e.interact(&mut world);
        assert_eq!(e.phase, Phase::Boarding);
        for _ in 0..30 {
            e.update(&mut world, Input::default(), WalkInput::default(), 0.05);
        }
        assert_eq!(e.phase, Phase::Surface);
        e.interact(&mut world);
        for _ in 0..80 {
            e.update(&mut world, Input::default(), WalkInput::default(), 0.05);
        }
        assert_eq!(e.phase, Phase::Orbit);
        assert!(e.completed);
        assert!(world.visited[2]);
        assert!(length(world.position - moon_position(world.time)) > 2.8);
    }
    #[test]
    fn surface_ship_cannot_leave_training_volume() {
        let mut e = Expedition {
            phase: Phase::Surface,
            ..Default::default()
        };
        let mut world = World::default();
        for _ in 0..600 {
            e.update(
                &mut world,
                Input {
                    thrust: 1.0,
                    vertical: -1.0,
                    boost: true,
                    ..Default::default()
                },
                WalkInput::default(),
                0.05,
            );
        }
        assert!(e.local.position.z >= -52.0);
        assert!(e.local.position.y >= 2.8);
    }
}
