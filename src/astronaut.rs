use crate::simulation::*;
use raylib::prelude::*;
use std::f32::consts::{PI, TAU};

pub const RADIUS: f32 = 0.48;
const HEIGHT: f32 = 3.25;
const GRAVITY: f32 = 16.0;
pub const SPAWN: Vector3 = Vector3 {
    x: 6.5,
    y: 0.55,
    z: 28.0,
};

#[derive(Clone, Copy)]
pub struct Platform {
    pub center: Vector3,
    pub size: Vector3,
}
impl Platform {
    pub fn top(self) -> f32 {
        self.center.y + self.size.y * 0.5
    }
    pub fn contains(self, p: Vector3, margin: f32) -> bool {
        (p.x - self.center.x).abs() < self.size.x * 0.5 + margin
            && (p.z - self.center.z).abs() < self.size.z * 0.5 + margin
    }
}

// Piloto de pruebas: recorre el mismo circuito a pie usando los controles normales.
// No cambia posiciones ni teletransporta al personaje.
#[derive(Default)]
pub struct TrialPilot {
    pub waypoint: usize,
    hold: f32,
}
impl TrialPilot {
    pub fn input(&mut self, a: &Astronaut, dt: f32) -> WalkInput {
        let route = [
            v(-22.0, 1.5, 4.0),
            v(-8.0, 1.0, -9.0),
            v(-5.0, 2.2, -16.0),
            v(0.0, 4.2, -25.0),
            v(26.0, 1.5, -5.0),
            SPAWN,
        ];
        let target = route[self.waypoint.min(route.len() - 1)];
        let delta = target - a.position;
        let flat = v(delta.x, 0.0, delta.z);
        let distance = length(flat);
        if distance < 1.3 && (delta.y).abs() < 0.2 && a.grounded && self.waypoint < route.len() - 1
        {
            self.waypoint += 1;
        }
        let forward = v(a.camera_yaw.sin(), 0.0, a.camera_yaw.cos());
        let right = v(-a.camera_yaw.cos(), 0.0, a.camera_yaw.sin());
        let direction = flat * (1.0 / distance.max(4.0));
        let jump =
            self.hold <= 0.0 && a.grounded && !a.charging && delta.y > 0.3 && distance < 10.0;
        if jump {
            self.hold = 0.75;
        }
        let held = self.hold > 0.0;
        self.hold = (self.hold - dt).max(0.0);
        WalkInput {
            forward: dot(direction, forward),
            horizontal: dot(direction, right),
            jump_pressed: jump,
            jump_held: held,
            run: true,
            ..Default::default()
        }
    }
}
const fn block(x: f32, z: f32, w: f32, height: f32, depth: f32) -> Platform {
    Platform {
        center: Vector3 {
            x,
            y: height * 0.5,
            z,
        },
        size: Vector3 {
            x: w,
            y: height,
            z: depth,
        },
    }
}
// Plataformas de salto, escalones y dos contenedores de entrenamiento.
pub const PLATFORMS: [Platform; 8] = [
    block(-22.0, 4.0, 8.0, 1.5, 8.0),
    block(-8.0, -9.0, 6.0, 1.0, 6.0),
    block(-5.0, -16.0, 6.0, 2.2, 6.0),
    block(0.0, -25.0, 10.0, 4.2, 9.0),
    block(26.0, -5.0, 8.0, 1.5, 8.0),
    block(14.0, 9.0, 5.0, 1.1, 6.0),
    block(-12.0, 17.0, 5.0, 2.5, 4.0),
    block(12.0, 32.0, 4.0, 2.5, 5.0),
];
pub const SHIP_BLOCK: Platform = Platform {
    center: Vector3 {
        x: 0.0,
        y: 1.8,
        z: 28.0,
    },
    size: Vector3 {
        x: 7.0,
        y: 2.5,
        z: 7.0,
    },
};
pub const TERMINAL: Platform = Platform {
    center: Vector3 {
        x: 18.0,
        y: 0.95,
        z: 24.0,
    },
    size: Vector3 {
        x: 1.8,
        y: 1.9,
        z: 1.4,
    },
};
fn solids(mercury: bool) -> impl Iterator<Item = Platform> {
    PLATFORMS
        .into_iter()
        .filter(move |_| !mercury)
        .chain(
            [SHIP_BLOCK, TERMINAL]
                .into_iter()
                .filter(move |b| !mercury || b.center.z == 28.0),
        )
        .chain(MERCURY_BLOCKS.into_iter().filter(move |_| mercury))
        .chain(
            crate::rescue::SHELTERS
                .into_iter()
                .map(|p| Platform {
                    center: p + v(0.0, 6.5, 0.0),
                    size: v(11.0, 0.3, 11.0),
                })
                .filter(move |_| mercury),
        )
        .chain(
            crate::rescue::COLLECTORS
                .into_iter()
                .map(|p| Platform {
                    center: p + v(0.0, 1.05, -2.8),
                    size: v(1.4, 2.1, 1.4),
                })
                .filter(move |_| mercury),
        )
}
pub const MERCURY_BLOCKS: [Platform; 3] = [
    block(8.0, -37.0, 14.0, 6.5, 1.0),
    block(1.0, -33.0, 1.0, 6.5, 9.0),
    block(15.0, -33.0, 1.0, 6.5, 9.0),
];

