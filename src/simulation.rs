use raylib::prelude::*;
use std::f32::consts::{PI, TAU};

pub fn v(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3::new(x, y, z)
}
pub fn dot(a: Vector3, b: Vector3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}
pub fn length(a: Vector3) -> f32 {
    dot(a, a).sqrt()
}
pub fn unit(a: Vector3) -> Vector3 {
    let n = length(a);
    if n > 0.0001 {
        a * (1.0 / n)
    } else {
        v(0.0, 0.0, 1.0)
    }
}
pub fn cross(a: Vector3, b: Vector3) -> Vector3 {
    v(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

#[derive(Clone, Copy)]
pub struct Planet {
    pub name: &'static str,
    pub subtitle: &'static str,
    pub radius: f32,
    pub orbit: f32,
    pub speed: f32,
    pub phase: f32,
    pub color: Color,
}

// Escalas artisticas: el orden es real, los radios, periodos y distancias son jugables.
pub const PLANETS: [Planet; 8] = [
    Planet {
        name: "MERCURIO",
        subtitle: "Mundo rocoso / crateres",
        radius: 2.0,
        orbit: 30.0,
        speed: 0.045,
        phase: -0.9,
        color: Color::new(178, 167, 150, 255),
    },
    Planet {
        name: "VENUS",
        subtitle: "Atmosfera dorada",
        radius: 3.6,
        orbit: 45.0,
        speed: 0.030,
        phase: 2.6,
        color: Color::new(237, 188, 105, 255),
    },
    Planet {
        name: "TIERRA",
        subtitle: "Oceanos / continentes / Luna",
        radius: 5.0,
        orbit: 66.0,
        speed: 0.018,
        phase: 0.0,
        color: Color::new(70, 170, 229, 255),
    },
    Planet {
        name: "MARTE",
        subtitle: "El planeta rojo",
        radius: 2.9,
        orbit: 88.0,
        speed: 0.014,
        phase: -0.85,
        color: Color::new(225, 108, 76, 255),
    },
    Planet {
        name: "JUPITER",
        subtitle: "Gigante gaseoso / bandas",
        radius: 10.0,
        orbit: 132.0,
        speed: 0.007,
        phase: 1.0,
        color: Color::new(219, 177, 137, 255),
    },
    Planet {
        name: "SATURNO",
        subtitle: "Gigante de los anillos",
        radius: 8.0,
        orbit: 181.0,
        speed: 0.005,
        phase: -1.5,
        color: Color::new(225, 204, 153, 255),
    },
    Planet {
        name: "URANO",
        subtitle: "Gigante de hielo / eje inclinado",
        radius: 5.8,
        orbit: 233.0,
        speed: 0.003,
        phase: 2.4,
        color: Color::new(136, 224, 231, 255),
    },
    Planet {
        name: "NEPTUNO",
        subtitle: "Azul profundo / vientos",
        radius: 5.5,
        orbit: 286.0,
        speed: 0.002,
        phase: -2.7,
        color: Color::new(78, 112, 237, 255),
    },
];

impl Planet {
    pub fn position(&self, time: f32) -> Vector3 {
        let angle = self.phase + time * self.speed;
        v(angle.sin() * self.orbit, 0.0, angle.cos() * self.orbit)
    }
}

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub thrust: f32,
    pub strafe: f32,
    pub vertical: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub steer_yaw: f32,
    pub steer_pitch: f32,
    pub boost: bool,
    pub brake: bool,
}
impl Input {
    fn manual(&self) -> bool {
        self.thrust.abs()
            + self.strafe.abs()
            + self.vertical.abs()
            + self.yaw.abs()
            + self.pitch.abs()
            + self.steer_yaw.abs()
            + self.steer_pitch.abs()
            > 0.001
            || self.brake
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stage {
    Ascend,
    Cruise,
    Approach,
}
#[derive(Clone, Copy, Debug)]
pub struct Autopilot {
    pub target: usize,
    pub stage: Stage,
}

pub struct World {
    pub thermal_shield: bool,
    pub time: f32,
    pub position: Vector3,
    pub velocity: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub steering: Vector2,
    pub selected: usize,
    pub autopilot: Option<Autopilot>,
    pub visited: [bool; 8],
    pub paused: bool,
    pub thrust: f32,
    pub message: String,
    pub message_left: f32,
}
impl Default for World {
    fn default() -> Self {
        Self {
            time: 0.0,
            position: PLANETS[2].position(0.0) + v(0.0, 4.0, 23.0),
            velocity: v(0.0, 0.0, 0.0),
            yaw: PI,
            pitch: 0.0,
            steering: Vector2::zero(),
            selected: 8,
            autopilot: None,
            thermal_shield: false,
            visited: [false; 8],
            paused: false,
            thrust: 0.0,
            message: "Prueba lunar: F / Triangulo para viajar a la Luna.".into(),
            message_left: 7.0,
        }
    }
}
impl World {
    pub fn forward(&self) -> Vector3 {
        v(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
    }
    pub fn right(&self) -> Vector3 {
        v(-self.yaw.cos(), 0.0, self.yaw.sin())
    }
    pub fn steer(&mut self, input: Input, dt: f32, yaw_speed: f32, pitch_limit: f32) {
        // Solo el stick izquierdo tiene entrada/salida amortiguada. El derecho
        // conserva su sensibilidad y respuesta directa de la version anterior.
        let response = 1.0 - (-8.0 * dt).exp();
        self.steering.x += (input.steer_yaw - self.steering.x) * response;
        self.steering.y += (input.steer_pitch - self.steering.y) * response;
        self.yaw = (self.yaw + (input.yaw + self.steering.x).clamp(-1.0, 1.0) * dt * yaw_speed)
            .rem_euclid(TAU);
        self.pitch = (self.pitch + (input.pitch + self.steering.y).clamp(-1.0, 1.0) * dt)
            .clamp(-pitch_limit, pitch_limit);
    }
    pub fn travel(&mut self, index: usize) {
        self.selected = index.min(8);
        self.steering = Vector2::zero();
        self.autopilot = Some(Autopilot {
            target: self.selected,
            stage: Stage::Ascend,
        });
        self.notify(format!("Ruta trazada: {}", destination_name(self.selected)));
    }
    pub fn notify(&mut self, message: String) {
        self.message = message;
        self.message_left = 4.0;
    }
    pub fn reset(&mut self) {
        let visited = self.visited;
        let shield = self.thermal_shield;
        *self = Self::default();
        self.visited = visited;
        self.thermal_shield = shield;
    }
    pub fn target_distance(&self) -> f32 {
        (length(self.position - destination_position(self.selected, self.time))
            - destination_radius(self.selected))
        .max(0.0)
    }
    pub fn update(&mut self, input: Input, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        if self.paused {
            return;
        }
        self.time += dt;
        self.message_left = (self.message_left - dt).max(0.0);
        if input.manual() {
            self.autopilot = None;
        }
        self.steer(input, dt, 1.3, 1.15);
        let mut desired = self.forward() * input.thrust
            + self.right() * input.strafe
            + v(0.0, input.vertical, 0.0);
        if length(desired) > 1.0 {
            desired = unit(desired);
        }
        desired *= if input.boost {
            95.0 * if self.thermal_shield { 1.15 } else { 1.0 }
        } else {
            30.0
        };
        if let Some(mut pilot) = self.autopilot {
            let target = destination_position(pilot.target, self.time);
            let approach = target
                + v(
                    0.0,
                    4.0,
                    destination_radius(pilot.target) + if pilot.target == 8 { 6.0 } else { 12.0 },
                );
            let waypoint = match pilot.stage {
                Stage::Ascend => v(self.position.x, 42.0, self.position.z),
                Stage::Cruise => v(approach.x, 42.0, approach.z),
                Stage::Approach => approach,
            };
            let delta = waypoint - self.position;
            let distance = length(delta);
            desired = unit(delta) * (distance * 1.8).min(65.0);
            if distance < 2.5 {
                match pilot.stage {
                    Stage::Ascend => pilot.stage = Stage::Cruise,
                    Stage::Cruise => pilot.stage = Stage::Approach,
                    Stage::Approach => {
                        self.autopilot = None;
                        self.velocity = v(0.0, 0.0, 0.0);
                        desired = v(0.0, 0.0, 0.0);
                        self.notify(format!(
                            "Llegada a {}. Control manual listo.",
                            destination_name(pilot.target)
                        ));
                    }
                }
            }
            if self.autopilot.is_some() {
                self.autopilot = Some(pilot);
            }
            let gaze = if pilot.stage == Stage::Approach {
                target - self.position
            } else {
                delta
            };
            if length(gaze) > 0.1 {
                self.yaw = gaze.x.atan2(gaze.z);
                self.pitch = (gaze.y / length(gaze)).asin().clamp(-1.15, 1.15);
            }
        }
        if input.brake {
            desired = v(0.0, 0.0, 0.0);
        }
        let response = if input.brake { 9.0 } else { 3.0 };
        self.velocity = self.velocity + (desired - self.velocity) * (1.0 - (-response * dt).exp());
        self.position += self.velocity * dt;
        self.thrust = (length(self.velocity) / 60.0).clamp(0.12, 1.0);
        self.separate(v(0.0, 0.0, 0.0), 13.5);
        for (i, planet) in PLANETS.iter().enumerate() {
            let center = planet.position(self.time);
            self.separate(center, planet.radius + 1.5);
            if !self.visited[i] && length(self.position - center) < planet.radius + 19.0 {
                self.visited[i] = true;
                self.notify(format!(
                    "{} registrado / {}/8 mundos explorados",
                    planet.name,
                    self.visited.iter().filter(|&&b| b).count()
                ));
            }
        }
        let moon = moon_position(self.time);
        self.separate(moon, 2.8);
        if length(self.position) > 390.0 {
            self.position = unit(self.position) * 390.0;
            self.velocity = v(0.0, 0.0, 0.0);
            self.notify("Limite de navegacion. Regresa al sistema.".into());
        }
    }
    fn separate(&mut self, center: Vector3, radius: f32) {
        let offset = self.position - center;
        if length(offset) < radius {
            let normal = unit(offset);
            self.position = center + normal * radius;
            let inward = dot(self.velocity, normal);
            if inward < 0.0 {
                self.velocity -= normal * inward;
            }
        }
    }
}
pub fn moon_position(time: f32) -> Vector3 {
    PLANETS[2].position(time)
        + v(
            (time * 0.12 + 1.8).sin() * 10.0,
            1.2,
            (time * 0.12 + 1.8).cos() * 10.0,
        )
}

pub fn destination_name(index: usize) -> &'static str {
    if index == 8 {
        "LUNA"
    } else {
        PLANETS[index].name
    }
}
pub fn destination_position(index: usize, time: f32) -> Vector3 {
    if index == 8 {
        moon_position(time)
    } else {
        PLANETS[index].position(time)
    }
}
pub fn destination_radius(index: usize) -> f32 {
    if index == 8 {
        1.25
    } else {
        PLANETS[index].radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rescued_mechanic_improves_turbo_and_upgrade_survives_repositioning() {
        let run = |shield| {
            let mut w = World {
                position: v(150.0, 180.0, 0.0),
                thermal_shield: shield,
                ..Default::default()
            };
            for _ in 0..120 {
                w.update(
                    Input {
                        thrust: 1.0,
                        boost: true,
                        ..Default::default()
                    },
                    1.0 / 60.0,
                );
            }
            w
        };
        let normal = run(false);
        let mut upgraded = run(true);
        assert!(length(upgraded.velocity) > length(normal.velocity) * 1.14);
        upgraded.reset();
        assert!(upgraded.thermal_shield);
    }
    #[test]
    fn eight_orbits_are_ordered_and_non_overlapping() {
        for pair in PLANETS.windows(2) {
            assert!(pair[1].orbit - pair[0].orbit > pair[0].radius + pair[1].radius);
        }
        for p in PLANETS {
            assert!((length(p.position(150.0)) - p.orbit).abs() < 0.001);
        }
    }
    #[test]
    fn collisions_keep_ship_outside_bodies() {
        for (i, p) in PLANETS.iter().enumerate() {
            let mut world = World {
                position: p.position(0.0),
                ..Default::default()
            };
            world.update(Input::default(), 1.0 / 60.0);
            assert!(length(world.position - PLANETS[i].position(world.time)) >= p.radius + 1.49);
        }
        let mut world = World {
            position: v(0.0, 0.0, 0.0),
            ..Default::default()
        };
        world.update(Input::default(), 0.01);
        assert!(length(world.position) >= 13.49);
    }
    #[test]
    fn autopilot_reaches_every_planet() {
        for (target, planet) in PLANETS.iter().enumerate() {
            let mut world = World::default();
            world.travel(target);
            for _ in 0..9000 {
                world.update(Input::default(), 1.0 / 60.0);
                if world.autopilot.is_none() {
                    break;
                }
            }
            assert!(world.autopilot.is_none(), "No llega a {}", planet.name);
            assert!(world.visited[target]);
            assert!(world.target_distance() < 19.0);
        }
    }
    #[test]
    fn autopilot_reaches_moon_with_landing_clearance() {
        let mut world = World::default();
        world.travel(8);
        for _ in 0..9000 {
            world.update(Input::default(), 1.0 / 60.0);
            if world.autopilot.is_none() {
                break;
            }
        }
        assert!(world.autopilot.is_none());
        assert!(crate::expedition::Expedition::default().can_land(&world));
    }
    #[test]
    fn manual_input_cancels_autopilot_and_pause_freezes_simulation() {
        let mut world = World::default();
        world.travel(5);
        world.update(
            Input {
                thrust: 1.0,
                ..Default::default()
            },
            0.02,
        );
        assert!(world.autopilot.is_none());
        world.paused = true;
        let pos = world.position;
        let time = world.time;
        world.update(
            Input {
                thrust: 1.0,
                ..Default::default()
            },
            0.02,
        );
        assert_eq!(pos, world.position);
        assert_eq!(time, world.time);
    }
    #[test]
    fn motion_is_approximately_frame_rate_independent() {
        let run = |steps, dt| {
            let mut w = World {
                position: v(100.0, 55.0, 0.0),
                ..Default::default()
            };
            for _ in 0..steps {
                w.update(
                    Input {
                        thrust: 1.0,
                        ..Default::default()
                    },
                    dt,
                );
            }
            w.position
        };
        assert!(length(run(120, 1.0 / 60.0) - run(240, 1.0 / 120.0)) < 0.2);
    }
}
