use crate::{
    astronaut::{Astronaut, SPAWN, WalkInput},
    simulation::*,
};
use raylib::prelude::*;

pub const COLLECTORS: [Vector3; 3] = [
    Vector3 {
        x: -20.0,
        y: 0.0,
        z: 12.0,
    },
    Vector3 {
        x: 22.0,
        y: 0.0,
        z: 0.0,
    },
    Vector3 {
        x: -16.0,
        y: 0.0,
        z: -20.0,
    },
];
pub const REFUGE: Vector3 = Vector3 {
    x: 8.0,
    y: 0.0,
    z: -32.0,
};
pub const SHELTERS: [Vector3; 6] = [
    COLLECTORS[0],
    COLLECTORS[1],
    COLLECTORS[2],
    REFUGE,
    Vector3 {
        x: 2.0,
        y: 0.0,
        z: 9.0,
    },
    Vector3 {
        x: 4.0,
        y: 0.0,
        z: -13.0,
    },
];
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Waiting,
    Celebration,
    Escort,
    Aboard,
    Complete,
}
pub struct Rescue {
    pub collectors: [bool; 3],
    pub heat: f32,
    pub overheats: u32,
    pub status: Status,
    pub brasa: Astronaut,
    pub celebration: f32,
    pub start: Vector3,
    pub boarding_start: Vector3,
    escort_step: usize,
}
impl Default for Rescue {
    fn default() -> Self {
        let mut brasa = Astronaut::default();
        brasa.mercury = true;
        brasa.position = REFUGE;
        brasa.yaw = 0.0;
        Self {
            collectors: [false; 3],
            heat: 0.0,
            overheats: 0,
            status: Status::Waiting,
            brasa,
            celebration: 0.0,
            start: REFUGE,
            boarding_start: REFUGE,
            escort_step: 0,
        }
    }
}
pub fn shaded(p: Vector3) -> bool {
    length(v(p.x, 0.0, p.z) - v(SPAWN.x, 0.0, SPAWN.z)) < 10.0
        || SHELTERS
            .iter()
            .any(|s| (p.x - s.x).abs() < 5.5 && (p.z - s.z).abs() < 5.5 && p.y < 5.8)
}
impl Rescue {
    pub fn unlocked(&self) -> bool {
        self.collectors.iter().all(|b| *b)
    }
    pub fn thermal_shield(&self) -> bool {
        self.status == Status::Complete
    }
    pub fn collector_near(&self, a: &Astronaut) -> Option<usize> {
        COLLECTORS
            .iter()
            .position(|p| length(a.position - *p) < 4.0 && a.grounded)
    }
    pub fn near_refuge(&self, a: &Astronaut) -> bool {
        length(a.position - (REFUGE + v(0.0, 0.0, 3.0))) < 4.5 && a.grounded
    }
    pub fn interact(&mut self, a: &Astronaut) -> bool {
        if let Some(i) = self.collector_near(a) {
            self.collectors[i] = true;
            return true;
        }
        if self.near_refuge(a) {
            if self.unlocked() && self.status == Status::Waiting {
                self.status = Status::Celebration;
                self.start = a.position;
                self.celebration = 0.0;
            }
            return true;
        }
        false
    }
    pub fn update(&mut self, a: &mut Astronaut, dt: f32) {
        if self.status == Status::Celebration {
            self.celebration = (self.celebration + dt / 4.5).min(1.0);
            let t = (self.celebration * 3.0).min(1.0);
            let t = t * t * (3.0 - 2.0 * t);
            a.position = self.start + (REFUGE + v(0.0, 0.0, 2.0) - self.start) * t;
            a.yaw = std::f32::consts::PI;
            a.velocity = Vector3::zero();
            a.time += dt;
            self.brasa.time += dt;
            if self.celebration >= 1.0 {
                self.status = Status::Escort;
            }
            return;
        }
        self.heat = (self.heat
            + dt * if shaded(a.position) || self.unlocked() {
                -24.0
            } else {
                4.2
            })
        .clamp(0.0, 100.0);
        if self.heat >= 100.0 {
            *a = Astronaut::default();
            a.mercury = true;
            self.heat = 0.0;
            self.overheats += 1;
        }
        if self.status == Status::Escort {
            let route = [
                v(8.0, 0.0, -20.0),
                v(8.0, 0.0, 0.0),
                v(9.0, 0.0, 18.0),
                SPAWN,
            ];
            let target = route[self.escort_step];
            let delta = target - self.brasa.position;
            if length(v(delta.x, 0.0, delta.z)) < 1.0 && self.escort_step < 3 {
                self.escort_step += 1;
            }
            let dir = v(delta.x, 0.0, delta.z) * (1.0 / length(delta).max(1.5));
            self.brasa.camera_yaw = 0.0;
            self.brasa.update(
                WalkInput {
                    forward: dir.z,
                    horizontal: -dir.x,
                    ..Default::default()
                },
                dt,
            );
        }
    }
    pub fn high_five(&self) -> f32 {
        if self.status != Status::Celebration {
            return 0.0;
        }
        let t = self.celebration;
        if t < 0.32 {
            0.0
        } else if t < 0.52 {
            (t - 0.32) / 0.20
        } else if t < 0.72 {
            1.0
        } else {
            (1.0 - t) / 0.28
        }
    }
}

