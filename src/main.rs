mod controls;
mod expedition;
mod interface;
mod lunar;
mod scene;
mod ship;
mod simulation;

use raylib::prelude::*;
use simulation::*;
use std::path::PathBuf;

struct Options {
    frames: Option<u32>,
    capture: Option<PathBuf>,
    overview: bool,
    destination: Option<usize>,
    hidden: bool,
    lunar: bool,
    near_moon: bool,
    controller_report: bool,
    smoke_tour: bool,
    flight_check: bool,
}
fn options() -> Result<Options, String> {
    let mut opts = Options {
        frames: None,
        capture: None,
        overview: false,
        destination: None,
        hidden: false,
        lunar: false,
        near_moon: false,
        controller_report: false,
        smoke_tour: false,
        flight_check: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => {
                let n = args
                    .next()
                    .ok_or("Falta numero de frames")?
                    .parse::<u32>()
                    .map_err(|_| "Frames invalidos")?;
                if n == 0 {
                    return Err("--frames debe ser mayor que cero".into());
                }
                opts.frames = Some(n);
            }
            "--capture" => opts.capture = Some(args.next().ok_or("Falta ruta de captura")?.into()),
            "--overview" => opts.overview = true,
            "--hidden" => opts.hidden = true,
            "--lunar" => opts.lunar = true,
            "--near-moon" => opts.near_moon = true,
            "--controller-report" => opts.controller_report = true,
            "--smoke-tour" => opts.smoke_tour = true,
            "--flight-check" => opts.flight_check = true,
            "--planet" => {
                let n = args
                    .next()
                    .ok_or("Falta indice 1-8")?
                    .parse::<usize>()
                    .map_err(|_| "Planeta invalido")?;
                if !(1..=8).contains(&n) {
                    return Err("Usa un planeta del 1 al 8".into());
                }
                opts.destination = Some(n - 1);
            }
            "--help" => {
                println!(
                    "RANA / 56 - Operacion Croac v0.2.1\ncargo run --release\nOpciones: --frames N --capture archivo.png --overview --planet 1..8 --hidden --lunar --near-moon --controller-report --smoke-tour --flight-check"
                );
                std::process::exit(0);
            }
            _ => return Err(format!("Opcion desconocida: {arg}")),
        }
    }
    if opts.capture.is_some() && opts.frames.is_none() {
        opts.frames = Some(8);
    }
    if opts.smoke_tour {
        opts.frames = Some(9000);
    }
    Ok(opts)
}
fn project_root() -> Result<PathBuf, String> {
    let mut candidates = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR"))];
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        candidates.insert(0, dir.to_path_buf());
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.clone());
        candidates.push(cwd.join("proyecto3"));
    }
    candidates.into_iter().find(|p|p.join("assets/RANA_56.obj").is_file()).ok_or("No se encuentra assets/RANA_56.obj; ejecuta desde proyecto3 o conserva assets junto al ejecutable.".into())
}
fn run() -> Result<(), String> {
    let opts = options()?;
    let root = project_root()?;
    let mut builder = raylib::init();
    builder
        .size(1440, 900)
        .title("RANA-56 | Operacion Croac v0.2.1 - Proyecto 3")
        .resizable()
        .msaa_4x()
        .log_level(TraceLogLevel::LOG_WARNING);
    if opts.hidden {
        builder.hidden();
    }
    let (mut rl, thread) = builder.build();
    rl.set_exit_key(None);
    rl.set_target_fps(if opts.smoke_tour { 0 } else { 60 });
    rl.set_window_min_size(1280, 800);
    let mut assets = scene::Assets::load(&mut rl, &thread, &root)?;
    let mut world = World::default();
    let mut expedition = expedition::Expedition::default();
    let mut controls = controls::Controls::default();
    if opts.lunar {
        expedition.phase = expedition::Phase::Surface;
    }
    if opts.near_moon {
        world.position = moon_position(0.0) + v(0.0, 4.0, 8.0);
    }
    if opts.smoke_tour {
        world.travel(8);
    }
    if opts.flight_check {
        world.position = v(100.0, 50.0, 0.0);
        world.yaw = 2.4;
        world.pitch = 0.9;
        world.thrust = 0.8;
        expedition.local.position = v(0.0, 12.0, 28.0);
        expedition.local.yaw = 2.4;
        expedition.local.pitch = 0.55;
        expedition.local.thrust = 0.8;
    }
    if let Some(index) = opts.destination {
        world.selected = index;
        world.position = PLANETS[index].position(0.0)
            + v(
                0.0,
                PLANETS[index].radius * 0.6,
                PLANETS[index].radius * 3.0 + 9.0,
            );
        world.message = format!("Aproximacion a {}", PLANETS[index].name);
    }
    let mut overview = opts.overview;
    let mut orbits = true;
    let mut help = false;
    let mut diagnostic = false;
    let mut menu = 0i32;
    let mut was_focused = true;
    let mut camera = if expedition.phase == expedition::Phase::Orbit {
        scene::flight_camera(&world)
    } else {
        lunar::camera(&expedition)
    };
    let mut frames = 0u32;
    while !rl.window_should_close() {
        use KeyboardKey::*;
        let dt = if opts.frames.is_some() {
            1.0 / 60.0
        } else {
            rl.get_frame_time().min(0.05)
        };
        let action = controls.read(&rl);
        if frames == 0 || controls.disconnected {
            controls.report(&rl);
        }
        if opts.controller_report {
            diagnostic = true;
        }
        let focused = rl.is_window_focused();
        if controls.disconnected || (!opts.hidden && was_focused && !focused) {
            world.paused = true;
        }
        was_focused = focused;
        let mut consumed = false;
        if action.pause {
            if help || diagnostic {
                help = false;
                diagnostic = false;
            } else {
                world.paused = !world.paused;
                menu = 0;
            }
            consumed = true;
        }
        if !consumed && (help || diagnostic) {
            if action.back || (help && action.help) || (diagnostic && action.diagnostic) {
                help = false;
                diagnostic = false;
            }
            consumed = true;
        }
        if !consumed && world.paused {
            menu = (menu + action.menu_step).rem_euclid(4);
            if action.back {
                world.paused = false;
            }
            if action.confirm {
                match menu {
                    0 => world.paused = false,
                    1 => {
                        expedition.restart(&mut world);
                        world.paused = false;
                    }
                    2 => help = true,
                    _ => break,
                }
            }
            consumed = true;
        }
        if !consumed {
            if action.help {
                help = true;
            }
            if action.diagnostic {
                diagnostic = true;
            }
            if !help && !diagnostic {
                if action.reset {
                    expedition.restart(&mut world);
                }
                if expedition.phase == expedition::Phase::Orbit {
                    if action.map {
                        overview = !overview;
                    }
                    if rl.is_key_pressed(KEY_O) {
                        orbits = !orbits;
                    }
                    world.selected = (world.selected as i32 + action.next).rem_euclid(9) as usize;
                    if action.route {
                        if world.autopilot.is_some() {
                            world.autopilot = None;
                        } else {
                            world.travel(world.selected);
                        }
                    }
                    if let Some(i) = action.direct {
                        world.travel(i);
                    }
                }
                if action.interact {
                    expedition.interact(&mut world);
                }
            }
        }
        let mut input = action.flight;
        if opts.smoke_tour {
            input = Input::default();
            match expedition.phase {
                expedition::Phase::Orbit
                    if expedition.can_land(&world) && !expedition.completed =>
                {
                    expedition.interact(&mut world)
                }
                expedition::Phase::Surface => {
                    let target = expedition
                        .beacons
                        .iter()
                        .position(|b| !*b)
                        .map_or(expedition::PAD, |i| expedition::BEACONS[i]);
                    let delta = target - expedition.local.position;
                    input.thrust = dot(
                        delta,
                        v(expedition.local.yaw.sin(), 0.0, expedition.local.yaw.cos()),
                    ) * 0.2;
                    input.strafe = dot(delta, expedition.local.right()) * 0.2;
                    input.vertical = delta.y * 0.2;
                    if expedition.beacons.iter().all(|b| *b) && expedition.at_pad() {
                        expedition.interact(&mut world);
                    }
                }
                _ => {}
            }
        }
        let previous_phase = expedition.phase;
        if !opts.flight_check
            && !help
            && !diagnostic
            && !world.paused
            && (!consumed || opts.smoke_tour)
        {
            expedition.update(&mut world, input, dt);
        }
        if previous_phase != expedition.phase {
            println!("Fase: {:?}", expedition.phase);
        }
        let desired = if expedition.phase == expedition::Phase::Orbit {
            scene::flight_camera(&world)
        } else {
            lunar::camera(&expedition)
        };
        if previous_phase != expedition.phase || action.interact || action.reset {
            camera = desired;
        }
        let smoothing = 1.0 - (-6.0 * dt).exp();
        camera.position = camera.position + (desired.position - camera.position) * smoothing;
        camera.target = camera.target + (desired.target - camera.target) * smoothing;
        let active_camera = if overview && expedition.phase == expedition::Phase::Orbit {
            scene::overview_camera()
        } else {
            camera
        };
        let screenshot = rl.is_key_pressed(KEY_F12);
        {
            let mut d = rl.begin_drawing(&thread);
            if expedition.phase == expedition::Phase::Orbit {
                scene::draw_scene(&mut d, &mut assets, &world, active_camera, orbits, overview);
                scene::draw_hud(&mut d, &world, overview, false);
            } else {
                lunar::draw(
                    &mut d,
                    &mut assets,
                    &expedition,
                    active_camera,
                    controls.pad.is_some(),
                );
            }
            interface::overlay(
                &mut d,
                &controls,
                &world,
                &expedition,
                help,
                diagnostic,
                menu,
            );
        }
        frames += 1;
        let final_frame = opts.frames.is_some_and(|limit| frames >= limit)
            || (opts.smoke_tour && expedition.completed);
        if screenshot || (final_frame && opts.capture.is_some()) {
            let path = opts
                .capture
                .clone()
                .unwrap_or_else(|| root.join("screenshots/captura.png"));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let image = rl.load_image_from_screen(&thread);
            let png = image
                .export_image_to_memory(".png")
                .map_err(|e| e.to_string())?;
            std::fs::write(&path, &*png).map_err(|e| format!("{}: {e}", path.display()))?;
            println!("Captura guardada: {}", path.display());
        }
        if final_frame {
            break;
        }
    }
    if opts.smoke_tour && !expedition.completed {
        return Err("La prueba de ida y vuelta no termino".into());
    }
    println!(
        "Sesion terminada: {frames} frames; fase {:?}; balizas {:?}; recorrido completo: {}",
        expedition.phase, expedition.beacons, expedition.completed
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