#[derive(Clone, Copy, Default)]
pub struct WalkInput {
    pub horizontal: f32,
    pub forward: f32,
    pub look_x: f32,
    pub look_y: f32,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub run: bool,
}
pub struct Astronaut {
    pub mercury: bool,
    pub position: Vector3,
    pub velocity: Vector3,
    pub yaw: f32,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub grounded: bool,
    pub charge: f32,
    pub charging: bool,
    pub landing: f32,
    pub animation: f32,
    pub time: f32,
    coyote: f32,
    jump_buffer: f32,
    jump_lock: bool,
}
impl Default for Astronaut {
    fn default() -> Self {
        Self {
            mercury: false,
            position: SPAWN,
            velocity: v(0.0, 0.0, 0.0),
            yaw: PI,
            camera_yaw: PI,
            camera_pitch: 0.32,
            grounded: true,
            charge: 0.0,
            charging: false,
            landing: 0.0,
            animation: 0.0,
            time: 0.0,
            coyote: 0.1,
            jump_buffer: 0.0,
            jump_lock: false,
        }
    }
}
pub fn floor(p: Vector3) -> f32 {
    if p.x * p.x + (p.z - 28.0).powi(2) < 9.0 * 9.0 {
        0.55
    } else {
        0.0
    }
}
impl Astronaut {
    pub fn near_ship(&self) -> bool {
        length(self.position - SPAWN) < 4.5 && self.grounded
    }
    pub fn near_terminal(&self) -> bool {
        length(v(self.position.x, 0.0, self.position.z) - v(18.0, 0.0, 24.0)) < 3.8 && self.grounded
    }
    pub fn update(&mut self, input: WalkInput, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        // Subpasos para no atravesar bordes ni perder plataformas a tasas bajas.
        let steps = (dt / (1.0 / 120.0)).ceil().max(1.0) as usize;
        for i in 0..steps {
            self.step(
                WalkInput {
                    jump_pressed: input.jump_pressed && i == 0,
                    ..input
                },
                dt / steps as f32,
            );
        }
    }
    fn step(&mut self, input: WalkInput, dt: f32) {
        self.time += dt;
        self.camera_yaw = (self.camera_yaw + input.look_x * dt * 2.1).rem_euclid(TAU);
        self.camera_pitch = (self.camera_pitch + input.look_y * dt * 1.5).clamp(0.08, 0.95);
        self.landing = (self.landing - dt * 4.0).max(0.0);
        self.coyote = if self.grounded {
            0.10
        } else {
            (self.coyote - dt).max(0.0)
        };
        self.jump_buffer = if input.jump_pressed {
            0.13
        } else {
            (self.jump_buffer - dt).max(0.0)
        };
        if !input.jump_held {
            self.jump_lock = false;
        }
        if !self.charging && !self.jump_lock && self.jump_buffer > 0.0 && self.coyote > 0.0 {
            self.charging = true;
            self.charge = 0.0;
            self.jump_buffer = 0.0;
        }
        if self.charging {
            self.charge = (self.charge + dt / 0.65).min(1.0);
            if !input.jump_held || self.charge >= 1.0 {
                self.velocity.y = 6.6 + self.charge * 3.9;
                self.charging = false;
                self.charge = 0.0;
                self.grounded = false;
                self.coyote = 0.0;
                self.jump_lock = true;
            }
        }
        let forward = v(self.camera_yaw.sin(), 0.0, self.camera_yaw.cos());
        let right = v(-self.camera_yaw.cos(), 0.0, self.camera_yaw.sin());
        let mut direction = forward * input.forward + right * input.horizontal;
        let strength = length(direction);
        if strength > 1.0 {
            direction = unit(direction);
        }
        let speed = if self.charging {
            2.0
        } else if input.run {
            9.0
        } else {
            5.8
        };
        let desired = direction * speed;
        let response = 1.0 - (-if self.grounded { 18.0 } else { 6.0 } * dt).exp();
        self.velocity.x += (desired.x - self.velocity.x) * response;
        self.velocity.z += (desired.z - self.velocity.z) * response;
        if strength > 0.03 {
            let angle = direction.x.atan2(direction.z);
            let delta = (angle - self.yaw + PI).rem_euclid(TAU) - PI;
            self.yaw += delta * (1.0 - (-14.0 * dt).exp());
        }
        self.animation += length(v(self.velocity.x, 0.0, self.velocity.z)) * dt * 2.4;
        let old = self.position;
        // Movimiento horizontal con deslizamiento por las paredes.
        for axis in 0..2 {
            if axis == 0 {
                self.position.x += self.velocity.x * dt;
            } else {
                self.position.z += self.velocity.z * dt;
            }
            for b in solids(self.mercury) {
                if self.position.y >= b.top() - 0.03
                    || self.position.y + HEIGHT <= b.center.y - b.size.y * 0.5
                {
                    continue;
                }
                if b.contains(self.position, RADIUS) {
                    if axis == 0 {
                        self.position.x = old.x;
                        self.velocity.x = 0.0;
                    } else {
                        self.position.z = old.z;
                        self.velocity.z = 0.0;
                    }
                }
            }
        }
        self.position.x = self.position.x.clamp(-49.0, 49.0);
        self.position.z = self.position.z.clamp(-49.0, 49.0);
        self.velocity.y -= GRAVITY * dt;
        self.position.y += self.velocity.y * dt;
        let was_grounded = self.grounded;
        self.grounded = false;
        let mut support = floor(self.position);
        for b in solids(self.mercury) {
            if b.contains(self.position, RADIUS * 0.55) {
                if self.velocity.y <= 0.0 && old.y >= b.top() - 0.04 && self.position.y <= b.top() {
                    support = support.max(b.top());
                }
                let bottom = b.center.y - b.size.y * 0.5;
                if self.velocity.y > 0.0
                    && old.y + HEIGHT <= bottom
                    && self.position.y + HEIGHT >= bottom
                {
                    self.position.y = bottom - HEIGHT;
                    self.velocity.y = 0.0;
                }
            }
        }
        if self.position.y <= support {
            if !was_grounded && self.velocity.y < -2.0 {
                self.landing = (-self.velocity.y / 12.0).min(1.0);
            }
            self.position.y = support;
            self.velocity.y = 0.0;
            self.grounded = true;
        }
        // El borde bajo de la plataforma de la nave funciona como un escalon.
        if self.grounded && floor(self.position) > self.position.y {
            self.position.y = floor(self.position);
        }
    }
    pub fn camera(&self) -> Camera3D {
        let target = self.position + v(0.0, 1.8, 0.0);
        let forward = v(self.camera_yaw.sin(), 0.0, self.camera_yaw.cos());
        let desired = target - forward * (9.5 * self.camera_pitch.cos())
            + v(0.0, 9.5 * self.camera_pitch.sin(), 0.0);
        Camera3D::perspective(
            self.clip_camera(target, desired),
            target,
            v(0.0, 1.0, 0.0),
            55.0,
        )
    }
    pub fn clip_camera(&self, target: Vector3, desired: Vector3) -> Vector3 {
        let mut safe = target;
        for i in 1..=48 {
            let p = target + (desired - target) * (i as f32 / 48.0);
            if solids(self.mercury).any(|b| {
                b.contains(p, 0.2)
                    && p.y > b.center.y - b.size.y * 0.5 - 0.2
                    && p.y < b.top() + 0.25
            }) {
                break;
            }
            safe = p;
        }
        safe.y = safe.y.max(floor(safe) + 0.5);
        safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trial_walks_and_jumps_through_all_platforms_then_returns_to_ship() {
        let mut a = Astronaut::default();
        let mut pilot = TrialPilot::default();
        let mut beacons = [false; 3];
        for _ in 0..9000 {
            let input = pilot.input(&a, 1.0 / 60.0);
            a.update(input, 1.0 / 60.0);
            for (i, b) in crate::expedition::BEACONS.iter().enumerate() {
                if length(a.position + v(0.0, 1.4, 0.0) - *b) < 2.7 {
                    beacons[i] = true;
                }
            }
            if pilot.waypoint == 5 && a.near_ship() {
                break;
            }
        }
        assert!(
            beacons.iter().all(|b| *b),
            "Balizas {beacons:?}; paso {}; posicion {:?}",
            pilot.waypoint,
            a.position
        );
        assert!(
            pilot.waypoint == 5 && a.near_ship(),
            "paso {} posicion {:?}",
            pilot.waypoint,
            a.position
        );
    }
    fn jump_peak(hold: usize) -> f32 {
        let mut a = Astronaut {
            position: v(35.0, 0.0, 35.0),
            ..Default::default()
        };
        let mut peak = 0.0f32;
        for i in 0..300 {
            a.update(
                WalkInput {
                    jump_pressed: i == 0,
                    jump_held: i < hold,
                    ..Default::default()
                },
                1.0 / 120.0,
            );
            peak = peak.max(a.position.y);
        }
        assert!(a.grounded);
        assert_eq!(a.position.y, 0.0);
        peak
    }
    #[test]
    fn charged_jump_is_higher_and_lands_without_auto_repeating() {
        let short = jump_peak(4);
        let charged = jump_peak(100);
        assert!(short > 1.0 && short < 1.8);
        assert!(charged > 3.2 && charged < 3.7);
        let mut a = Astronaut::default();
        for i in 0..600 {
            a.update(
                WalkInput {
                    jump_pressed: i == 0,
                    jump_held: true,
                    ..Default::default()
                },
                1.0 / 120.0,
            );
        }
        assert!(a.grounded);
        assert!(!a.charging);
    }
    #[test]
    fn camera_relative_motion_is_bounded_and_frame_rate_independent() {
        let run = |hz: i32| {
            let mut a = Astronaut {
                position: v(35.0, 0.0, 30.0),
                camera_yaw: PI * 0.5,
                ..Default::default()
            };
            for _ in 0..hz * 2 {
                a.update(
                    WalkInput {
                        forward: 1.0,
                        ..Default::default()
                    },
                    1.0 / hz as f32,
                );
            }
            a
        };
        let a = run(30);
        let b = run(120);
        assert!(a.position.x > 45.0);
        assert!((a.position - b.position).length() < 0.08);
        assert!(a.position.x <= 49.0);
        assert!(a.position.y >= 0.0);
    }
    #[test]
    fn walls_block_walking_and_platform_catches_a_fall() {
        let mut a = Astronaut {
            position: v(-22.0, 0.0, 12.0),
            ..Default::default()
        };
        for _ in 0..180 {
            a.update(
                WalkInput {
                    forward: 1.0,
                    ..Default::default()
                },
                1.0 / 60.0,
            );
        }
        assert!(a.position.z >= 8.4);
        a.position = v(-22.0, 6.0, 4.0);
        a.velocity = v(0.0, -1.0, 0.0);
        for _ in 0..120 {
            a.update(WalkInput::default(), 1.0 / 60.0);
        }
        assert!(a.grounded);
        assert!((a.position.y - 1.5).abs() < 0.001);
        let cam = a.camera();
        assert!(cam.position.y > 0.5);
        a.position = v(0.0, 0.0, -18.0);
        a.camera_yaw = 0.0;
        a.camera_pitch = 0.1;
        let camera = a.camera();
        assert!(
            camera.position.z > -20.5,
            "La camara atraviesa la plataforma central"
        );
        assert!(length(camera.position - camera.target) < 4.0);
    }
}
