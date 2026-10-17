use crate::{
    expedition::*,
    scene::{self, ACCENT, Assets},
    simulation::*,
};
use raylib::prelude::*;
use std::f32::consts::TAU;

pub fn camera(e: &Expedition) -> Camera3D {
    if e.phase == Phase::Landing || e.phase == Phase::Launch {
        Camera3D::perspective(
            v(26.0, 18.0, 58.0),
            e.local.position + v(0.0, 1.0, 0.0),
            v(0.0, 1.0, 0.0),
            55.0,
        )
    } else if e.phase == Phase::OnFoot {
        e.astronaut.camera()
    } else if e.astronaut_visible() {
        Camera3D::perspective(
            v(19.0, 8.0, 42.0),
            e.astronaut.position + v(0.0, 1.5, 0.0),
            v(0.0, 1.0, 0.0),
            52.0,
        )
    } else {
        scene::flight_camera(&e.local)
    }
}
fn triangle<D: RaylibDraw3D>(d: &mut D, a: Vector3, b: Vector3, c: Vector3, color: Color) {
    d.draw_triangle3D(a, b, c, color);
    d.draw_triangle3D(c, b, a, color);
}
pub fn draw(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    e: &Expedition,
    camera: Camera3D,
    pad: bool,
) {
    let (w, h) = (d.get_screen_width(), d.get_screen_height());
    d.clear_background(Color::new(7, 10, 26, 255));
    d.draw_rectangle_gradient_v(
        0,
        0,
        w,
        h,
        Color::new(4, 8, 24, 255),
        Color::new(39, 42, 73, 255),
    );
    assets.prepare(camera, e.local.time);
    {
        let mut s = d.begin_mode3D(camera);
        for i in 0..400 {
            let a = i as f32 * 2.39996;
            let y = (i as f32 * 0.618).fract();
            let radial = (1.0 - y * y).sqrt();
            s.draw_point3D(
                camera.position + v(a.sin() * radial, y, a.cos() * radial) * 450.0,
                Color::new(157, 174, 200, 255),
            );
        }
        assets.kind(3);
        s.draw_model(&assets.sphere, v(-52.0, 42.0, -155.0), 20.0, Color::WHITE);
        // Mosaico lunar con caras de color plano; el area central es transitable.
        for x in -22i32..22 {
            for z in -22i32..22 {
                let height = |a: i32, b: i32| {
                    let edge = ((a.abs().max(b.abs()) as f32 - 7.0) * 0.25).clamp(0.0, 1.0);
                    ((a as f32 * 1.7).sin() * (b as f32 * 1.2).cos() + 1.0) * 4.0 * edge
                };
                let point = |a, b| v(a as f32 * 8.0, height(a, b), b as f32 * 8.0);
                let shade = ((x * 31 + z * 17).rem_euclid(5) * 4) as u8;
                let c = Color::new(62 + shade, 70 + shade, 88 + shade, 255);
                triangle(&mut s, point(x, z), point(x + 1, z), point(x + 1, z + 1), c);
                triangle(
                    &mut s,
                    point(x, z),
                    point(x + 1, z + 1),
                    point(x, z + 1),
                    Color::new(c.r + 3, c.g + 3, c.b + 3, 255),
                );
            }
        }
        // Crestas y crateres fuera del volumen de vuelo.
        for i in 0..24 {
            let a = i as f32 / 24.0 * TAU;
            let center = v(a.sin() * 110.0, 0.0, a.cos() * 110.0);
            s.draw_cylinder(
                center,
                0.0,
                14.0,
                14.0 + (i % 5) as f32 * 5.0,
                5,
                Color::new(46, 54, 73, 255),
            );
        }
        for i in 0..90 {
            let x = ((i * 47) % 260) as f32 - 130.0;
            let z = ((i * 79) % 260) as f32 - 130.0;
            if x.abs() < 55.0 && z.abs() < 55.0 {
                continue;
            }
            s.draw_cube(v(x, 0.4, z), 1.2, 0.8, 1.5, Color::new(104, 112, 128, 255));
        }
        s.draw_cylinder(
            v(0.0, 0.0, 28.0),
            9.0,
            10.0,
            0.55,
            12,
            Color::new(27, 42, 58, 255),
        );
        s.draw_circle3D(v(0.0, 0.6, 28.0), 8.5, v(1.0, 0.0, 0.0), 90.0, ACCENT);
        s.draw_circle3D(
            v(0.0, 0.61, 28.0),
            6.0,
            v(1.0, 0.0, 0.0),
            90.0,
            Color::new(72, 135, 154, 255),
        );
        s.draw_cube(v(0.0, 0.62, 28.0), 4.0, 0.04, 0.6, ACCENT);
        s.draw_cube(v(-1.7, 0.62, 28.0), 0.6, 0.04, 4.0, ACCENT);
        s.draw_cube(v(1.7, 0.62, 28.0), 0.6, 0.04, 4.0, ACCENT);
        for i in 0..12 {
            let a = i as f32 / 12.0 * TAU;
            s.draw_sphere_ex(
                v(a.sin() * 9.0, 0.8, 28.0 + a.cos() * 9.0),
                0.22,
                4,
                6,
                ACCENT,
            );
        }
        // Base modular fuera del limite: la nave permanece en el patio de vuelo.
        for i in 0..4 {
            let x = -24.0 + i as f32 * 16.0;
            s.draw_cube(
                v(x, 4.0, -65.0),
                12.0,
                8.0,
                10.0,
                Color::new(123, 139, 158, 255),
            );
            s.draw_cube(
                v(x, 5.0, -59.9),
                8.0,
                2.4,
                0.1,
                Color::new(48, 216, 198, 255),
            );
            s.draw_cube(
                v(x, 8.4, -65.0),
                13.0,
                0.8,
                11.0,
                Color::new(37, 55, 72, 255),
            );
            s.draw_cylinder(
                v(x, 8.8, -65.0),
                0.15,
                0.2,
                7.0,
                6,
                Color::new(166, 183, 201, 255),
            );
            s.draw_sphere_ex(v(x, 16.0, -65.0), 0.4, 6, 8, ACCENT);
        }
        // Las mismas dimensiones se usan para dibujar y resolver colisiones.
        for (i, b) in crate::astronaut::PLATFORMS.iter().enumerate() {
            let color = if i < 6 {
                Color::new(82, 111, 134, 255)
            } else {
                Color::new(153, 109, 65, 255)
            };
            s.draw_cube_v(b.center, b.size, color);
            s.draw_cube_v(
                v(b.center.x, b.top() + 0.04, b.center.z),
                v(b.size.x + 0.04, 0.08, b.size.z + 0.04),
                Color::new(163, 192, 195, 255),
            );
            s.draw_cube_v(
                v(b.center.x, b.top() + 0.09, b.center.z + b.size.z * 0.35),
                v(b.size.x * 0.65, 0.02, 0.25),
                Color::new(249, 184, 81, 255),
            );
        }
        for (i, b) in BEACONS.iter().enumerate() {
            let c = if e.beacons[i] {
                ACCENT
            } else {
                Color::new(255, 190, 97, 255)
            };
            s.draw_cylinder(
                v(b.x, 0.0, b.z),
                2.4,
                3.0,
                0.5,
                8,
                Color::new(34, 47, 64, 255),
            );
            s.draw_circle3D(*b, 3.2, v(0.0, 1.0, 0.0), 0.0, c);
            s.draw_sphere_ex(*b, 0.5 + 0.1 * (e.local.time * 3.0).sin(), 6, 8, c);
            s.draw_line3D(v(b.x, 0.5, b.z), *b, c);
        }
        // Luces de limite y cheminos de aproximacion.
        for i in -10..=10 {
            let q = i as f32 * 5.0;
            for pos in [
                v(q, 0.2, -52.0),
                v(q, 0.2, 52.0),
                v(-52.0, 0.2, q),
                v(52.0, 0.2, q),
            ] {
                s.draw_cube(pos, 0.4, 0.3, 0.4, Color::new(73, 165, 187, 255));
            }
        }
        assets.kind(11);
        crate::ship::draw(&mut s, &assets.ship, &e.local, 0.55);
        if e.astronaut_visible() {
            crate::frog::draw(&mut s, &assets.frog_mesh, &e.astronaut);
        }
    }
    let collected = e.beacons.iter().filter(|&&b| b).count();
    for (i, b) in BEACONS.iter().enumerate() {
        if let Some(p) = scene::project(*b + v(0.0, 4.0, 0.0), camera, w, h)
            && p.x > 30.0
            && p.x < (w - 200) as f32
            && p.y > 160.0
            && p.y < (h - 190) as f32
        {
            d.draw_text(
                &format!("BALIZA {} {}", i + 1, if e.beacons[i] { "OK" } else { "" }),
                p.x as i32 - 40,
                p.y as i32,
                16,
                if e.beacons[i] { ACCENT } else { Color::GOLD },
            );
        }
    }
    d.draw_rectangle(24, 24, 430, 83, Color::new(7, 17, 32, 235));
    d.draw_rectangle(24, 24, 4, 83, ACCENT);
    d.draw_text("BASE NENUFAR", 44, 38, 30, Color::WHITE);
    d.draw_text("LUNA / EXPLORACION A PIE / v0.3", 44, 79, 15, ACCENT);
    d.draw_rectangle(w - 340, 24, 316, 96, Color::new(7, 17, 32, 235));
    d.draw_text(
        &format!("TELEMETRIA   {collected} / 3"),
        w - 320,
        42,
        23,
        ACCENT,
    );
    d.draw_text(
        if collected == 3 {
            "Vuelve a tu nave para despegar"
        } else {
            "Salta hasta las tres balizas doradas"
        },
        w - 320,
        82,
        14,
        Color::WHITE,
    );
    d.draw_rectangle(24, h - 142, w - 48, 118, Color::new(7, 17, 32, 235));
    if e.astronaut_visible() {
        let a = &e.astronaut;
        d.draw_text("LUMA / RANA DE ARBOL / CAPITANA", 44, h - 124, 18, ACCENT);
        d.draw_text(if pad {"Stick izq: caminar   Stick der: camara   X: saltar (mantener para cargar)   R2: correr"} else {"WASD: caminar   Flechas: camara   Espacio: saltar (mantener para cargar)   Shift: correr"},44,h-91,16,Color::WHITE);
        let message = match e.phase {
            Phase::Disembark => "DESCENSO DE LA TRIPULANTE / Preparando exploracion",
            Phase::Boarding => "ABORDANDO RANA-56 / Preparando cabina",
            _ if a.near_ship() => {
                if pad {
                    "TRIANGULO: abordar la nave   |   Options: pausa   |   Arriba: ayuda"
                } else {
                    "E: abordar la nave   |   Esc: pausa   |   H: ayuda"
                }
            }
            _ => {
                "Explora las plataformas. La baliza central requiere un salto cargado desde los escalones."
            }
        };
        d.draw_text(message, 44, h - 58, 16, ACCENT);
        if a.charging {
            d.draw_rectangle(w / 2 - 125, h - 205, 250, 40, Color::new(7, 17, 32, 240));
            d.draw_rectangle(
                w / 2 - 115,
                h - 196,
                (230.0 * a.charge) as i32,
                8,
                Color::GOLD,
            );
            d.draw_text("CARGANDO SALTO", w / 2 - 105, h - 181, 14, Color::GOLD);
        }
    } else {
        d.draw_text(
            &format!(
                "ALTURA  {:02.0} m    VELOCIDAD  {:02.0} m/s",
                e.local.position.y,
                length(e.local.velocity)
            ),
            44,
            h - 124,
            18,
            ACCENT,
        );
        d.draw_text(
            if pad {
                "R2/L2 avanzar/retroceder   Stick izq: orientar suave   Stick der: orientar"
            } else {
                "W/S avanzar/retroceder   A/D lateral   Espacio/Ctrl altura   Flechas orientar"
            },
            44,
            h - 91,
            17,
            Color::WHITE,
        );
        let message = match e.phase {
            Phase::Landing => "DESCENSO EN CURSO / Motores de aproximacion",
            Phase::Launch => "ASCENSO EN CURSO / Regresando al sistema solar",
            _ if e.at_pad() => {
                if pad {
                    "X: despegar   |   Cuadrado: salir de la nave   |   Options: pausa"
                } else {
                    "E: despegar   |   Q: salir de la nave   |   Esc: pausa"
                }
            }
            _ => "Explora el patio. Para salir, vuelve a la plataforma H y reduce velocidad.",
        };
        d.draw_text(message, 44, h - 58, 16, ACCENT);
    }
    // Brújula hacia la plataforma; siempre visible aunque quede detras de la camara.
    let delta = if e.astronaut_visible() {
        crate::astronaut::SPAWN - e.astronaut.position
    } else {
        PAD - e.local.position
    };
    let yaw = if e.astronaut_visible() {
        e.astronaut.camera_yaw
    } else {
        e.local.yaw
    };
    let angle = delta.x.atan2(delta.z) - yaw;
    let (cx, cy) = (w - 106, h - 224);
    d.draw_circle(cx, cy, 48.0, Color::new(7, 17, 32, 230));
    d.draw_circle_lines(cx, cy, 48.0, ACCENT);
    d.draw_line_ex(
        Vector2::new(cx as f32, cy as f32),
        Vector2::new(
            cx as f32 - angle.sin() * 32.0,
            cy as f32 - angle.cos() * 32.0,
        ),
        3.0,
        ACCENT,
    );
    d.draw_text(
        &format!("H  {:.0} m", length(delta)),
        cx - 38,
        cy + 55,
        16,
        Color::WHITE,
    );
}
