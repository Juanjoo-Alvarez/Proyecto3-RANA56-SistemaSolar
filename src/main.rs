mod scene;
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
}
fn options() -> Result<Options, String> {
    let mut opts = Options {
        frames: None,
        capture: None,
        overview: false,
        destination: None,
        hidden: false,
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
                    "RANA / 56 - Odisea Solar\ncargo run --release\nOpciones: --frames N --capture archivo.png --overview --planet 1..8 --hidden"
                );
                std::process::exit(0);
            }
            _ => return Err(format!("Opcion desconocida: {arg}")),
        }
    }
    if opts.capture.is_some() && opts.frames.is_none() {
        opts.frames = Some(8);
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
fn axis(rl: &RaylibHandle, positive: KeyboardKey, negative: KeyboardKey) -> f32 {
    i32::from(rl.is_key_down(positive)) as f32 - i32::from(rl.is_key_down(negative)) as f32
}
fn run() -> Result<(), String> {
    let opts = options()?;
    let root = project_root()?;
    let mut builder = raylib::init();
    builder
        .size(1440, 900)
        .title("RANA-56 | Odisea Solar - Proyecto 3")
        .resizable()
        .msaa_4x()
        .log_level(TraceLogLevel::LOG_WARNING);
    if opts.hidden {
        builder.hidden();
    }
    let (mut rl, thread) = builder.build();
    rl.set_target_fps(60);
    rl.set_window_min_size(1280, 800);
    let mut assets = scene::Assets::load(&mut rl, &thread, &root)?;
    let mut world = World::default();
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
    let mut camera = scene::flight_camera(&world);
    let mut frames = 0u32;
    while !rl.window_should_close() {
        use KeyboardKey::*;
        let dt = if opts.frames.is_some() {
            1.0 / 60.0
        } else {
            rl.get_frame_time().min(0.05)
        };
        if rl.is_key_pressed(KEY_TAB) {
            overview = !overview;
        }
        if rl.is_key_pressed(KEY_O) {
            orbits = !orbits;
        }
        if rl.is_key_pressed(KEY_H) {
            help = !help;
        }
        if rl.is_key_pressed(KEY_P) {
            world.paused = !world.paused;
        }
        if rl.is_key_pressed(KEY_R) {
            world.reset();
            camera = scene::flight_camera(&world);
        }
        if rl.is_key_pressed(KEY_F) {
            if world.autopilot.is_some() {
                world.autopilot = None;
            } else {
                world.travel(world.selected);
            }
        }
        let keys = [
            KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR, KEY_FIVE, KEY_SIX, KEY_SEVEN, KEY_EIGHT,
        ];
        for (i, key) in keys.into_iter().enumerate() {
            if rl.is_key_pressed(key) {
                world.travel(i);
            }
        }
        let input = if help {
            Input::default()
        } else {
            Input {
                thrust: axis(&rl, KEY_W, KEY_S),
                strafe: axis(&rl, KEY_D, KEY_A),
                vertical: axis(&rl, KEY_SPACE, KEY_LEFT_CONTROL),
                yaw: axis(&rl, KEY_LEFT, KEY_RIGHT),
                pitch: axis(&rl, KEY_UP, KEY_DOWN),
                boost: rl.is_key_down(KEY_LEFT_SHIFT),
                brake: rl.is_key_down(KEY_X),
            }
        };
        world.update(input, dt);
        let desired = scene::flight_camera(&world);
        let smoothing = 1.0 - (-6.0 * dt).exp();
        camera.position = camera.position + (desired.position - camera.position) * smoothing;
        camera.target = camera.target + (desired.target - camera.target) * smoothing;
        let active_camera = if overview {
            scene::overview_camera()
        } else {
            camera
        };
        let screenshot = rl.is_key_pressed(KEY_F12);
        {
            let mut d = rl.begin_drawing(&thread);
            scene::draw_scene(&mut d, &mut assets, &world, active_camera, orbits, overview);
            scene::draw_hud(&mut d, &world, overview, help);
        }
        frames += 1;
        let final_frame = opts.frames.is_some_and(|limit| frames >= limit);
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
    println!("Sesion terminada: {frames} frames; 8 planetas; nave RANA-56 cargada.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
