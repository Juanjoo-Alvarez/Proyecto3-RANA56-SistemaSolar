use crate::{
    astronaut::{Astronaut, WalkInput},
    comms::{self, Comms},
    frog::{self, Pose, Species},
    scene::{self, Assets},
    simulation::*,
    ui::{CREAM, GOLD, MINT, MUTED, Ui},
};
use raylib::prelude::*;

// Un puesto por planeta. Solo se ocupa al completar su rescate, nunca al visitarlo.
pub struct Cabin {
    pub captain: Astronaut,
    pub crew: [bool; 8],
    pub time: f32,
    angle: f32,
    elevation: f32,
}
impl Default for Cabin {
    fn default() -> Self {
        let mut captain = Astronaut::default();
        captain.position = v(0.0, 0.0, 4.0);
        Self {
            captain,
            crew: [false; 8],
            time: 0.0,
            angle: 0.30,
            elevation: 15.0,
        }
    }
}
pub fn station(index: usize) -> Vector3 {
    v(
        if index < 4 { -5.2 } else { 5.2 },
        0.0,
        -5.0 + (index % 4) as f32 * 4.0,
    )
}
impl Cabin {
    pub fn recruit(&mut self, planet: usize) {
        if let Some(slot) = self.crew.get_mut(planet) {
            *slot = true;
        }
    }
    pub fn count(&self) -> usize {
        self.crew.iter().filter(|&&present| present).count()
    }
    pub fn reset_position(&mut self) {
        self.captain.position = v(0.0, 0.0, 4.0);
        self.captain.velocity = Vector3::zero();
        self.angle = 0.30;
        self.elevation = 15.0;
    }
    pub fn near_brasa(&self) -> bool {
        self.crew[0] && length(self.captain.position - station(0)) < 3.0
    }
    pub fn talk(&self, comms: &mut Comms) {
        if self.near_brasa() {
            comms.begin_script(&[
                (comms::LUMA, "¡Qué bien verte en casa, Brasa! Este lugar ya se siente menos vacío."),
                (comms::BRASA, "Motores revisados, capitana. El escudo térmico está listo. Ahora nos toca traer a los demás."),
                (comms::LUMA, "Guardamos un lugar para cada uno. Ninguna rana se queda atrás."),
            ]);
        }
    }
    pub fn update(&mut self, input: WalkInput, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        self.time += dt;
        self.angle = (self.angle + input.look_x * dt * 0.65).clamp(-0.55, 0.55);
        self.elevation = (self.elevation - input.look_y * dt * 6.0).clamp(11.0, 19.0);
        let forward = v(-self.angle.sin(), 0.0, -self.angle.cos());
        let right = v(self.angle.cos(), 0.0, -self.angle.sin());
        let mut dir = forward * input.forward + right * input.horizontal;
        if length(dir) > 1.0 {
            dir = unit(dir);
        }
        let a = &mut self.captain;
        a.velocity +=
            (dir * if input.run { 6.0 } else { 4.0 } - a.velocity) * (1.0 - (-12.0 * dt).exp());
        let old = a.position;
        let desired = old + a.velocity * dt;
        let occupied = |p: Vector3| {
            (0..8).any(|i| {
                let seat = station(i);
                (p.x - seat.x).abs() < 1.65 && (p.z - seat.z).abs() < 1.65
            })
        };
        a.position.x = desired.x.clamp(-6.2, 6.2);
        if occupied(a.position) {
            a.position.x = old.x;
            a.velocity.x = 0.0;
        }
        a.position.z = desired.z.clamp(-7.0, 8.8);
        if occupied(a.position) {
            a.position.z = old.z;
            a.velocity.z = 0.0;
        }
        a.position.y = 0.0;
        let distance = length(a.position - old);
        if distance > 0.0001 {
            let yaw = a.velocity.x.atan2(a.velocity.z);
            let delta = (yaw - a.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            a.yaw += delta * (1.0 - (-12.0 * dt).exp());
        }
        a.time += dt;
        a.animation += distance * 2.6;
    }
    pub fn camera(&self) -> Camera3D {
        Camera3D::perspective(
            v(
                self.angle.sin() * 24.0,
                self.elevation,
                self.angle.cos() * 24.0,
            ),
            v(0.0, 1.0, 0.0),
            v(0.0, 1.0, 0.0),
            49.0,
        )
    }
}

pub fn draw(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    cabin: &Cabin,
    camera: Camera3D,
    world: &World,
    ui: &Ui,
    comms: &Comms,
) {
    let (w, h) = (d.get_screen_width(), d.get_screen_height());
    d.clear_background(Color::new(5, 12, 24, 255));
    d.draw_rectangle_gradient_v(
        0,
        0,
        w,
        h,
        Color::new(7, 16, 32, 255),
        Color::new(22, 40, 49, 255),
    );
    assets.prepare(camera, cabin.time);
    {
        let mut s = d.begin_mode3D(camera);
        for i in 0..240 {
            let x = ((i * 47) % 220) as f32 - 110.0;
            let y = ((i * 31) % 100) as f32 - 20.0;
            s.draw_sphere_ex(v(x, y, -100.0), 0.06, 4, 4, Color::new(146, 180, 205, 255));
        }
        assets.kind(if world.selected == 8 {
            10
        } else {
            world.selected as i32 + 1
        });
        s.draw_model(&assets.sphere, v(-1.5, 7.0, -34.0), 7.0, Color::WHITE);
        assets.kind(11);
        let hull = Color::new(52, 75, 89, 255);
        let floor = Color::new(29, 48, 61, 255);
        let trim = Color::new(132, 180, 174, 255);
        s.draw_cube(v(0.0, -0.3, 0.0), 14.5, 0.6, 21.0, hull);
        for x in -3..=3 {
            for z in -5..=4 {
                s.draw_cube(
                    v(x as f32 * 2.0, -0.035, z as f32 * 2.0 + 1.0),
                    1.96,
                    0.08,
                    1.96,
                    floor,
                );
            }
        }
        for side in [-1.0, 1.0] {
            s.draw_cube(v(side * 7.1, 1.25, 0.0), 0.3, 2.5, 20.5, hull);
            s.draw_cube(v(side * 7.1, 2.55, 0.0), 0.32, 0.08, 20.5, trim);
            s.draw_cube(v(side * 3.1, 0.04, 0.0), 0.06, 0.04, 18.0, MINT);
            s.draw_cube(v(side * 6.2, 3.2, -10.0), 2.0, 6.4, 0.4, hull);
            s.draw_cube(v(side * 5.2, 3.2, -10.0), 0.14, 6.4, 0.5, MINT);
            // Jardines hidropónicos junto al puente.
            s.draw_cube(v(side * 4.0, 0.65, -8.5), 1.4, 1.3, 1.4, trim);
            for j in 0..4 {
                let p = v(
                    side * 4.0 + (j as f32 * 2.3).sin() * 0.35,
                    1.4,
                    -8.5 + (j as f32).cos() * 0.3,
                );
                s.draw_cylinder(p, 0.03, 0.04, 1.1, 6, Color::new(82, 156, 89, 255));
                s.draw_sphere_ex(
                    p + v(0.15, 0.8, 0.0),
                    0.25,
                    6,
                    8,
                    Color::new(114, 200, 115, 255),
                );
            }
        }
        s.draw_cube(v(0.0, 0.65, -10.0), 10.5, 1.3, 0.4, hull);
        s.draw_cube(v(0.0, 6.3, -10.0), 10.5, 0.35, 0.4, trim);
        // Ventanal abierto al espacio y consola de navegación.
        s.draw_cube(v(0.0, 0.7, -8.4), 4.8, 1.4, 1.4, hull);
        s.draw_cube(v(0.0, 1.45, -8.4), 4.8, 0.12, 1.4, trim);
        s.draw_cube(
            v(0.0, 2.1, -8.8),
            3.6,
            1.25,
            0.16,
            Color::new(15, 46, 65, 255),
        );
        for i in -2..=2 {
            s.draw_cube(v(i as f32 * 0.55, 2.1, -8.69), 0.3, 0.12, 0.04, MINT);
        }
        for i in 0..8 {
            let p = station(i);
            let occupied = cabin.crew[i];
            let color = if occupied {
                MINT
            } else {
                Color::new(91, 119, 133, 255)
            };
            s.draw_cube(
                p + v(0.0, 0.14, 0.0),
                2.4,
                0.28,
                2.8,
                Color::new(39, 67, 77, 255),
            );
            s.draw_cube(
                p + v(if i < 4 { -1.3 } else { 1.3 }, 1.6, 0.0),
                0.14,
                2.8,
                2.8,
                hull,
            );
            s.draw_cube(p + v(0.0, 0.3, 1.2), 2.2, 0.06, 0.08, color);
            if !occupied {
                s.draw_cube(
                    p + v(0.0, 0.7, 0.0),
                    1.55,
                    0.7,
                    1.6,
                    Color::new(51, 76, 91, 255),
                );
                s.draw_cube(
                    p + v(0.0, 1.35, -0.65),
                    1.55,
                    0.8,
                    0.2,
                    Color::new(63, 95, 108, 255),
                );
            }
        }
        if cabin.crew[0] {
            let mut brasa = Astronaut::default();
            brasa.position = station(0) + v(0.0, 0.28, 0.0);
            brasa.yaw = std::f32::consts::FRAC_PI_2;
            brasa.time = comms.time;
            let talking = comms.active.as_ref().is_some_and(|a| {
                matches!(a.message.speaker.portrait, comms::Portrait::Brasa) && !a.complete()
            });
            frog::draw_actor(
                &mut s,
                &assets.frog_mesh,
                &brasa,
                Species::Tomato,
                &Pose {
                    talking,
                    high_five: 0.0,
                },
            );
        }
        let mut luma = Astronaut::default();
        luma.position = cabin.captain.position;
        luma.yaw = cabin.captain.yaw;
        luma.animation = cabin.captain.animation;
        luma.time = comms.time;
        if !comms.blocking() {
            luma.velocity = cabin.captain.velocity;
        }
        let talking = comms.active.as_ref().is_some_and(|a| {
            matches!(a.message.speaker.portrait, comms::Portrait::Luma) && !a.complete()
        });
        frog::draw_actor(
            &mut s,
            &assets.frog_mesh,
            &luma,
            Species::Tree,
            &Pose {
                talking,
                high_five: 0.0,
            },
        );
    }
    for (i, planet) in PLANETS.iter().enumerate() {
        if let Some(p) = scene::project(station(i) + v(0.0, 0.4, 1.5), camera, w, h) {
            let label = if cabin.crew[i] {
                "Brasa · Mecánica"
            } else {
                planet.name
            };
            ui.text(
                d,
                label,
                p.x - 65.0,
                p.y,
                14.0,
                if cabin.crew[i] { MINT } else { MUTED },
            );
        }
    }
    if let Some(p) = scene::project(v(0.0, 5.6, -9.7), camera, w, h) {
        ui.text(d, "RANA / 56", p.x - 61.0, p.y, 22.0, CREAM);
        ui.text(
            d,
            "UN LUGAR PARA VOLVER",
            p.x - 88.0,
            p.y + 26.0,
            13.0,
            GOLD,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cabin_movement_stays_inside_and_does_not_cross_crew_stations() {
        let mut c = Cabin::default();
        for forward in [-1.0, 0.0, 1.0] {
            for horizontal in [-1.0, 0.0, 1.0] {
                c.reset_position();
                for _ in 0..600 {
                    c.update(
                        WalkInput {
                            forward,
                            horizontal,
                            run: true,
                            ..Default::default()
                        },
                        1.0 / 60.0,
                    );
                    let p = c.captain.position;
                    assert!(p.x.abs() <= 6.2 && (-7.0..=8.8).contains(&p.z));
                    assert_eq!(p.y, 0.0);
                    assert!(!(0..8).any(|i| {
                        let s = station(i);
                        (p.x - s.x).abs() < 1.65 && (p.z - s.z).abs() < 1.65
                    }));
                }
            }
        }
    }
    #[test]
    fn only_rescued_crew_occupy_slots_and_recruitment_is_idempotent() {
        let mut c = Cabin::default();
        assert_eq!(c.count(), 0);
        c.recruit(0);
        c.recruit(0);
        c.recruit(8);
        assert_eq!(c.count(), 1);
        assert!(!c.near_brasa());
        c.captain.position = v(-2.5, 0.0, -5.0);
        assert!(c.near_brasa());
        let mut comms = Comms::default();
        c.talk(&mut comms);
        assert!(comms.blocking());
        c.reset_position();
        assert_eq!(c.count(), 1);
    }
}
