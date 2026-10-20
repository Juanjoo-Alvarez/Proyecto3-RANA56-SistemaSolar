mod astronaut;
mod audio;
mod cabin;
mod comms;
mod controls;
mod expedition;
mod frog;
mod hud;
mod lunar;
mod mercury;
mod portraits;
mod rescue;
mod scene;
mod ship;
mod simulation;
mod ui;

use raylib::prelude::*;
use simulation::*;
use std::path::PathBuf;

struct Options {
    cabin: bool,
    crew_preview: bool,
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
    walking: bool,
    portrait: bool,
    radio_preview: bool,
    dialogue_preview: bool,
    compact: bool,
    pad_preview: bool,
    mercury: bool,
    smoke_rescue: bool,
    celebration_preview: bool,
    celebration_demo: bool,
    audio_menu_preview: bool,
    no_audio: bool,
    audio_check: bool,
    mute: bool,
}
fn options() -> Result<Options, String> {
    let mut opts = Options {
        cabin: false,
        crew_preview: false,
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
        walking: false,
        portrait: false,
        radio_preview: false,
        dialogue_preview: false,
        compact: false,
        pad_preview: false,
        mercury: false,
        smoke_rescue: false,
        celebration_preview: false,
        celebration_demo: false,
        audio_menu_preview: false,
        no_audio: false,
        audio_check: false,
        mute: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cabin" => opts.cabin = true,
            "--crew-preview" => {
                opts.cabin = true;
                opts.crew_preview = true;
            }
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
            "--mercury" => {
                opts.mercury = true;
                opts.walking = true;
            }
            "--smoke-rescue" => opts.smoke_rescue = true,
            "--celebration-preview" => {
                opts.mercury = true;
                opts.walking = true;
                opts.celebration_preview = true;
            }
            "--celebration-demo" => {
                opts.mercury = true;
                opts.walking = true;
                opts.celebration_demo = true;
            }
            "--audio-menu-preview" => {
                opts.audio_menu_preview = true;
                opts.mercury = true;
                opts.walking = true;
            }
            "--no-audio" => opts.no_audio = true,
            "--audio-check" => opts.audio_check = true,
            "--mute" => opts.mute = true,
            "--walking" => opts.walking = true,
            "--radio-preview" => {
                opts.walking = true;
                opts.radio_preview = true;
            }
            "--dialogue-preview" => {
                opts.walking = true;
                opts.dialogue_preview = true;
            }
            "--compact" => opts.compact = true,
            "--pad-preview" => opts.pad_preview = true,
            "--portrait" => {
                opts.walking = true;
                opts.portrait = true;
            }
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
                    "RANA / 56 - Operacion Croac v0.4.3\ncargo run --release\nOpciones: --frames N --capture archivo.png --overview --planet 1..8 --hidden --lunar --near-moon --controller-report --smoke-tour --flight-check --walking --portrait --radio-preview --dialogue-preview --compact --pad-preview --mercury --smoke-rescue --celebration-preview --celebration-demo --audio-menu-preview --audio-check --mute --no-audio --cabin --crew-preview"
                );
                std::process::exit(0);
            }
            _ => return Err(format!("Opcion desconocida: {arg}")),
        }
    }
    if opts.capture.is_some() && opts.frames.is_none() {
        opts.frames = Some(8);
    }
    if opts.smoke_tour || opts.smoke_rescue {
        opts.frames = Some(9000);
    }
    if opts.audio_check && opts.no_audio {
        return Err("--audio-check requiere un dispositivo de audio".into());
    }
    if (opts.audio_check || opts.celebration_demo) && opts.frames.is_none() {
        opts.frames = Some(360);
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
        .size(
            if opts.compact { 1280 } else { 1440 },
            if opts.compact { 800 } else { 900 },
        )
        .title("RANA-56 | Operacion Croac v0.4.3 - Proyecto 3")
        .resizable()
        .msaa_4x()
        .log_level(TraceLogLevel::LOG_WARNING);
    if opts.hidden {
        builder.hidden();
    }
    let (mut rl, thread) = builder.build();
    rl.set_exit_key(None);
    rl.set_target_fps(if opts.smoke_tour || opts.smoke_rescue {
        0
    } else {
        60
    });
    rl.set_window_min_size(1280, 800);
    let mut assets = scene::Assets::load(&mut rl, &thread, &root)?;
    let mut portraits = portraits::Portraits::load(&mut rl, &thread)?;
    let ui = ui::Ui::load(&mut rl, &thread, &root)?;
    let settings_path = root.join("audio-settings.txt");
    let mut audio_settings = audio::AudioSettings::load(&settings_path);
    if opts.mute {
        audio_settings.muted = true;
    }
    // El dispositivo vive mas que los streams y sonidos que dependen de el.
    let audio_device = if opts.no_audio {
        None
    } else {
        match RaylibAudio::init_audio_device() {
            Ok(device) => Some(device),
            Err(error) => {
                eprintln!("Audio no disponible: {error}");
                None
            }
        }
    };
    let mut soundscape =
        audio_device
            .as_ref()
            .and_then(|device| match audio::Soundscape::load(device, &root) {
                Ok(audio) => Some(audio),
                Err(error) => {
                    eprintln!("Audio no disponible: {error}");
                    None
                }
            });
    if opts.audio_check && soundscape.is_none() {
        return Err("No se pudo inicializar el audio y cargar todos sus recursos".into());
    }
    println!(
        "Audio: {}",
        if soundscape.is_some() {
            "música y efectos listos"
        } else {
            "desactivado"
        }
    );
    let mut comms = comms::Comms::default();
    if opts.dialogue_preview {
        comms.begin_conversation();
        comms.reveal_all();
        comms.advance();
        comms.reveal_all();
    }
    let mut world = World::default();
    let mut expedition = expedition::Expedition::default();
    world.selected = 0;
    world.message = "Primer rescate: Mercurio. F / Triángulo para trazar ruta.".into();
    if opts.mercury {
        expedition.zone = expedition::Zone::Mercury;
        expedition.astronaut.mercury = true;
    }
    if opts.celebration_preview || opts.celebration_demo {
        expedition.rescue.collectors = [true; 3];
        expedition.rescue.status = rescue::Status::Celebration;
        expedition.rescue.celebration = if opts.celebration_preview {
            rescue::HIGH_FIVE_CONTACT
        } else {
            0.0
        };
        expedition.rescue.start = rescue::REFUGE + v(0.0, 0.0, 2.0);
        expedition.astronaut.position = expedition.rescue.start;
        expedition.rescue.update(&mut expedition.astronaut, 0.0);
    }
    let mut controls = controls::Controls::default();
    if opts.lunar {
        expedition.phase = expedition::Phase::Surface;
    }
    if opts.walking {
        expedition.phase = expedition::Phase::OnFoot;
        expedition.local.position.y = ship::GROUND_Y;
    }
    if opts.celebration_preview || opts.celebration_demo {
        expedition.phase = expedition::Phase::Rescue;
    }
    if opts.near_moon {
        world.position = moon_position(0.0) + v(0.0, 4.0, 8.0);
    }
    if opts.smoke_rescue {
        world.travel(0);
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
    let mut menu = if opts.audio_menu_preview { 3 } else { 0i32 };
    if opts.audio_menu_preview {
        world.paused = true;
    }
    if opts.cabin {
        if opts.crew_preview {
            expedition.rescue.status = rescue::Status::Complete;
            expedition.rescue.collectors = [true; 3];
        }
        expedition.phase = expedition::Phase::Orbit;
        expedition.toggle_cabin(&mut world);
    }
    let mut was_focused = true;
    let mut camera = if expedition.phase == expedition::Phase::Cabin {
        expedition.cabin.camera()
    } else if expedition.phase == expedition::Phase::Orbit {
        scene::flight_camera(&world)
    } else {
        lunar::camera(&expedition)
    };
    let mut frames = 0u32;
    let mut trial = astronaut::TrialPilot::default();
    let mut smoke_conversation = false;
    let mut rescue_trial = rescue::RescuePilot::default();
    while !rl.window_should_close() {
        use KeyboardKey::*;
        let dt = if opts.frames.is_some() {
            1.0 / 60.0
        } else {
            rl.get_frame_time().min(0.05)
        };
        let action = controls.read(&rl);
        let mut settings_changed = false;
        if action.mute {
            audio_settings.muted = !audio_settings.muted;
            settings_changed = true;
            comms.toast = if audio_settings.muted {
                "Audio silenciado"
            } else {
                "Audio activado"
            }
            .into();
            comms.toast_left = 2.0;
        }
        let previous_phase = expedition.phase;
        let previous_beacons = expedition.beacons;
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
            if let Some(audio) = &mut soundscape {
                audio.ui_confirm(&audio_settings);
            }
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
            menu = (menu + action.menu_step).rem_euclid(7);
            if action.menu_step != 0
                && let Some(audio) = &mut soundscape
            {
                audio.ui_move(&audio_settings);
            }
            if action.menu_adjust != 0 && (menu == 3 || menu == 4) {
                let value = if menu == 3 {
                    &mut audio_settings.music
                } else {
                    &mut audio_settings.effects
                };
                *value = (*value + action.menu_adjust as f32 * 0.05).clamp(0.0, 1.0);
                settings_changed = true;
                if let Some(audio) = &mut soundscape {
                    audio.ui_move(&audio_settings);
                }
            }
            if action.back {
                world.paused = false;
            }
            if action.confirm {
                if let Some(audio) = &mut soundscape {
                    audio.ui_confirm(&audio_settings);
                }
                match menu {
                    0 => world.paused = false,
                    1 => {
                        expedition.restart(&mut world);
                        world.paused = false;
                    }
                    2 => help = true,
                    5 => {
                        audio_settings.muted = !audio_settings.muted;
                        settings_changed = true;
                    }
                    6 => break,
                    _ => {}
                }
            }
            consumed = true;
        }
        if settings_changed && let Err(error) = audio_settings.save(&settings_path) {
            eprintln!("No se pudo guardar el volumen: {error}");
        }
        if !consumed && comms.blocking() {
            if action.confirm {
                comms.advance();
            }
            consumed = true;
        }
        if !consumed && action.radio_advance {
            comms.advance();
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
                if action.cabin && expedition.toggle_cabin(&mut world) {
                    overview = false;
                    consumed = true;
                    if let Some(audio) = &mut soundscape {
                        audio.ui_confirm(&audio_settings);
                    }
                }
                if !consumed && expedition.phase == expedition::Phase::Orbit {
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
                if !consumed && expedition.phase == expedition::Phase::Cabin {
                    if action.foot_interact {
                        expedition.cabin.talk(&mut comms);
                    }
                } else if !consumed && expedition.phase == expedition::Phase::OnFoot {
                    if action.foot_interact {
                        if expedition.zone == expedition::Zone::Moon
                            && expedition.astronaut.near_terminal()
                        {
                            comms.begin_conversation();
                        } else {
                            expedition.interact(&mut world);
                        }
                    }
                } else if !consumed && action.interact {
                    expedition.interact(&mut world);
                }
                if !consumed && action.disembark {
                    expedition.disembark();
                }
            }
        }
        let mut input = action.flight;
        let mut walk = action.walk;
        if opts.smoke_tour {
            input = Input::default();
            match expedition.phase {
                expedition::Phase::Orbit
                    if expedition.can_land(&world) && !expedition.completed =>
                {
                    expedition.interact(&mut world)
                }
                expedition::Phase::Surface => {
                    if expedition.beacons.iter().all(|b| *b) {
                        expedition.interact(&mut world);
                    } else {
                        expedition.disembark();
                    }
                }
                expedition::Phase::OnFoot => {
                    if !smoke_conversation {
                        comms.begin_conversation();
                        smoke_conversation = true;
                    }
                    if comms.blocking() {
                        walk = astronaut::WalkInput::default();
                        if frames.is_multiple_of(60) {
                            comms.advance();
                        }
                    } else {
                        walk = trial.input(&expedition.astronaut, dt);
                    }
                    if expedition.beacons.iter().all(|b| *b) && expedition.astronaut.near_ship() {
                        expedition.interact(&mut world);
                    }
                }
                _ => {}
            }
        }
        if opts.smoke_rescue {
            input = Input::default();
            walk = astronaut::WalkInput::default();
            if comms.blocking() {
                if frames.is_multiple_of(30) {
                    comms.advance();
                }
            } else {
                match expedition.phase {
                    expedition::Phase::Orbit
                        if expedition.can_land(&world) && !expedition.rescue.thermal_shield() =>
                    {
                        expedition.interact(&mut world)
                    }
                    expedition::Phase::OnFoot => {
                        walk = rescue_trial.input(&expedition.astronaut, &expedition.rescue);
                        if rescue_trial.interact(&expedition.astronaut, &expedition.rescue) {
                            expedition.interact(&mut world);
                        }
                    }
                    expedition::Phase::Surface => expedition.interact(&mut world),
                    _ => {}
                }
            }
        }
        if !opts.flight_check
            && !opts.portrait
            && !opts.celebration_preview
            && !help
            && !diagnostic
            && !world.paused
            && !comms.blocking()
            && (!consumed || opts.smoke_tour || opts.smoke_rescue)
        {
            expedition.update(&mut world, input, walk, dt);
        }
        if !opts.celebration_preview && !opts.celebration_demo {
            comms.observe(&expedition, previous_beacons);
        }
        world.thermal_shield = expedition.rescue.thermal_shield();
        if !world.paused && !help && !diagnostic {
            comms.update(dt);
        }
        if opts.radio_preview || opts.dialogue_preview {
            comms.reveal_all();
        }
        if previous_phase != expedition.phase {
            println!("Fase: {:?}", expedition.phase);
        }
        let desired = if expedition.phase == expedition::Phase::Cabin {
            expedition.cabin.camera()
        } else if expedition.phase == expedition::Phase::Orbit {
            scene::flight_camera(&world)
        } else {
            lunar::camera(&expedition)
        };
        if previous_phase != expedition.phase || action.reset {
            camera = desired;
        }
        let smoothing = 1.0 - (-6.0 * dt).exp();
        camera.position = camera.position + (desired.position - camera.position) * smoothing;
        camera.target = camera.target + (desired.target - camera.target) * smoothing;
        if expedition.phase == expedition::Phase::OnFoot {
            camera.position = expedition
                .astronaut
                .clip_camera(camera.target, camera.position);
        }
        if opts.portrait {
            camera = Camera3D::perspective(
                expedition.astronaut.position + v(4.3, 3.1, -7.0),
                expedition.astronaut.position + v(0.0, 1.6, 0.0),
                v(0.0, 1.0, 0.0),
                38.0,
            );
        }
        let active_camera = if overview && expedition.phase == expedition::Phase::Orbit {
            scene::overview_camera()
        } else {
            camera
        };
        if let Some(audio) = &mut soundscape {
            audio.update(
                &world,
                &expedition,
                &comms,
                &audio_settings,
                action.flight.boost,
                world.paused || help || diagnostic,
                dt,
            );
        }
        portraits.update(&mut rl, &thread, &mut assets, &comms);
        let screenshot = rl.is_key_pressed(KEY_F12);
        {
            let mut d = rl.begin_drawing(&thread);
            if expedition.phase == expedition::Phase::Cabin {
                cabin::draw(
                    &mut d,
                    &mut assets,
                    &expedition.cabin,
                    active_camera,
                    &world,
                    &ui,
                    &comms,
                );
            } else if expedition.phase == expedition::Phase::Orbit {
                scene::draw_scene(
                    &mut d,
                    &mut assets,
                    &world,
                    active_camera,
                    orbits,
                    overview,
                    &ui,
                );
            } else if expedition.zone == expedition::Zone::Mercury {
                mercury::draw(&mut d, &mut assets, &expedition, active_camera);
            } else {
                lunar::draw(&mut d, &mut assets, &expedition, active_camera);
            }
            hud::Hud {
                audio_settings: &audio_settings,
                audio_available: soundscape.is_some(),
                portraits: &portraits,
                ui: &ui,
                controls: &controls,
                world: &world,
                e: &expedition,
                comms: &comms,
                camera: active_camera,
                overview,
                help,
                diagnostic,
                menu,
                gamepad_hints: controls.pad_hints() || opts.pad_preview,
                turbo: action.flight.boost,
            }
            .draw(&mut d);
        }
        frames += 1;
        let final_frame = opts.frames.is_some_and(|limit| frames >= limit)
            || (opts.smoke_tour && expedition.completed)
            || (opts.smoke_rescue && expedition.rescue.thermal_shield());
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
    if let Some(audio) = &soundscape {
        println!("{}", audio.report());
    }
    if opts.smoke_rescue
        && (!expedition.rescue.thermal_shield() || comms.conversations_finished != 1)
    {
        return Err(format!(
            "Rescate incompleto: {:?}; colectores {:?}; Luma {:?}; Brasa {:?}; prueba paso {}; diálogo {}",
            expedition.rescue.status,
            expedition.rescue.collectors,
            expedition.astronaut.position,
            expedition.rescue.brasa.position,
            rescue_trial.waypoint,
            comms.conversations_finished
        ));
    }
    println!(
        "Rescate {:?}; emergencias térmicas {}; escudo {}",
        expedition.rescue.status, expedition.rescue.overheats, world.thermal_shield
    );
    if opts.smoke_tour && !expedition.completed {
        return Err("La prueba de ida y vuelta no termino".into());
    }
    if opts.smoke_tour && comms.conversations_finished != 1 {
        return Err("No se completo la conversacion de prueba".into());
    }
    println!(
        "Comunicaciones: {} mensajes; {} conversaciones completadas",
        comms.messages_shown, comms.conversations_finished
    );
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
