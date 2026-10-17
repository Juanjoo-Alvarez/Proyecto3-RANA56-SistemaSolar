use crate::{controls::Controls, expedition::*, scene::ACCENT, simulation::*};
use raylib::prelude::*;

pub fn overlay(
    d: &mut RaylibDrawHandle,
    controls: &Controls,
    world: &World,
    e: &Expedition,
    help: bool,
    diagnostic: bool,
    menu: i32,
) {
    let (w, h) = (d.get_screen_width(), d.get_screen_height());
    let pad = controls.pad.is_some();
    if e.phase == Phase::Orbit {
        // Indicaciones de vuelo reflejan el dispositivo conectado.
        if pad {
            d.draw_rectangle(435, h - 100, w - 745, 56, Color::new(9, 18, 29, 255));
            d.draw_text(
                "R2/L2 impulso   Stick izq lateral/altura",
                438,
                h - 94,
                14,
                Color::WHITE,
            );
            d.draw_text(
                "Stick der orientar   R3 turbo   Circulo frenar",
                438,
                h - 71,
                14,
                ACCENT,
            );
        }
        d.draw_rectangle(25, h - 23, w - 50, 23, Color::new(4, 8, 17, 255));
        d.draw_text(
            if pad {
                "Cuadrado: mapa   Triangulo: ruta   Options: pausa   Arriba: ayuda   Share: mando"
            } else {
                "TAB mapa   F ruta   1-9 destinos   E descender   ESC pausa   H ayuda   F3 mando"
            },
            30,
            h - 19,
            13,
            ACCENT,
        );
        let distance = length(world.position - moon_position(world.time));
        let text = if e.completed {
            "PRUEBA COMPLETADA / Base Nenufar conectada. Puedes volver a descender.".to_string()
        } else if e.can_land(world) {
            if pad {
                "CRUZ: descender a la Base Nenufar".into()
            } else {
                "E: descender a la Base Nenufar".into()
            }
        } else if distance < 13.0 {
            "Frena para descender: menos de 8 u/s".into()
        } else if world.selected == 8 {
            format!("Objetivo: Base Nenufar / Luna a {distance:.0} u / F o Triangulo: ruta")
        } else {
            "Descenso habilitado en la Luna / selecciona 9 o usa L1/R1".into()
        };
        d.draw_rectangle(28, 152, (w - 320).min(820), 38, Color::new(15, 35, 43, 240));
        d.draw_text(&text, 42, 164, 15, ACCENT);
    }
    // Identificacion en pantalla y diagnostico de conexion sin reiniciar el juego.
    let label = if pad {
        format!(
            "MANDO: {}",
            controls.name.chars().take(40).collect::<String>()
        )
    } else {
        "TECLADO / conecta DualShock 4 por USB".into()
    };
    d.draw_text(
        &label,
        30,
        if e.phase == Phase::Orbit { 196 } else { 128 },
        13,
        Color::new(153, 178, 195, 255),
    );
    if help || diagnostic {
        d.draw_rectangle(0, 0, w, h, Color::new(2, 5, 12, 210));
        let x = w / 2 - 390;
        let y = h / 2 - 290;
        d.draw_rectangle(x, y, 780, 580, Color::new(11, 25, 39, 255));
        d.draw_rectangle_lines(x, y, 780, 580, ACCENT);
        d.draw_text(
            if diagnostic {
                "PRUEBA DE MANDO"
            } else {
                "OPERACION CROAC / GUIA DE VUELO"
            },
            x + 30,
            y + 26,
            27,
            ACCENT,
        );
        if diagnostic {
            d.draw_text(&label, x + 30, y + 80, 18, Color::WHITE);
            if let Some(id) = controls.pad {
                use GamepadAxis::*;
                use GamepadButton::*;
                let axes = [
                    ("Stick izquierdo X", GAMEPAD_AXIS_LEFT_X),
                    ("Stick izquierdo Y", GAMEPAD_AXIS_LEFT_Y),
                    ("Stick derecho X", GAMEPAD_AXIS_RIGHT_X),
                    ("Stick derecho Y", GAMEPAD_AXIS_RIGHT_Y),
                    ("L2", GAMEPAD_AXIS_LEFT_TRIGGER),
                    ("R2", GAMEPAD_AXIS_RIGHT_TRIGGER),
                ];
                for (i, (name, a)) in axes.into_iter().enumerate() {
                    let value = d.get_gamepad_axis_movement(id, a);
                    let line = y + 130 + i as i32 * 38;
                    d.draw_text(
                        &format!("{name}: {value:+.2}"),
                        x + 30,
                        line,
                        17,
                        Color::WHITE,
                    );
                    d.draw_rectangle(x + 310, line, 350, 18, Color::new(29, 49, 64, 255));
                    d.draw_rectangle(x + 310, line, ((value + 1.0) * 175.0) as i32, 18, ACCENT);
                }
                let buttons = [
                    ("CRUZ", GAMEPAD_BUTTON_RIGHT_FACE_DOWN),
                    ("CIRCULO", GAMEPAD_BUTTON_RIGHT_FACE_RIGHT),
                    ("CUADRADO", GAMEPAD_BUTTON_RIGHT_FACE_LEFT),
                    ("TRIANGULO", GAMEPAD_BUTTON_RIGHT_FACE_UP),
                    ("L1", GAMEPAD_BUTTON_LEFT_TRIGGER_1),
                    ("R1", GAMEPAD_BUTTON_RIGHT_TRIGGER_1),
                    ("R3", GAMEPAD_BUTTON_RIGHT_THUMB),
                ];
                for (i, (name, b)) in buttons.into_iter().enumerate() {
                    let bx = x + 30 + (i as i32 % 4) * 175;
                    let by = y + 385 + (i as i32 / 4) * 44;
                    let active = d.is_gamepad_button_down(id, b);
                    d.draw_rectangle(
                        bx,
                        by,
                        162,
                        32,
                        if active {
                            ACCENT
                        } else {
                            Color::new(29, 49, 64, 255)
                        },
                    );
                    d.draw_text(
                        name,
                        bx + 8,
                        by + 8,
                        16,
                        if active { Color::BLACK } else { Color::WHITE },
                    );
                }
            } else {
                d.draw_text(
                    "Conecta el mando con un cable USB de datos.",
                    x + 30,
                    y + 140,
                    21,
                    Color::WHITE,
                );
                d.draw_text(
                    "La deteccion se actualiza automaticamente.",
                    x + 30,
                    y + 180,
                    19,
                    ACCENT,
                );
                d.draw_text(
                    "Puedes continuar la prueba con el teclado.",
                    x + 30,
                    y + 220,
                    19,
                    Color::WHITE,
                );
            }
            d.draw_text(
                "Share / F3 / Circulo: cerrar. El juego esta detenido.",
                x + 30,
                y + 530,
                17,
                ACCENT,
            );
        } else {
            let lines = [
                "1. Selecciona LUNA (9 o L1/R1) y activa F / Triangulo.",
                "2. Al llegar, pulsa E / Cruz para descender.",
                "3. Acercate a las tres balizas de la base lunar.",
                "4. Regresa a H, frena y pulsa E / Cruz para despegar.",
                "",
                "R2 / L2          Avanzar / retroceder       W / S",
                "Stick izquierdo  Lateral / altura          A,D / Espacio,Ctrl",
                "Stick derecho    Orientar la nave          Flechas",
                "R3 / Circulo     Turbo / frenar            Shift / X",
                "Cuadrado         Mapa del sistema          Tab",
                "L1 / R1          Seleccionar destino       [ / ]",
                "Options          Pausa y menu              Esc / P",
                "Share            Prueba de mando           F3",
                "",
                "Arriba / H / Circulo: cerrar ayuda. Progreso de sesion.",
            ];
            for (i, line) in lines.iter().enumerate() {
                d.draw_text(line, x + 30, y + 90 + i as i32 * 29, 17, Color::WHITE);
            }
        }
    }
    if world.paused && !help && !diagnostic {
        d.draw_rectangle(0, 0, w, h, Color::new(2, 5, 12, 215));
        let x = w / 2 - 250;
        let y = h / 2 - 190;
        d.draw_rectangle(x, y, 500, 380, Color::new(11, 25, 39, 255));
        d.draw_text("PAUSA", x + 36, y + 30, 34, ACCENT);
        for (i, label) in ["Continuar", "Reubicar nave", "Controles", "Salir del juego"]
            .iter()
            .enumerate()
        {
            let yy = y + 95 + i as i32 * 49;
            if menu == i as i32 {
                d.draw_rectangle(x + 24, yy - 7, 452, 40, Color::new(29, 67, 75, 255));
            }
            d.draw_text(label, x + 38, yy, 21, Color::WHITE);
        }
        d.draw_text(
            "Arriba/Abajo + Cruz/Enter | Circulo/Esc: volver",
            x + 24,
            y + 330,
            16,
            ACCENT,
        );
    }
}
