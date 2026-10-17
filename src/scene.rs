use crate::simulation::*;
use raylib::prelude::*;
use std::f32::consts::TAU;
use std::path::Path;

pub const ACCENT: Color = Color::new(125, 238, 202, 255);
const MUTED: Color = Color::new(138, 160, 181, 255);
const WHITE: Color = Color::new(229, 240, 248, 255);

pub struct Assets {
    pub sphere: Model,
    pub ship: Model,
    pub frog_mesh: Model,
    shader: Shader,
    kind_loc: i32,
    time_loc: i32,
    eye_loc: i32,
    stars: Vec<(Vector3, Color)>,
    rocks: Vec<(Vector3, f32)>,
}
fn random(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    (*seed >> 8) as f32 / 16777216.0
}
impl Assets {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path) -> Result<Self, String> {
        let shader = rl.load_shader_from_memory(
            thread,
            Some(include_str!("../assets/shaders/world.vs")),
            Some(include_str!("../assets/shaders/world.fs")),
        );
        let kind_loc = shader.get_shader_location("bodyKind");
        let time_loc = shader.get_shader_location("simTime");
        let eye_loc = shader.get_shader_location("eyePos");
        if kind_loc < 0 || eye_loc < 0 {
            return Err(
                "No se pudo compilar el shader de los planetas (requiere OpenGL 3.3).".into(),
            );
        }
        let mesh = Mesh::gen_mesh_sphere(thread, 1.0, 48, 80);
        // Se transfiere la propiedad de la malla al Model; Model la libera al salir.
        let mut sphere = rl
            .load_model_from_mesh(thread, unsafe { mesh.make_weak() })
            .map_err(|e| e.to_string())?;
        let path = root.join("assets/RANA_56.obj");
        if !path.is_file() {
            return Err(format!("No se encontro la nave: {}", path.display()));
        }
        let mut ship = rl
            .load_model(thread, &path.to_string_lossy())
            .map_err(|e| e.to_string())?;
        for material in sphere.materials_mut() {
            material.set_shader(&shader);
        }
        for material in ship.materials_mut() {
            material.set_shader(&shader);
        }
        let frog_mesh = Mesh::gen_mesh_sphere(thread, 1.0, 10, 12);
        let mut frog_mesh = rl
            .load_model_from_mesh(thread, unsafe { frog_mesh.make_weak() })
            .map_err(|e| e.to_string())?;
        for material in frog_mesh.materials_mut() {
            material.set_shader(&shader);
        }
        let mut seed = 24856;
        let stars = (0..1600)
            .map(|_| {
                let y = random(&mut seed) * 2.0 - 1.0;
                let angle = random(&mut seed) * TAU;
                let radial = (1.0 - y * y).sqrt();
                let bright = (100.0 + random(&mut seed) * 155.0) as u8;
                (
                    v(radial * angle.cos(), y, radial * angle.sin()),
                    Color::new(bright, bright, bright.saturating_add(20), 255),
                )
            })
            .collect();
        let rocks = (0..260)
            .map(|_| {
                let angle = random(&mut seed) * TAU;
                let radius = 105.0 + random(&mut seed) * 10.0;
                (
                    v(
                        angle.sin() * radius,
                        (random(&mut seed) - 0.5) * 4.0,
                        angle.cos() * radius,
                    ),
                    0.15 + random(&mut seed) * 0.55,
                )
            })
            .collect();
        Ok(Self {
            sphere,
            ship,
            frog_mesh,
            shader,
            kind_loc,
            time_loc,
            eye_loc,
            stars,
            rocks,
        })
    }
    pub(crate) fn kind(&mut self, kind: i32) {
        self.shader.set_shader_value(self.kind_loc, kind);
    }
    pub(crate) fn prepare(&mut self, camera: Camera3D, time: f32) {
        self.shader.set_shader_value(self.eye_loc, camera.position);
        self.shader.set_shader_value(self.time_loc, time);
    }
}

