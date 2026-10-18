use crate::{astronaut::Astronaut, simulation::*};
use raylib::prelude::*;

// Rig procedural: todas las piezas se expresan en el espacio local del personaje.
// Los ojos rojos, flancos azules y discos naranjas identifican la rana de arbol.
#[derive(Clone, Copy, PartialEq)]
pub enum Species {
    Tree,
    Tomato,
}
#[derive(Default)]
pub struct Pose {
    pub talking: bool,
    pub high_five: f32,
}

const UPPER_ARM: f32 = 0.43;
const FOREARM: f32 = 0.44;
const PALM_THICKNESS: f32 = 0.09;

fn body_width(species: Species) -> f32 {
    if species == Species::Tomato {
        1.12
    } else {
        1.0
    }
}

fn world_point(a: &Astronaut, species: Species, bob: f32, p: Vector3) -> Vector3 {
    let (s, c) = a.yaw.sin_cos();
    let p = v(p.x * body_width(species), p.y, p.z);
    a.position + v(0.0, bob, 0.0) + v(c * p.x + s * p.z, p.y, -s * p.x + c * p.z)
}

struct Arm {
    shoulder: Vector3,
    elbow: Vector3,
    hand: Vector3,
}

// Resolver en metros, antes de deshacer la anchura del cuerpo, conserva ambos
// huesos incluso en Brasa. La mezcla mueve la muneca; nunca estira los huesos.
fn arm_pose(
    species: Species,
    side: f32,
    body_y: f32,
    airborne: f32,
    step: f32,
    salute: f32,
) -> Arm {
    let width = body_width(species);
    let shoulder = v(side * 0.62 * width, body_y + 0.30, 0.0);
    let resting = v(
        side * (0.88 + airborne) * width,
        body_y - 0.39 + airborne * 1.9,
        0.20 + step * 0.22,
    );
    let raised = v(side * 1.11, body_y + 0.90, 0.20);
    let target = resting + (raised - resting) * salute;
    let delta = target - shoulder;
    let distance = length(delta).clamp(0.02, UPPER_ARM + FOREARM - 0.001);
    let axis = unit(delta);
    let hand = shoulder + axis * distance;
    let along =
        (UPPER_ARM * UPPER_ARM - FOREARM * FOREARM + distance * distance) / (2.0 * distance);
    let height = (UPPER_ARM * UPPER_ARM - along * along).max(0.0).sqrt();
    let pole = v(side, -0.18, -0.60);
    let bend = unit(pole - axis * dot(pole, axis));
    let elbow = shoulder + axis * along + bend * height;
    let unscale = |p: Vector3| v(p.x / width, p.y, p.z);
    Arm {
        shoulder: unscale(shoulder),
        elbow: unscale(elbow),
        hand: unscale(hand),
    }
}

fn hand_offset(offset: Vector3, side: f32, salute: f32) -> Vector3 {
    let (sx, cx) = (-std::f32::consts::FRAC_PI_2 * salute).sin_cos();
    let (sy, cy) = (-side * std::f32::consts::FRAC_PI_2 * salute).sin_cos();
    let p = v(
        offset.x,
        cx * offset.y - sx * offset.z,
        sx * offset.y + cx * offset.z,
    );
    v(cy * p.x + sy * p.z, p.y, -sy * p.x + cy * p.z)
}

fn hand_rotation(yaw: f32, side: f32, salute: f32) -> (Vector3, f32) {
    let (sy, cy) = ((yaw - side * std::f32::consts::FRAC_PI_2 * salute) * 0.5).sin_cos();
    let (sx, cx) = (-std::f32::consts::FRAC_PI_2 * salute * 0.5).sin_cos();
    let q = v(cy * sx, sy * cx, -sy * sx);
    let n = length(q);
    let axis = if n > 0.000001 {
        q * (1.0 / n)
    } else {
        v(0.0, 1.0, 0.0)
    };
    (axis, (2.0 * n.atan2(cy * cx)).to_degrees())
}

