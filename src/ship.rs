use crate::simulation::*;
use raylib::prelude::*;

// Dos escalas de escena: planetas legibles en órbita y una nave habitable en tierra.
pub const ORBIT_SCALE: f32 = 0.035;
pub const SURFACE_SCALE: f32 = 2.2;
pub const GROUND_Y: f32 = 0.55 + 0.62 * SURFACE_SCALE;
pub const DOOR: Vector3 = Vector3 {
    x: 8.5,
    y: 0.55,
    z: 28.0,
};

// Centros de las caras 07_Motores_turquesa del OBJ, antes de escalar/rotar.
const NOZZLES: [Vector3; 2] = [
    Vector3 {
        x: -2.715,
        y: -0.075,
        z: -2.635,
    },
    Vector3 {
        x: 2.715,
        y: -0.075,
        z: -2.635,
    },
];

pub struct Pose {
    pub position: Vector3,
    pub scale: f32,
    pub axis: Vector3,
    pub angle: f32,
}
impl Pose {
    pub fn new(world: &World, scale: f32) -> Self {
        // Yaw alrededor de Y y pitch alrededor del X local. Quaternion compuesto:
        // evita el giro de alabeo accidental de la antigua rotacion +Z -> forward.
        let (sy, cy) = (world.yaw * 0.5).sin_cos();
        let (sp, cp) = (world.pitch * 0.5).sin_cos();
        let q = v(-sp * cy, cp * sy, sp * sy);
        let n = length(q);
        Self {
            position: world.position,
            scale,
            axis: if n > 0.000001 {
                q * (1.0 / n)
            } else {
                v(0.0, 1.0, 0.0)
            },
            angle: 2.0 * n.atan2(cp * cy),
        }
    }
    pub fn rotate(&self, point: Vector3) -> Vector3 {
        let (s, c) = self.angle.sin_cos();
        point * c + cross(self.axis, point) * s + self.axis * dot(self.axis, point) * (1.0 - c)
    }
    pub fn point(&self, point: Vector3) -> Vector3 {
        self.position + self.rotate(point) * self.scale
    }
}

pub fn draw<D: RaylibDraw3D>(d: &mut D, model: &Model, world: &World, scale: f32) {
    let pose = Pose::new(world, scale);
    d.draw_model_ex(
        model,
        pose.position,
        pose.axis,
        pose.angle.to_degrees(),
        v(scale, scale, scale),
        Color::WHITE,
    );
    // Nave, anclajes y direccion de cada estela usan exactamente la misma pose.
    for nozzle in NOZZLES {
        for i in (0..10).rev() {
            let t = i as f32 / 10.0;
            let local = nozzle + v(0.0, 0.0, -t * (1.0 + world.thrust * 7.0));
            d.draw_sphere_ex(
                pose.point(local),
                scale * 0.34 * (1.0 - t * 0.8),
                6,
                8,
                Color::new(80, 230, 215, (230.0 * (1.0 - t)) as u8),
            );
        }
    }
}

pub fn boarding_marker<D: RaylibDraw3D>(d: &mut D) {
    let center = crate::astronaut::SPAWN;
    let mint = Color::new(125, 238, 202, 255);
    d.draw_circle3D(
        center + v(0.0, 0.04, 0.0),
        1.2,
        v(1.0, 0.0, 0.0),
        90.0,
        mint,
    );
    for z in [-1.4, 1.4] {
        d.draw_cube(center + v(0.0, 0.08, z), 2.0, 0.1, 0.12, mint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_obj_fits_the_landing_collision_and_has_sensible_scene_scales() {
        let points: Vec<Vector3> = include_str!("../assets/RANA_56.obj")
            .lines()
            .filter_map(|line| line.strip_prefix("v "))
            .map(|line| {
                let xyz: Vec<f32> = line
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect();
                v(xyz[0], xyz[1], xyz[2])
            })
            .collect();
        let width = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max)
            - points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        assert!(
            PLANETS
                .iter()
                .all(|p| p.radius * 2.0 > width * ORBIT_SCALE * 15.0)
        );
        assert!(width * SURFACE_SCALE > 16.0);
        let block = crate::astronaut::SHIP_BLOCK;
        let pose = Pose::new(
            &World {
                position: v(0.0, GROUND_Y, 28.0),
                yaw: std::f32::consts::PI,
                ..Default::default()
            },
            SURFACE_SCALE,
        );
        for p in points {
            let p = pose.point(p);
            let relative = p - block.center;
            assert!(p.y >= 0.549);
            assert!(relative.x.abs() <= block.size.x * 0.5 + 0.001);
            assert!(relative.y.abs() <= block.size.y * 0.5 + 0.001);
            assert!(relative.z.abs() <= block.size.z * 0.5 + 0.001);
        }
        assert!(!block.contains(crate::astronaut::SPAWN, crate::astronaut::RADIUS));
    }
    #[test]
    fn nose_and_engines_share_orientation_at_all_headings() {
        for yaw in [
            0.0,
            0.7,
            std::f32::consts::PI,
            4.8,
            std::f32::consts::TAU - 0.01,
        ] {
            for pitch in [-1.15, -0.6, 0.0, 0.6, 1.15] {
                let world = World {
                    yaw,
                    pitch,
                    position: v(5.0, 7.0, -3.0),
                    ..Default::default()
                };
                for scale in [ORBIT_SCALE, SURFACE_SCALE] {
                    let pose = Pose::new(&world, scale);
                    assert!(length(pose.rotate(v(0.0, 0.0, 1.0)) - world.forward()) < 0.00001);
                    // Simetria, distancia al casco y direccion posterior al inclinar.
                    let a = pose.point(NOZZLES[0]);
                    let b = pose.point(NOZZLES[1]);
                    assert!((length(a - b) - 5.43 * scale).abs() < 0.00001);
                    for nozzle in NOZZLES {
                        let origin = pose.point(nozzle);
                        let tail = pose.point(nozzle + v(0.0, 0.0, -2.0));
                        assert!(dot(unit(tail - origin), world.forward()) < -0.9999);
                        assert!(
                            (length(origin - world.position) - length(nozzle) * scale).abs()
                                < 0.00001
                        );
                    }
                }
            }
        }
    }
}
