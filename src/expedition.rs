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
    Landing,
    Surface,
    Disembark,
    OnFoot,
    Boarding,
    Launch,
}
pub struct Expedition {
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
    pub fn astronaut_visible(&self) -> bool {
        matches!(
            self.phase,
            Phase::OnFoot | Phase::Disembark | Phase::Boarding
        )
    }
    pub fn disembark(&mut self) {
        if self.phase == Phase::Surface && self.at_pad() {
            self.local = surface_ship();
            self.local.position.y = 1.2;
            self.phase = Phase::Disembark;
            self.progress = 0.0;
            self.astronaut = Astronaut::default();
        }
    }
    pub fn can_land(&self, orbit: &World) -> bool {
        self.phase == Phase::Orbit
            && length(orbit.position - moon_position(orbit.time)) < 13.0
            && length(orbit.velocity) < 8.0
    }
    pub fn at_pad(&self) -> bool {
        length(self.local.position - PAD) < 8.0 && length(self.local.velocity) < 4.0
    }
    pub fn interact(&mut self, orbit: &mut World) {
        match self.phase {
            Phase::Orbit if self.can_land(orbit) => {
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
            Phase::OnFoot if self.astronaut.near_ship() => {
                self.phase = Phase::Boarding;
                self.progress = 0.0;
                self.boarding_start = self.astronaut.position;
            }
            _ => {}
        }
    }
    pub fn restart(&mut self, orbit: &mut World) {
        if self.phase == Phase::Orbit {
            orbit.reset();
            orbit.selected = MOON;
        } else if self.astronaut_visible() {
            self.astronaut = Astronaut::default();
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
                        orbit.position = moon_position(orbit.time) + v(0.0, 5.0, 10.0);
                        orbit.velocity = v(0.0, 0.0, 0.0);
                        orbit.yaw = 0.0;
                        orbit.pitch = 0.1;
                        orbit.notify("Ascenso completado. De vuelta en orbita lunar.".into());
                        if self.beacons.iter().all(|b| *b) {
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
                let door = v(3.0, 1.3, 28.0);
                let (from, to) = if self.phase == Phase::Disembark {
                    (door, SPAWN)
                } else {
                    (self.boarding_start, door)
                };
                self.astronaut.position = from + (to - from) * t;
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
                        self.astronaut = Astronaut::default();
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
                for (i, beacon) in BEACONS.iter().enumerate() {
                    if length(self.astronaut.position + v(0.0, 1.4, 0.0) - *beacon) < 2.7 {
                        self.beacons[i] = true;
                    }
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