pub fn draw<D: RaylibDraw3D>(d: &mut D, mesh: &Model, a: &Astronaut) {
    draw_actor(d, mesh, a, Species::Tree, &Pose::default());
}
pub fn draw_actor<D: RaylibDraw3D>(
    d: &mut D,
    mesh: &Model,
    a: &Astronaut,
    species: Species,
    pose: &Pose,
) {
    let tomato = species == Species::Tomato;
    let suit = if tomato {
        Color::new(223, 166, 92, 255)
    } else {
        Color::new(225, 234, 222, 255)
    };
    let dark = Color::new(24, 55, 69, 255);
    let green = if tomato {
        Color::new(220, 66, 37, 255)
    } else {
        Color::new(76, 191, 55, 255)
    };
    let lime = if tomato {
        Color::new(255, 163, 85, 255)
    } else {
        Color::new(143, 216, 65, 255)
    };
    let orange = Color::new(250, 133, 48, 255);
    let blue = Color::new(32, 122, 181, 255);
    let motion = length(v(a.velocity.x, 0.0, a.velocity.z));
    let walk = if a.grounded {
        (motion / 4.0).min(1.0)
    } else {
        0.0
    };
    let sway = a.animation.sin() * walk;
    let crouch = if a.charging {
        0.12 + a.charge * 0.38
    } else {
        a.landing * 0.22
    };
    let bob = if a.grounded {
        a.animation.cos().abs() * walk * 0.07 + (a.time * 2.0).sin() * 0.025
    } else {
        0.0
    } * (1.0 - pose.high_five);
    let local = |p: Vector3| world_point(a, species, bob, p);
    let ball = |d: &mut D, center: Vector3, size: Vector3, color: Color| {
        d.draw_model_ex(
            mesh,
            local(center),
            v(0.0, 1.0, 0.0),
            a.yaw.to_degrees(),
            size,
            color,
        );
    };
    let limb = |d: &mut D, from: Vector3, to: Vector3, radius: f32, color: Color| {
        d.draw_cylinder_ex(local(from), local(to), radius, radius * 0.85, 8, color);
    };
    // Sombra de contacto en el suelo; se separa de los pies durante el salto.
    let ground = if a.grounded {
        a.position.y + 0.015
    } else {
        crate::astronaut::floor(a.position) + 0.015
    };
    d.draw_cylinder(
        v(a.position.x, ground, a.position.z),
        0.8,
        0.8,
        0.012,
        16,
        Color::new(7, 14, 26, 105),
    );
    let body_y = 1.18 - crouch;
    ball(
        d,
        v(0.0, body_y, -0.10),
        v(0.63, 0.74 - crouch * 0.25, 0.47),
        suit,
    );
    ball(
        d,
        v(0.0, body_y, 0.26),
        v(0.40, 0.50, 0.24),
        Color::new(241, 234, 186, 255),
    );
    ball(d, v(0.0, body_y - 0.38, 0.0), v(0.63, 0.14, 0.45), dark);
    // Mochila y dos cilindros de oxigeno.
    ball(d, v(0.0, body_y, -0.53), v(0.46, 0.51, 0.22), dark);
    for side in [-1.0, 1.0] {
        limb(
            d,
            v(side * 0.32, body_y - 0.30, -0.66),
            v(side * 0.32, body_y + 0.35, -0.66),
            0.16,
            Color::new(55, 191, 182, 255),
        );
        ball(
            d,
            v(side * 0.32, body_y + 0.38, -0.66),
            v(0.17, 0.12, 0.17),
            suit,
        );
    }
    // Panel del traje, cuello y hombros.
    ball(d, v(0.0, body_y + 0.18, 0.46), v(0.25, 0.20, 0.05), dark);
    for i in 0..3 {
        ball(
            d,
            v(-0.12 + i as f32 * 0.12, body_y + 0.18, 0.51),
            v(0.035, 0.07, 0.02),
            if i == 2 {
                orange
            } else {
                Color::new(93, 239, 201, 255)
            },
        );
    }
    ball(d, v(0.0, 1.72 - crouch, 0.01), v(0.67, 0.15, 0.50), dark);
    for side in [-1.0, 1.0] {
        let step = sway * side;
        let knee = v(side * 0.52, 0.52 - crouch * 0.50, 0.05 + step * 0.25);
        let foot = v(side * 0.52, 0.13, 0.16 - step * 0.31);
        limb(d, v(side * 0.34, body_y - 0.40, -0.03), knee, 0.23, suit);
        limb(d, knee, foot, 0.17, blue);
        ball(d, knee, v(0.25, 0.22, 0.23), green);
        ball(d, foot, v(0.30, 0.14, 0.38), dark);
        for toe in -1..=1 {
            let x = side * 0.52 + toe as f32 * 0.16;
            limb(
                d,
                v(x, 0.12, foot.z + 0.12),
                v(x + toe as f32 * 0.05, 0.11, foot.z + 0.49),
                0.062,
                orange,
            );
            ball(
                d,
                v(x + toe as f32 * 0.05, 0.11, foot.z + 0.51),
                v(0.105, 0.065, 0.12),
                orange,
            );
        }
        let airborne = if !a.grounded { 0.33 } else { 0.0 };
        let salute = if side == if tomato { -1.0 } else { 1.0 } {
            pose.high_five
        } else {
            0.0
        };
        let Arm {
            shoulder,
            elbow,
            hand,
        } = arm_pose(species, side, body_y, airborne, step, salute);
        ball(d, shoulder, v(0.22, 0.22, 0.23), suit);
        limb(d, shoulder, elbow, 0.13, green);
        ball(d, elbow, v(0.17, 0.17, 0.17), blue);
        limb(d, elbow, hand, 0.11, green);
        let (axis, angle) = hand_rotation(a.yaw, side, salute);
        d.draw_model_ex(
            mesh,
            local(hand),
            axis,
            angle,
            v(0.20, PALM_THICKNESS, 0.18),
            orange,
        );
        for finger in -1..=1 {
            let offset = hand_offset(v(finger as f32 * 0.10, -0.03, 0.24), side, salute);
            let end = hand + v(offset.x / body_width(species), offset.y, offset.z);
            limb(d, hand, end, 0.035, orange);
            d.draw_model_ex(mesh, local(end), axis, angle, v(0.07, 0.045, 0.07), orange);
        }
        // Franja azul de los costados.
        ball(
            d,
            v(side * 0.60, body_y + 0.02, 0.02),
            v(0.04, 0.30, 0.28),
            blue,
        );
    }
    let head = 2.15 - crouch;
    ball(
        d,
        v(0.0, head, 0.03),
        v(0.78, if tomato { 0.60 } else { 0.43 }, 0.55),
        green,
    );
    ball(d, v(0.0, head - 0.16, 0.26), v(0.64, 0.20, 0.39), lime);
    // Boca sonriente: arco de segmentos sobre el hocico.
    for i in 0..8 {
        let x0 = -0.38 + i as f32 * 0.095;
        let x1 = x0 + 0.095;
        let y0 = head - 0.18 + x0 * x0 * 0.35;
        let y1 = head - 0.18 + x1 * x1 * 0.35;
        d.draw_line3D(local(v(x0, y0, 0.59)), local(v(x1, y1, 0.59)), dark);
    }
    if pose.talking {
        ball(
            d,
            v(0.0, head - 0.16, 0.607),
            v(0.22, 0.025 + (a.time * 16.0).sin().abs() * 0.10, 0.035),
            dark,
        );
    }
    if tomato {
        // Llave de mecanica en el cinturon.
        limb(
            d,
            v(-0.62, 0.8, 0.31),
            v(-0.62, 1.42, 0.31),
            0.06,
            Color::LIGHTGRAY,
        );
        ball(
            d,
            v(-0.62, 1.45, 0.31),
            v(0.16, 0.13, 0.07),
            Color::LIGHTGRAY,
        );
    }
    let blink = (a.time % 4.7) > 4.52;
    for side in [-1.0, 1.0] {
        let eye = v(side * 0.46, head + 0.31, 0.27);
        ball(d, eye, v(0.32, 0.34, 0.30), green);
        ball(
            d,
            eye + v(0.0, 0.0, 0.23),
            v(0.25, if blink { 0.025 } else { 0.27 }, 0.10),
            if blink {
                green
            } else {
                if tomato {
                    Color::new(249, 202, 65, 255)
                } else {
                    Color::new(229, 60, 35, 255)
                }
            },
        );
        if !blink {
            ball(
                d,
                eye + v(0.0, 0.0, 0.31),
                v(
                    if tomato { 0.17 } else { 0.055 },
                    if tomato { 0.075 } else { 0.22 },
                    0.035,
                ),
                Color::new(12, 25, 24, 255),
            );
            ball(
                d,
                eye + v(-0.065, 0.09, 0.335),
                v(0.048, 0.055, 0.025),
                Color::WHITE,
            );
        }
        ball(
            d,
            v(side * 0.25, head + 0.035, 0.55),
            v(0.045, 0.025, 0.025),
            dark,
        );
    }
    // Burbuja de casco dibujada al final para conservar la cara visible.
    let helmet = local(v(0.0, head + 0.06, 0.02));
    d.draw_sphere_ex(helmet, 1.03, 14, 20, Color::new(150, 228, 245, 17));
    d.draw_circle3D(
        helmet,
        1.035,
        v(0.0, 1.0, 0.0),
        a.yaw.to_degrees(),
        Color::new(142, 224, 235, 110),
    );
    d.draw_circle3D(
        helmet,
        1.035,
        v(0.0, 1.0, 0.0),
        a.yaw.to_degrees() + 90.0,
        Color::new(142, 224, 235, 70),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arm_bones_and_fingers_keep_their_lengths_through_every_blend() {
        for species in [Species::Tree, Species::Tomato] {
            for side in [-1.0, 1.0] {
                for yaw in [0.0, 0.73, std::f32::consts::PI, 4.8] {
                    let mut a = Astronaut::default();
                    a.yaw = yaw;
                    a.position = v(12.0, 0.55, -8.0);
                    for frame in 0..=120 {
                        let blend = frame as f32 / 120.0;
                        for (crouch, airborne, step) in
                            [(0.0, 0.0, 0.0), (0.30, 0.0, 1.0), (0.0, 0.33, -1.0)]
                        {
                            let arm = arm_pose(species, side, 1.18 - crouch, airborne, step, blend);
                            let shoulder = world_point(&a, species, 0.0, arm.shoulder);
                            let elbow = world_point(&a, species, 0.0, arm.elbow);
                            let hand = world_point(&a, species, 0.0, arm.hand);
                            assert!((length(elbow - shoulder) - UPPER_ARM).abs() < 0.00001);
                            assert!((length(hand - elbow) - FOREARM).abs() < 0.00001);
                            for finger in -1..=1 {
                                let rest = v(finger as f32 * 0.10, -0.03, 0.24);
                                let offset = hand_offset(rest, side, blend);
                                let end = arm.hand
                                    + v(offset.x / body_width(species), offset.y, offset.z);
                                let end = world_point(&a, species, 0.0, end);
                                assert!((length(end - hand) - length(rest)).abs() < 0.00001);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn both_palms_meet_at_the_effect_point_without_colliding_helmets() {
        use crate::rescue::{HIGH_FIVE_CONTACT, HIGH_FIVE_POINT, REFUGE, Rescue, Status};
        let mut r = Rescue::default();
        let mut a = Astronaut::default();
        a.position = REFUGE + v(0.0, 0.0, 4.0);
        r.collectors = [true; 3];
        assert!(r.interact(&a));
        r.celebration = HIGH_FIVE_CONTACT;
        r.update(&mut a, 0.0);
        assert_eq!(r.status, Status::Celebration);
        let mut palms = Vec::new();
        for (actor, species, side) in [(&a, Species::Tree, 1.0), (&r.brasa, Species::Tomato, -1.0)]
        {
            let arm = arm_pose(species, side, 1.18, 0.0, 0.0, r.high_five());
            let hand = world_point(actor, species, 0.0, arm.hand);
            let normal = hand_offset(v(0.0, 1.0, 0.0), side, r.high_five());
            let palm = hand + normal * PALM_THICKNESS;
            assert!(length(palm - HIGH_FIVE_POINT) < 0.00001, "palm {palm:?}");
            palms.push(palm);
        }
        assert!(length(palms[0] - palms[1]) < 0.00001);
        assert!(length(a.position - r.brasa.position) > 2.0 * 1.035 + 0.20);
    }
}