pub fn flight_camera(world: &World) -> Camera3D {
    Camera3D::perspective(
        world.position - world.forward() * 14.0 + v(0.0, 6.0, 0.0) + world.right() * 3.3,
        world.position + world.forward() * 20.0 + v(0.0, 1.0, 0.0),
        v(0.0, 1.0, 0.0),
        55.0,
    )
}
pub fn overview_camera() -> Camera3D {
    Camera3D::perspective(
        v(484.0, 516.0, 368.0),
        v(70.0, 0.0, -70.0),
        v(0.0, 1.0, 0.0),
        53.0,
    )
}
pub fn project(point: Vector3, camera: Camera3D, width: i32, height: i32) -> Option<Vector2> {
    let front = unit(camera.target - camera.position);
    let right = unit(cross(front, camera.up));
    let up = cross(right, front);
    let delta = point - camera.position;
    let depth = dot(delta, front);
    if depth < 0.1 {
        return None;
    }
    let focal = height as f32 / (2.0 * (camera.fovy.to_radians() * 0.5).tan());
    Some(Vector2::new(
        width as f32 * 0.5 + dot(delta, right) * focal / depth,
        height as f32 * 0.5 - dot(delta, up) * focal / depth,
    ))
}

pub fn draw_scene(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    world: &World,
    camera: Camera3D,
    orbits: bool,
    overview: bool,
) {
    let (w, h) = (d.get_screen_width(), d.get_screen_height());
    d.clear_background(Color::new(4, 8, 17, 255));
    d.draw_rectangle_gradient_v(
        0,
        0,
        w,
        h,
        Color::new(8, 14, 28, 255),
        Color::new(2, 5, 12, 255),
    );
    // Halo solar en pantalla; se dibuja antes de los cuerpos para respetar sus siluetas.
    if let Some(sun) = project(v(0.0, 0.0, 0.0), camera, w, h) {
        let distance = length(camera.position).max(14.0);
        let radius = (h as f32 * 12.0 / distance).clamp(12.0, 1000.0);
        d.draw_circle_gradient(
            sun.x as i32,
            sun.y as i32,
            radius * 2.0,
            Color::new(245, 119, 25, 50),
            Color::new(245, 119, 25, 0),
        );
    }
    assets
        .shader
        .set_shader_value(assets.eye_loc, camera.position);
    assets.shader.set_shader_value(assets.time_loc, world.time);
    {
        let mut space = d.begin_mode3D(camera);
        for (i, (direction, color)) in assets.stars.iter().enumerate() {
            let pos = camera.position + *direction * 500.0;
            space.draw_point3D(pos, *color);
            if i % 29 == 0 {
                space.draw_line3D(pos - v(0.14, 0.0, 0.0), pos + v(0.14, 0.0, 0.0), *color);
            }
        }
        if orbits || overview {
            for (i, planet) in PLANETS.iter().enumerate() {
                let color = if i == world.selected {
                    Color::new(68, 142, 143, 150)
                } else {
                    Color::new(43, 64, 84, 120)
                };
                for segment in 0..180 {
                    let a = segment as f32 / 180.0 * TAU;
                    let b = (segment + 1) as f32 / 180.0 * TAU;
                    space.draw_line3D(
                        v(a.sin() * planet.orbit, 0.0, a.cos() * planet.orbit),
                        v(b.sin() * planet.orbit, 0.0, b.cos() * planet.orbit),
                        color,
                    );
                }
            }
        }
        for (pos, size) in &assets.rocks {
            space.draw_cube(
                *pos,
                *size,
                *size * 0.7,
                *size * 1.2,
                Color::new(66, 63, 66, 255),
            );
        }
        assets.kind(0);
        space.draw_model(&assets.sphere, v(0.0, 0.0, 0.0), 12.0, Color::WHITE);
        for (i, planet) in PLANETS.iter().enumerate() {
            assets.kind(i as i32 + 1);
            let pos = planet.position(world.time);
            let axis = if i == 6 {
                v(0.0, 0.15, 1.0)
            } else {
                v(0.08, 1.0, 0.0)
            };
            space.draw_model_ex(
                &assets.sphere,
                pos,
                axis,
                world.time * 3.0 + 30.0,
                v(planet.radius, planet.radius, planet.radius),
                Color::WHITE,
            );
            if i == 5 || i == 6 {
                rings(&mut space, pos, planet.radius, i == 6);
            }
        }
        assets.kind(10);
        space.draw_model(
            &assets.sphere,
            moon_position(world.time),
            1.25,
            Color::WHITE,
        );
        assets.kind(9);
        crate::ship::draw(&mut space, &assets.ship, world, 0.38);
    }
    // Rotulos con ubicaciones alternativas para evitar superposiciones en el mapa.
    if world.selected == 8
        && let Some(p) = project(moon_position(world.time) + v(0.0, 3.0, 0.0), camera, w, h)
        && p.x > 30.0
        && p.x < (w - 260) as f32
        && p.y > 220.0
        && p.y < (h - 180) as f32
    {
        d.draw_circle_lines(p.x as i32, p.y as i32, 12.0, ACCENT);
        d.draw_text(
            "LUNA / BASE NENUFAR",
            p.x as i32 + 19,
            p.y as i32 - 6,
            14,
            ACCENT,
        );
    }
    let mut labels = Vec::<(i32, i32, i32, i32)>::new();
    if let Some(sun) = project(v(0.0, 0.0, 0.0), camera, w, h) {
        let depth = dot(-camera.position, unit(camera.target - camera.position)).max(0.1);
        let radius =
            (h as f32 * 12.0 / (2.0 * (camera.fovy.to_radians() * 0.5).tan() * depth)) as i32 + 6;
        labels.push((
            sun.x as i32 - radius,
            sun.y as i32 - radius,
            2 * radius,
            2 * radius,
        ));
    }
    if let Some(sun) = project(v(0.0, 16.0, 0.0), camera, w, h) {
        let (x, y) = (sun.x as i32 - 15, sun.y as i32 - 18);
        if x > 30 && x < w - 260 && y > 150 && y < h - 175 {
            d.draw_text("SOL", x, y, 16, Color::new(255, 201, 115, 255));
            labels.push((x - 8, y - 6, 50, 29));
        }
    }
    for (i, p) in PLANETS.iter().enumerate() {
        let position = p.position(world.time);
        if let Some(screen) = project(position + v(0.0, p.radius + 2.5, 0.0), camera, w, h) {
            let x = screen.x as i32;
            let y = screen.y as i32;
            if x > 30
                && x < w - 260
                && y > 100
                && y < h - 175
                && (overview || i == world.selected || length(position - camera.position) < 120.0)
            {
                let text = format!("{:02}  {}", i + 1, p.name);
                let width = text.len() as i32 * 9;
                for (dx, dy) in [
                    (32, 12),
                    (-width - 18, -22),
                    (32, -35),
                    (-width - 18, 30),
                    (25, 50),
                    (25, -62),
                ] {
                    let rect = (x + dx - 5, y + dy - 4, width + 10, 25);
                    let overlaps = labels.iter().any(|&(a, b, c, e)| {
                        rect.0 < a + c
                            && rect.0 + rect.2 > a
                            && rect.1 < b + e
                            && rect.1 + rect.3 > b
                    });
                    if rect.0 < 30
                        || rect.0 + rect.2 > w - 260
                        || rect.1 < 150
                        || rect.1 + rect.3 > h - 175
                        || overlaps
                    {
                        continue;
                    }
                    let anchor_x = if dx > 0 {
                        x + dx - 7
                    } else {
                        x + dx + width + 2
                    };
                    d.draw_circle(x, y + 20, 2.5, p.color);
                    d.draw_line(x, y + 20, anchor_x, y + dy + 8, p.color);
                    d.draw_text(&text, x + dx, y + dy, 16, WHITE);
                    labels.push(rect);
                    break;
                }
            }
        }
    }
}

