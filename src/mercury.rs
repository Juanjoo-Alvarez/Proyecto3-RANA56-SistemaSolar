use crate::{
    expedition::{Expedition, Phase},
    frog::{self, Pose, Species},
    rescue::*,
    scene::Assets,
    simulation::*,
};
use raylib::prelude::*;

pub fn draw(d: &mut RaylibDrawHandle, assets: &mut Assets, e: &Expedition, camera: Camera3D) {
    d.clear_background(Color::new(14, 10, 18, 255));
    d.draw_rectangle_gradient_v(
        0,
        0,
        d.get_screen_width(),
        d.get_screen_height(),
        Color::new(7, 8, 18, 255),
        Color::new(91, 44, 34, 255),
    );
    assets.prepare(camera, e.local.time);
    let mut s = d.begin_mode3D(camera);
    assets.kind(0);
    s.draw_model(&assets.sphere, v(-82.0, 25.0, -180.0), 36.0, Color::WHITE);
    for x in -16i32..16 {
        for z in -16i32..16 {
            let shade = ((x * 17 + z * 31).rem_euclid(6) * 5) as u8;
            s.draw_cube(
                v(x as f32 * 8.0, -0.25, z as f32 * 8.0),
                8.0,
                0.5,
                8.0,
                Color::new(83 + shade, 65 + shade / 2, 59 + shade / 3, 255),
            );
            if x.abs() > 7 || z.abs() > 7 {
                s.draw_sphere_ex(
                    v(x as f32 * 9.0, 0.0, z as f32 * 9.0),
                    2.0 + shade as f32 * 0.2,
                    4,
                    6,
                    Color::new(73, 56, 54, 255),
                );
            }
        }
    }
    let steel = Color::new(44, 65, 80, 255);
    let mint = Color::new(115, 235, 201, 255);
    let gold = Color::new(255, 186, 81, 255);
    // Los rectangulos de sombra corresponden a las areas de enfriamiento.
    for p in SHELTERS {
        s.draw_cube(
            p + v(0.0, 0.018, 0.0),
            11.0,
            0.025,
            11.0,
            Color::new(34, 42, 53, 255),
        );
        s.draw_cube(p + v(0.0, 6.5, 0.0), 11.0, 0.3, 11.0, steel);
        for i in -2..=2 {
            s.draw_cube(
                p + v(i as f32 * 2.0, 6.68, 0.0),
                1.85,
                0.06,
                10.5,
                Color::new(29, 68, 91, 255),
            );
            s.draw_line3D(
                p + v(i as f32 * 2.0, 6.74, -5.25),
                p + v(i as f32 * 2.0, 6.74, 5.25),
                gold,
            );
        }
        for x in [-5.0, 5.0] {
            for z in [-5.0, 5.0] {
                s.draw_cylinder_ex(
                    p + v(x, 0.0, z),
                    p + v(x, 6.5, z),
                    0.10,
                    0.10,
                    6,
                    Color::new(128, 139, 147, 255),
                );
            }
        }
        for z in [-5.4, 5.4] {
            s.draw_cube(p + v(0.0, 0.05, z), 10.8, 0.04, 0.12, mint);
        }
    }
    for (i, p) in COLLECTORS.iter().enumerate() {
        let c = if e.rescue.collectors[i] { mint } else { gold };
        s.draw_cylinder(*p + v(0.0, 0.0, -2.8), 0.6, 0.85, 2.1, 8, steel);
        s.draw_cube(*p + v(0.0, 1.8, -2.42), 1.15, 0.65, 0.13, c);
        s.draw_sphere_ex(*p + v(0.0, 3.3, -2.8), 0.28, 6, 8, c);
        s.draw_circle3D(*p + v(0.0, 0.06, 0.0), 2.7, v(1.0, 0.0, 0.0), 90.0, c);
        // Conduccion al refugio, apagada tras la interaccion.
        s.draw_line3D(*p + v(0.0, 0.07, -2.8), REFUGE + v(0.0, 0.07, 0.0), c);
    }
    for b in crate::astronaut::MERCURY_BLOCKS {
        s.draw_cube_v(b.center, b.size, steel);
    }
    s.draw_cube(
        v(8.0, 6.6, -33.0),
        15.0,
        0.45,
        10.0,
        Color::new(83, 107, 119, 255),
    );
    if !e.rescue.unlocked() {
        s.draw_cube(
            v(8.0, 2.4, -28.4),
            12.0,
            4.8,
            0.12,
            Color::new(184, 68, 39, 220),
        );
        for i in 0..6 {
            s.draw_cube(v(3.0 + i as f32 * 2.0, 2.5, -28.25), 0.08, 4.7, 0.08, gold);
        }
    } else {
        s.draw_cube(v(8.0, 6.1, -28.2), 12.0, 0.15, 0.1, mint);
    }
    s.draw_cylinder(v(0.0, 0.0, 28.0), 9.0, 9.0, 0.55, 32, steel);
    for i in 0..48 {
        let a = i as f32 * std::f32::consts::TAU / 48.0;
        s.draw_cube(
            v(a.sin() * 8.3, 0.59, 28.0 + a.cos() * 8.3),
            0.3,
            0.06,
            0.3,
            mint,
        );
    }
    assets.kind(11);
    crate::ship::draw(&mut s, &assets.ship, &e.local, 0.55);
    if e.astronaut_visible() {
        frog::draw_actor(
            &mut s,
            &assets.frog_mesh,
            &e.astronaut,
            Species::Tree,
            &Pose {
                talking: false,
                high_five: e.rescue.high_five(),
            },
        );
    }
    if e.phase == Phase::Boarding || !matches!(e.rescue.status, Status::Aboard | Status::Complete) {
        frog::draw_actor(
            &mut s,
            &assets.frog_mesh,
            &e.rescue.brasa,
            Species::Tomato,
            &Pose {
                talking: false,
                high_five: e.rescue.high_five(),
            },
        );
    }
    if e.rescue.high_five() > 0.9 {
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::TAU / 8.0 + e.local.time;
            let center = HIGH_FIVE_POINT;
            s.draw_line3D(
                center + v(a.cos(), a.sin(), 0.0) * 0.35,
                center + v(a.cos(), a.sin(), 0.0) * 0.65,
                gold,
            );
        }
    }
}