// Recorrido de aceptacion: entradas de caminar e interactuar, sin teletransporte.
#[derive(Default)]
pub struct RescuePilot {
    pub waypoint: usize,
}
impl RescuePilot {
    pub fn input(&mut self, a: &Astronaut, r: &Rescue) -> WalkInput {
        let route = [
            COLLECTORS[0],
            SHELTERS[4],
            COLLECTORS[1],
            SHELTERS[5],
            COLLECTORS[2],
            REFUGE + v(0.0, 0.0, 4.0),
            v(20.0, 0.0, -20.0),
            v(20.0, 0.0, 18.0),
            SPAWN,
        ];
        if r.status == Status::Escort {
            self.waypoint = self.waypoint.max(6);
        }
        let target = route[self.waypoint.min(8)];
        let delta = target - a.position;
        let flat = v(delta.x, 0.0, delta.z);
        let distance = length(flat);
        if distance < 1.5 && self.waypoint < 8 {
            let needs_collector = match self.waypoint {
                0 => !r.collectors[0],
                2 => !r.collectors[1],
                4 => !r.collectors[2],
                5 => true,
                _ => false,
            };
            if !needs_collector {
                self.waypoint += 1;
            }
        }
        let dir = flat * (1.0 / distance.max(1.5));
        let forward = v(a.camera_yaw.sin(), 0.0, a.camera_yaw.cos());
        let right = v(-a.camera_yaw.cos(), 0.0, a.camera_yaw.sin());
        WalkInput {
            forward: dot(dir, forward),
            horizontal: dot(dir, right),
            run: true,
            ..Default::default()
        }
    }
    pub fn interact(&self, a: &Astronaut, r: &Rescue) -> bool {
        r.collector_near(a).is_some_and(|i| !r.collectors[i])
            || (r.near_refuge(a) && r.unlocked() && r.status == Status::Waiting)
            || (r.status == Status::Escort && r.brasa.near_ship() && a.near_ship())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn door_requires_every_collector_and_return_unlocks_upgrade_only_at_completion() {
        let mut r = Rescue::default();
        let mut a = Astronaut::default();
        a.mercury = true;
        a.position = REFUGE + v(0.0, 0.0, 3.0);
        r.interact(&a);
        assert_eq!(r.status, Status::Waiting);
        for p in COLLECTORS {
            a.position = p;
            r.interact(&a);
        }
        a.position = REFUGE + v(0.0, 0.0, 3.0);
        r.interact(&a);
        assert_eq!(r.status, Status::Celebration);
        assert!(!r.thermal_shield());
        for _ in 0..300 {
            r.update(&mut a, 1.0 / 60.0);
        }
        assert_eq!(r.status, Status::Escort);
        for _ in 0..1800 {
            r.update(&mut a, 1.0 / 60.0);
        }
        assert!(r.brasa.near_ship(), "Brasa {:?}", r.brasa.position);
        assert!(!r.thermal_shield());
    }
    #[test]
    fn heat_recovers_in_actual_canopy_and_retry_keeps_collectors() {
        let mut r = Rescue::default();
        let mut a = Astronaut::default();
        a.mercury = true;
        a.position = v(40.0, 0.0, 40.0);
        r.collectors[0] = true;
        for _ in 0..100 {
            r.update(&mut a, 0.05);
        }
        assert!(r.heat > 20.0);
        a.position = SHELTERS[4];
        for _ in 0..100 {
            r.update(&mut a, 0.05);
        }
        assert_eq!(r.heat, 0.0);
        a.position = v(40.0, 0.0, 40.0);
        for _ in 0..500 {
            r.update(&mut a, 0.05);
        }
        assert_eq!(r.overheats, 1);
        assert!(a.near_ship());
        assert!(r.collectors[0]);
    }
}