fn rings<D: RaylibDraw3D>(d: &mut D, pos: Vector3, radius: f32, icy: bool) {
    let bands = if icy { 3 } else { 10 };
    for band in 0..bands {
        if !icy && band == 6 {
            continue;
        } // Division visible entre anillos.
        let inner = radius * (1.35 + band as f32 * 0.075);
        let outer = inner + radius * 0.058;
        let color = if icy {
            Color::new(121, 170, 178, 130)
        } else {
            Color::new(150 + band * 7, 130 + band * 6, 98 + band * 4, 220)
        };
        let point = |angle: f32, r: f32| {
            let x = angle.cos() * r;
            let z = angle.sin() * r;
            if icy {
                pos + v(x * 0.2, x * 0.98, z)
            } else {
                pos + v(x, z * 0.30, z * 0.954)
            }
        };
        for j in 0..100 {
            let a = j as f32 / 100.0 * TAU;
            let b = (j + 1) as f32 / 100.0 * TAU;
            let p0 = point(a, inner);
            let p1 = point(a, outer);
            let p2 = point(b, outer);
            let p3 = point(b, inner);
            for (a, b, c) in [(p0, p1, p2), (p0, p2, p3), (p2, p1, p0), (p3, p2, p0)] {
                d.draw_triangle3D(a, b, c, color);
            }
        }
    }
}

fn panel(d: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32) {
    d.draw_rectangle(x, y, w, h, Color::new(9, 18, 29, 225));
    d.draw_rectangle_lines(x, y, w, h, Color::new(42, 64, 80, 230));
}

pub fn draw_hud(d: &mut RaylibDrawHandle, world: &World, overview: bool, help: bool) {
    let (w, h) = (d.get_screen_width(), d.get_screen_height());
    let visited = world.visited.iter().filter(|&&x| x).count();
    d.draw_rectangle(28, 30, 4, 58, ACCENT);
    d.draw_text("RANA / 56", 48, 29, 34, WHITE);
    d.draw_text("ODISEA SOLAR   /   PROYECTO 03", 49, 69, 15, MUTED);
    let status = if world.paused {
        "SIMULACION EN PAUSA"
    } else if overview {
        "CARTA DEL SISTEMA"
    } else {
        "EXPLORACION ORBITAL"
    };
    d.draw_text(status, w / 2 - 110, 36, 17, ACCENT);
    d.draw_text(
        &format!("REGISTRO  {visited:02} / 08"),
        w / 2 - 78,
        63,
        15,
        MUTED,
    );
    d.draw_text("OPERACION CROAC / v0.3.0", w - 253, 34, 15, MUTED);
    d.draw_text("SOL  /  SISTEMA LOCAL", w - 253, 60, 17, WHITE);

    let nav_x = w - 247;
    panel(d, nav_x, 118, 219, 407);
    d.draw_text("DESTINOS", nav_x + 17, 134, 18, ACCENT);
    d.draw_text("L1/R1 elegir | Triangulo", nav_x + 12, 160, 12, MUTED);
    for (i, color) in PLANETS
        .iter()
        .map(|p| p.color)
        .chain(std::iter::once(ACCENT))
        .enumerate()
    {
        let y = 191 + i as i32 * 35;
        if world.selected == i {
            d.draw_rectangle(nav_x + 8, y - 3, 203, 31, Color::new(25, 52, 62, 255));
        }
        d.draw_text(&format!("{}", i + 1), nav_x + 17, y + 3, 16, MUTED);
        d.draw_circle(nav_x + 46, y + 11, 4.0, color);
        d.draw_text(
            destination_name(i),
            nav_x + 61,
            y + 3,
            16,
            if world.selected == i { WHITE } else { MUTED },
        );
        if i < 8 && world.visited[i] {
            d.draw_text("+", nav_x + 192, y + 2, 18, ACCENT);
        }
    }
    let subtitle = if world.selected == 8 {
        "LUNA / Descenso disponible / Base Nenufar"
    } else {
        PLANETS[world.selected].subtitle
    };
    d.draw_text(subtitle, 30, h - 174, 16, MUTED);
    if world.message_left > 0.0 {
        d.draw_rectangle(28, 111, (w - 310).min(760), 35, Color::new(15, 35, 43, 220));
        d.draw_text(&world.message, 42, 121, 15, ACCENT);
    }

    // Radar: proyeccion cenital del sistema y posicion de la nave.
    let rx = nav_x + 109;
    let ry = h - 195;
    panel(d, nav_x, h - 315, 219, 227);
    d.draw_text("RADAR / 600 u", nav_x + 15, h - 300, 14, MUTED);
    for p in PLANETS {
        d.draw_circle_lines(rx, ry, p.orbit * 0.29, Color::new(35, 56, 72, 255));
    }
    d.draw_circle(rx, ry, 4.0, Color::new(255, 191, 79, 255));
    for (i, p) in PLANETS.iter().enumerate() {
        let pos = p.position(world.time);
        let x = rx + (pos.x * 0.29) as i32;
        let y = ry + (pos.z * 0.29) as i32;
        d.draw_circle(x, y, if i == world.selected { 4.0 } else { 2.5 }, p.color);
    }
    let px = rx + (world.position.x * 0.29) as i32;
    let py = ry + (world.position.z * 0.29) as i32;
    d.draw_circle(px, py, 3.0, ACCENT);
    d.draw_line(
        px,
        py,
        px + (world.forward().x * 10.0) as i32,
        py + (world.forward().z * 10.0) as i32,
        ACCENT,
    );

    panel(d, 28, h - 143, w - 303, 115);
    d.draw_text("VELOCIDAD", 46, h - 125, 13, MUTED);
    d.draw_text(
        &format!("{:02.0}", length(world.velocity)),
        46,
        h - 104,
        36,
        WHITE,
    );
    d.draw_text("u/s", 107, h - 88, 15, MUTED);
    d.draw_line(165, h - 123, 165, h - 49, Color::new(45, 68, 82, 255));
    d.draw_text("NAVEGACION", 188, h - 125, 13, MUTED);
    d.draw_text(destination_name(world.selected), 188, h - 101, 25, WHITE);
    d.draw_text(
        &format!("{:.0} u hasta superficie", world.target_distance()),
        188,
        h - 64,
        14,
        MUTED,
    );
    let mode = if world.autopilot.is_some() {
        "PILOTO AUTOMATICO"
    } else {
        "VUELO MANUAL"
    };
    d.draw_text(mode, 438, h - 122, 16, ACCENT);
    d.draw_text(
        "W/S  impulso    A/D  lateral    Flechas  orientar",
        438,
        h - 94,
        14,
        WHITE,
    );
    d.draw_text(
        "ESP/CTRL  altura    SHIFT  turbo    X  frenar",
        438,
        h - 71,
        14,
        MUTED,
    );
    d.draw_text(
        "TAB  mapa   F  ruta   O  orbitas   P  pausa   H  ayuda",
        30,
        h - 19,
        13,
        MUTED,
    );
    if !overview {
        let x = w / 2;
        let y = h / 2;
        d.draw_line(x - 9, y, x - 4, y, Color::new(140, 215, 208, 100));
        d.draw_line(x + 4, y, x + 9, y, Color::new(140, 215, 208, 100));
        d.draw_line(x, y - 9, x, y - 4, Color::new(140, 215, 208, 100));
    }
    if world.paused {
        d.draw_text("PAUSA", w / 2 - 65, h / 2 - 28, 38, WHITE);
    }
    if help {
        let x = w / 2 - 300;
        let y = h / 2 - 225;
        panel(d, x, y, 600, 450);
        d.draw_text("BITACORA DEL PILOTO", x + 28, y + 26, 26, ACCENT);
        let lines = [
            "Mision: acercate a los ocho planetas para registrarlos.",
            "W / S        Avanzar / retroceder",
            "A / D        Desplazamiento lateral",
            "Flechas      Orientar la nave (giro e inclinacion)",
            "Espacio/Ctrl Subir / bajar",
            "Shift        Turbo      X: frenar",
            "1 - 8        Viajar al planeta seleccionado",
            "F            Iniciar / cancelar piloto automatico",
            "Tab          Vista general / tercera persona",
            "O / P        Orbitas / pausa",
            "R / F12      Reiniciar posicion / guardar captura",
            "H / Escape   Cerrar ayuda / salir del juego",
        ];
        for (i, line) in lines.iter().enumerate() {
            d.draw_text(
                line,
                x + 28,
                y + 78 + i as i32 * 28,
                16,
                if i == 0 { WHITE } else { MUTED },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_projection_rejects_objects_behind_camera() {
        let camera =
            Camera3D::perspective(v(0.0, 0.0, 10.0), v(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0), 60.0);
        assert!(project(v(0.0, 0.0, 20.0), camera, 1200, 800).is_none());
        let p = project(v(0.0, 0.0, 0.0), camera, 1200, 800).unwrap();
        assert_eq!(p, Vector2::new(600.0, 400.0));
    }
}
