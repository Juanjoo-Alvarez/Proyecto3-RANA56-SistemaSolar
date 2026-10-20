use crate::{
    comms::{Comms, Kind, Portrait},
    controls::Controls,
    expedition::{Expedition, Phase},
    simulation::*,
    ui::*,
};
use raylib::prelude::*;

pub struct Hud<'a> {
    pub audio_settings: &'a crate::audio::AudioSettings,
    pub audio_available: bool,
    pub ui: &'a Ui,
    pub controls: &'a Controls,
    pub world: &'a World,
    pub e: &'a Expedition,
    pub portraits: &'a crate::portraits::Portraits,
    pub comms: &'a Comms,
    pub camera: Camera3D,
    pub overview: bool,
    pub help: bool,
    pub diagnostic: bool,
    pub menu: i32,
    pub gamepad_hints: bool,
    pub turbo: bool,
}
fn pretty(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or(String::new(), |c| {
        c.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
    })
}
impl Hud<'_> {
    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
        let ui = self.ui;
        let e = self.e;
        if e.phase == Phase::Cabin {
            self.cabin(d, w, h);
        } else if e.phase == Phase::Orbit {
            self.orbit(d, w, h);
        } else {
            if e.zone == crate::expedition::Zone::Mercury {
                self.mercury(d, w, h);
            } else {
                self.lunar(d, w, h);
            }
        }
        if self.comms.toast_left > 0.0 {
            let slide = ((3.5 - self.comms.toast_left) * 7.0).min(1.0);
            let x = w * 0.5 - 170.0;
            let y = 128.0 - (1.0 - slide) * 12.0;
            ui.panel(d, Rectangle::new(x, y, 340.0, 53.0));
            self.portraits.draw(
                d,
                Rectangle::new(x + 7.0, y + 7.0, 40.0, 40.0),
                Portrait::Luma,
            );
            ui.text(d, &self.comms.toast, x + 60.0, y + 15.0, 21.0, MINT);
        }
        if !self.help && !self.diagnostic && !self.world.paused {
            self.caption(d, w, h);
        }
        if self.help || self.diagnostic {
            self.sheet(d, w, h);
        } else if self.world.paused {
            self.pause(d, w, h);
        }
        if self.audio_settings.muted {
            ui.text(d, "Audio silenciado", w - 310.0, h - 22.0, 13.0, MUTED);
        }
        // Una firma discreta; los detalles del dispositivo viven en su diagnostico.
        ui.text(
            d,
            "RANA / 56 · 0.4.3",
            w - 200.0,
            h - 22.0,
            13.0,
            MUTED,
        );
    }
    fn heading(&self, d: &mut RaylibDrawHandle, title: &str, subtitle: &str) {
        let ui = self.ui;
        ui.panel(d, Rectangle::new(26.0, 25.0, 322.0, 87.0));
        leaf(d, Vector2::new(54.0, 65.0), 16.0, MINT);
        ui.text(d, title, 82.0, 40.0, 29.0, CREAM);
        ui.text(d, subtitle, 83.0, 77.0, 15.0, MUTED);
    }
    fn cabin(&self, d: &mut RaylibDrawHandle, w: f32, h: f32) {
        let ui = self.ui;
        let cabin = &self.e.cabin;
        self.heading(
            d,
            "Hogar entre estrellas",
            "RANA-56 · Cubierta de tripulación",
        );
        ui.panel(d, Rectangle::new(w - 294.0, 25.0, 268.0, 132.0));
        ui.text(
            d,
            "Ninguna rana se queda atrás",
            w - 278.0,
            41.0,
            18.0,
            CREAM,
        );
        ui.text(
            d,
            &format!("Rescatados  {} / 8", cabin.count()),
            w - 278.0,
            75.0,
            25.0,
            MINT,
        );
        ui.text(d, "Luma · Capitana a bordo", w - 278.0, 118.0, 17.0, MUTED);
        ui.text(
            d,
            "Nave estacionada · ruta detenida",
            28.0,
            129.0,
            17.0,
            MUTED,
        );
        ui.prompt(
            d,
            "Volver a pilotar",
            Button::Circle,
            self.gamepad_hints,
            "C",
            Vector2::new(w * 0.5, h - 78.0),
        );
        ui.text(
            d,
            if self.gamepad_hints {
                "Stick izq · Caminar     Stick der · Cámara"
            } else {
                "WASD · Caminar     Flechas · Cámara"
            },
            28.0,
            h - 51.0,
            17.0,
            CREAM,
        );
        if !self.comms.blocking() && cabin.near_brasa() {
            ui.prompt(
                d,
                "Hablar con Brasa",
                Button::Triangle,
                self.gamepad_hints,
                "E",
                Vector2::new(
                    w * 0.5,
                    if self.comms.active.is_some() {
                        h - 302.0
                    } else {
                        h - 143.0
                    },
                ),
            );
        }
    }
    fn mercury(&self, d: &mut RaylibDrawHandle, w: f32, h: f32) {
        use crate::rescue::{self, Status};
        let ui = self.ui;
        let e = self.e;
        let r = &e.rescue;
        self.heading(d, "Estación Girasol", "Mercurio · Rescate de Brasa");
        let objective = match r.status {
            Status::Waiting if !r.unlocked() => "Apaga los tres colectores",
            Status::Waiting => "Abre el refugio de Brasa",
            Status::Celebration => "¡Ninguna rana se queda atrás!",
            Status::Escort => "Reúnete con Brasa en la nave",
            Status::Aboard | Status::Complete => "Despega con tu tripulante",
        };
        ui.panel(d, Rectangle::new(26.0, 126.0, 322.0, 80.0));
        ui.text(d, objective, 43.0, 141.0, 19.0, CREAM);
        ui.text(
            d,
            &format!(
                "Colectores apagados  {} / 3",
                r.collectors.iter().filter(|b| **b).count()
            ),
            43.0,
            173.0,
            18.0,
            MINT,
        );
        ui.panel(d, Rectangle::new(w - 278.0, 25.0, 252.0, 110.0));
        let cool = rescue::shaded(e.astronaut.position) || r.unlocked();
        let color = if r.heat > 70.0 {
            Color::new(255, 120, 78, 255)
        } else {
            GOLD
        };
        ui.text(
            d,
            if r.unlocked() {
                "Refrigeración restaurada"
            } else if cool {
                "En sombra · enfriando"
            } else {
                "Exposición solar"
            },
            w - 260.0,
            41.0,
            19.0,
            if cool { MINT } else { color },
        );
        d.draw_rectangle_rounded(
            Rectangle::new(w - 258.0, 76.0, 211.0, 9.0),
            0.8,
            8,
            Color::new(56, 62, 70, 255),
        );
        if r.heat > 0.0 {
            d.draw_rectangle_rounded(
                Rectangle::new(w - 258.0, 76.0, 211.0 * r.heat / 100.0, 9.0),
                0.8,
                8,
                color,
            );
        }
        ui.text(
            d,
            &format!("Temperatura del traje  {:.0}%", r.heat),
            w - 260.0,
            99.0,
            16.0,
            MUTED,
        );
        ui.text(
            d,
            if self.gamepad_hints {
                "Cruceta arriba · Ayuda"
            } else {
                "H · Ayuda    Esc · Pausa"
            },
            w - 262.0,
            146.0,
            15.0,
            MUTED,
        );
        if e.phase == Phase::OnFoot {
            let a = &e.astronaut;
            for (i, p) in rescue::COLLECTORS.iter().enumerate() {
                if let Some(p) =
                    crate::scene::project(*p + v(0.0, 4.1, 0.0), self.camera, w as i32, h as i32)
                    && p.x > 35.0
                    && p.x < w - 40.0
                    && p.y > 215.0
                    && p.y < h - 255.0
                {
                    ui.text(
                        d,
                        &format!(
                            "Colector {} {}",
                            i + 1,
                            if r.collectors[i] { "OK" } else { "" }
                        ),
                        p.x - 45.0,
                        p.y,
                        17.0,
                        if r.collectors[i] { MINT } else { GOLD },
                    );
                }
            }
            if let Some(p) = crate::scene::project(
                rescue::REFUGE + v(0.0, 7.5, 0.0),
                self.camera,
                w as i32,
                h as i32,
            ) && p.y > 215.0
                && p.y < h - 255.0
                && p.x > 45.0
                && p.x < w - 160.0
            {
                ui.text(d, "Refugio de Brasa", p.x - 60.0, p.y, 20.0, GOLD);
            }
            let prompt = if let Some(i) = r.collector_near(a) {
                Some(if r.collectors[i] {
                    "Colector apagado"
                } else {
                    "Apagar colector"
                })
            } else if r.near_refuge(a) && r.status == Status::Waiting {
                Some(if r.unlocked() {
                    "Abrir refugio · Rescatar a Brasa"
                } else {
                    "Refugio bloqueado · faltan colectores"
                })
            } else if a.near_ship() {
                Some(if r.status == Status::Escort && !r.brasa.near_ship() {
                    "Esperar a Brasa en el punto de abordaje"
                } else {
                    "Abordar RANA-56"
                })
            } else {
                None
            };
            if !self.comms.blocking()
                && let Some(label) = prompt
            {
                ui.prompt(
                    d,
                    label,
                    Button::Triangle,
                    self.gamepad_hints,
                    "E",
                    Vector2::new(
                        w * 0.5,
                        if self.comms.active.is_some() {
                            h - 302.0
                        } else {
                            h - 94.0
                        },
                    ),
                );
            }
            let target = if r.status == Status::Waiting && r.unlocked() {
                rescue::REFUGE
            } else {
                crate::astronaut::SPAWN
            };
            ui.panel(d, Rectangle::new(w - 235.0, h - 114.0, 209.0, 81.0));
            ui.text(
                d,
                if target == rescue::REFUGE {
                    "Refugio de Brasa"
                } else {
                    "RANA-56"
                },
                w - 218.0,
                h - 102.0,
                20.0,
                MINT,
            );
            ui.text(
                d,
                &format!("{:.0} m", length(target - a.position)),
                w - 218.0,
                h - 69.0,
                18.0,
                CREAM,
            );
        } else if e.phase == Phase::Surface {
            ui.prompt(
                d,
                "Despegar",
                Button::Cross,
                self.gamepad_hints,
                "E",
                Vector2::new(w * 0.5 - 110.0, h - 100.0),
            );
            ui.prompt(
                d,
                "Salir",
                Button::Square,
                self.gamepad_hints,
                "Q",
                Vector2::new(w * 0.5 + 110.0, h - 100.0),
            );
        }
    }
    fn lunar(&self, d: &mut RaylibDrawHandle, w: f32, h: f32) {
        let ui = self.ui;
        let e = self.e;
        let pad = self.gamepad_hints;
        self.heading(d, "Base Nenúfar", "Luna · Expedición de Luma");
        ui.panel(d, Rectangle::new(w - 275.0, 25.0, 249.0, 103.0));
        ui.text(d, "Señales recuperadas", w - 254.0, 41.0, 18.0, MUTED);
        let count = e.beacons.iter().filter(|&&b| b).count();
        ui.text(d, &format!("{count} / 3"), w - 254.0, 69.0, 29.0, CREAM);
        for (i, &done) in e.beacons.iter().enumerate() {
            let x = w - 130.0 + i as f32 * 34.0;
            d.draw_circle(
                x as i32,
                86,
                12.0,
                if done {
                    MINT
                } else {
                    Color::new(61, 78, 86, 255)
                },
            );
            if done {
                leaf(d, Vector2::new(x, 86.0), 8.0, Color::new(19, 58, 50, 255));
            } else {
                d.draw_circle_lines(x as i32, 86, 12.0, GOLD);
            }
        }
        ui.text(
            d,
            if pad {
                "Cruceta arriba · Ayuda"
            } else {
                "H · Ayuda    Esc · Pausa"
            },
            w - 268.0,
            138.0,
            14.0,
            MUTED,
        );
        if !e.astronaut_visible() {
            self.speed(
                d,
                Vector2::new(26.0, 184.0),
                length(e.local.velocity),
                e.local.thrust,
            );
        }
        for (i, beacon) in crate::expedition::BEACONS.iter().enumerate() {
            if let Some(p) =
                crate::scene::project(*beacon + v(0.0, 3.0, 0.0), self.camera, w as i32, h as i32)
                && p.x > 45.0
                && p.x < w - 50.0
                && p.y > 190.0
                && p.y < h - 240.0
            {
                let text = format!(
                    "{} {}",
                    if e.beacons[i] { "Señal" } else { "Baliza" },
                    i + 1
                );
                ui.text(
                    d,
                    &text,
                    p.x - 28.0,
                    p.y,
                    17.0,
                    if e.beacons[i] { MINT } else { GOLD },
                );
            }
        }
        if e.astronaut_visible() {
            let a = &e.astronaut;
            if !self.comms.blocking() && e.phase == Phase::OnFoot {
                let prompt_y = if self.comms.active.is_some() {
                    h - 302.0
                } else {
                    h - 94.0
                };
                if a.near_terminal() {
                    ui.prompt(
                        d,
                        "Hablar con la base",
                        Button::Triangle,
                        pad,
                        "E",
                        Vector2::new(w * 0.5, prompt_y),
                    );
                } else if a.near_ship() {
                    ui.prompt(
                        d,
                        "Abordar la nave",
                        Button::Triangle,
                        pad,
                        "E",
                        Vector2::new(w * 0.5, prompt_y),
                    );
                }
                if a.charging {
                    let y = if self.comms.active.is_some() {
                        h - 359.0
                    } else {
                        h - 151.0
                    };
                    ui.panel(d, Rectangle::new(w * 0.5 - 112.0, y, 224.0, 47.0));
                    ui.text(d, "Salto cargado", w * 0.5 - 94.0, y + 5.0, 17.0, GOLD);
                    d.draw_rectangle_rounded(
                        Rectangle::new(w * 0.5 - 94.0, y + 31.0, 188.0, 7.0),
                        0.9,
                        8,
                        Color::new(56, 66, 70, 255),
                    );
                    if a.charge > 0.01 {
                        d.draw_rectangle_rounded(
                            Rectangle::new(w * 0.5 - 94.0, y + 31.0, 188.0 * a.charge, 7.0),
                            0.9,
                            8,
                            GOLD,
                        );
                    }
                }
            }
        } else if e.phase == Phase::Surface && e.at_pad() && !self.comms.blocking() {
            let y = if self.comms.active.is_some() {
                h - 302.0
            } else {
                h - 100.0
            };
            ui.prompt(
                d,
                "Despegar",
                Button::Cross,
                pad,
                "E",
                Vector2::new(w * 0.5 - 110.0, y),
            );
            ui.prompt(
                d,
                "Salir",
                Button::Square,
                pad,
                "Q",
                Vector2::new(w * 0.5 + 110.0, y),
            );
        } else {
            let text = match e.phase {
                Phase::Landing => "Aproximación lunar",
                Phase::Launch => "Ascendiendo a órbita",
                Phase::Disembark => "Preparando expedición",
                Phase::Boarding => "Abordando RANA-56",
                _ => "Vuelo sobre la base",
            };
            ui.pill(d, text, 28.0, 130.0, MINT);
        }
        let (pos, yaw) = if e.astronaut_visible() {
            (e.astronaut.position, e.astronaut.camera_yaw)
        } else {
            (e.local.position, e.local.yaw)
        };
        let delta = crate::astronaut::SPAWN - pos;
        let angle = delta.x.atan2(delta.z) - yaw;
        let c = Vector2::new(w - 79.0, h - 107.0);
        ui.panel(d, Rectangle::new(w - 139.0, h - 169.0, 113.0, 137.0));
        d.draw_circle_lines_v(c, 35.0, Color::new(96, 143, 139, 150));
        let tip = c + Vector2::new(-angle.sin() * 26.0, -angle.cos() * 26.0);
        d.draw_line_ex(c, tip, 2.5, MINT);
        d.draw_circle_v(tip, 3.5, MINT);
        ui.text(d, "RANA-56", w - 126.0, h - 68.0, 16.0, CREAM);
        ui.text(
            d,
            &format!("{:.0} m", length(delta)),
            w - 111.0,
            h - 46.0,
            14.0,
            MUTED,
        );
    }
    fn orbit(&self, d: &mut RaylibDrawHandle, w: f32, h: f32) {
        let ui = self.ui;
        let world = self.world;
        let pad = self.gamepad_hints;
        self.heading(
            d,
            &pretty(destination_name(world.selected)),
            "RANA-56 · Navegación orbital",
        );
        ui.button(d, Button::Circle, pad, "C", 28.0, 354.0);
        ui.text(d, "Entrar a la nave", 78.0, 360.0, 19.0, CREAM);
        ui.text(
            d,
            &format!("{:.0} u hasta la superficie", world.target_distance()),
            43.0,
            128.0,
            21.0,
            CREAM,
        );
        ui.pill(
            d,
            if world.autopilot.is_some() {
                "Ruta automática"
            } else {
                "Vuelo libre"
            },
            28.0,
            163.0,
            MINT,
        );
        self.speed(
            d,
            Vector2::new(26.0, 212.0),
            length(world.velocity),
            world.thrust,
        );
        ui.text(
            d,
            &format!(
                "Exploración  {} / 8",
                world.visited.iter().filter(|&&b| b).count()
            ),
            w - 210.0,
            34.0,
            20.0,
            MINT,
        );
        ui.text(
            d,
            if pad {
                "Cruceta arriba · Ayuda"
            } else {
                "H · Ayuda    Esc · Pausa"
            },
            w - 225.0,
            70.0,
            15.0,
            MUTED,
        );
        ui.text(
            d,
            if self.e.rescue.thermal_shield() {
                "Brasa 1/8 · Escudo térmico +15% turbo"
            } else {
                "Primer rescate · Brasa en Mercurio"
            },
            28.0,
            309.0,
            18.0,
            GOLD,
        );
        if self.overview {
            ui.panel(d, Rectangle::new(w - 288.0, 118.0, 262.0, 448.0));
            ui.text(d, "Elegir destino", w - 267.0, 135.0, 25.0, CREAM);
            ui.text(
                d,
                if pad {
                    "L1 / R1 para seleccionar"
                } else {
                    "[ / ] para seleccionar"
                },
                w - 267.0,
                171.0,
                16.0,
                MUTED,
            );
            for i in 0..9 {
                let y = 208.0 + i as f32 * 36.0;
                if world.selected == i {
                    d.draw_rectangle_rounded(
                        Rectangle::new(w - 276.0, y - 4.0, 238.0, 32.0),
                        0.4,
                        8,
                        Color::new(40, 79, 76, 255),
                    );
                }
                ui.text(
                    d,
                    &format!("{}  {}", i + 1, pretty(destination_name(i))),
                    w - 263.0,
                    y,
                    20.0,
                    if world.selected == i { MINT } else { CREAM },
                );
            }
            ui.prompt(
                d,
                "Trazar ruta",
                Button::Triangle,
                pad,
                "F",
                Vector2::new(w - 157.0, 582.0),
            );
            let detail = if world.selected == 8 {
                "Luna · Base Nenúfar"
            } else {
                PLANETS[world.selected].subtitle
            };
            for (i, line) in ui.wrap(detail, 17.0, 238.0).iter().enumerate() {
                ui.text(d, line, w - 267.0, 648.0 + i as f32 * 22.0, 17.0, MUTED);
            }
        } else {
            let center = Vector2::new(w - 114.0, h - 135.0);
            let scale = 0.24;
            ui.panel(d, Rectangle::new(w - 212.0, h - 239.0, 186.0, 207.0));
            ui.text(d, "Sistema solar", w - 193.0, h - 225.0, 17.0, MUTED);
            for planet in PLANETS {
                d.draw_circle_lines_v(center, planet.orbit * scale, Color::new(65, 102, 111, 140));
                let p = planet.position(world.time);
                d.draw_circle_v(center + Vector2::new(p.x, p.z) * scale, 3.0, planet.color);
            }
            d.draw_circle_v(center, 4.0, GOLD);
            let p = world.position;
            d.draw_circle_v(center + Vector2::new(p.x, p.z) * scale, 3.5, MINT);
            ui.button(d, Button::Square, pad, "Tab", w - 192.0, h - 76.0);
            ui.text(d, "Abrir mapa", w - 144.0, h - 70.0, 16.0, CREAM);
        }
        if self.e.can_land(world) {
            ui.prompt(
                d,
                if self.e.landing_zone(world) == Some(crate::expedition::Zone::Mercury) {
                    "Descender a Mercurio"
                } else {
                    "Descender a la Luna"
                },
                Button::Cross,
                pad,
                "E",
                Vector2::new(
                    w * 0.5,
                    if self.comms.active.is_some() {
                        h - 302.0
                    } else {
                        h - 100.0
                    },
                ),
            );
        } else if !self.overview {
            ui.prompt(
                d,
                "Trazar ruta",
                Button::Triangle,
                pad,
                "F",
                Vector2::new(
                    w * 0.5,
                    if self.comms.active.is_some() {
                        h - 302.0
                    } else {
                        h - 100.0
                    },
                ),
            );
        }
    }
    fn speed(&self, d: &mut RaylibDrawHandle, at: Vector2, speed: f32, thrust: f32) {
        let ui = self.ui;
        let color = if self.turbo { GOLD } else { MINT };
        ui.panel(d, Rectangle::new(at.x, at.y, 232.0, 83.0));
        ui.text(
            d,
            &format!("{speed:02.0}"),
            at.x + 18.0,
            at.y + 10.0,
            38.0,
            CREAM,
        );
        ui.text(d, "u/s", at.x + 83.0, at.y + 28.0, 17.0, MUTED);
        ui.text(
            d,
            if self.turbo { "Turbo" } else { "Impulso" },
            at.x + 137.0,
            at.y + 18.0,
            15.0,
            color,
        );
        d.draw_rectangle_rounded(
            Rectangle::new(at.x + 135.0, at.y + 45.0, 72.0, 7.0),
            0.9,
            8,
            Color::new(54, 77, 84, 255),
        );
        d.draw_rectangle_rounded(
            Rectangle::new(at.x + 135.0, at.y + 45.0, 72.0 * thrust.max(0.02), 7.0),
            0.9,
            8,
            color,
        );
    }
    fn caption(&self, d: &mut RaylibDrawHandle, w: f32, h: f32) {
        let Some(a) = &self.comms.active else {
            return;
        };
        let ui = self.ui;
        let width = (w - 295.0).min(920.0);
        let x = 28.0;
        let y = h - 237.0;
        ui.panel(d, Rectangle::new(x, y, width, 205.0));
        self.portraits.draw(
            d,
            Rectangle::new(x + 14.0, y + 31.0, 119.0, 137.0),
            a.message.speaker.portrait,
        );
        let tx = x + 153.0;
        let channel = if a.message.kind == Kind::Radio {
            "RADIO · RANA-56"
        } else {
            "COMUNICACIÓN DIRECTA"
        };
        ui.text(d, channel, tx, y + 13.0, 13.0, MINT);
        ui.text(d, a.message.speaker.name, tx, y + 33.0, 27.0, CREAM);
        ui.text(
            d,
            a.message.speaker.role,
            tx + ui.width(a.message.speaker.name, 27.0) + 18.0,
            y + 43.0,
            16.0,
            MUTED,
        );
        let full = ui.wrap(&a.message.text, 25.0, width - 183.0);
        let mut remain = a.visible().chars().count();
        for (i, line) in full.iter().take(3).enumerate() {
            let visible: String = line.chars().take(remain).collect();
            ui.text(d, &visible, tx, y + 73.0 + i as f32 * 29.0, 25.0, CREAM);
            remain = remain.saturating_sub(line.chars().count() + 1);
        }
        let (button, key) = if a.message.kind == Kind::Conversation {
            (Button::Cross, "Enter")
        } else {
            (Button::Down, "Enter")
        };
        let label = if !a.complete() {
            "Completar mensaje"
        } else if a.message.kind == Kind::Conversation {
            "Continuar"
        } else {
            "Cerrar radio"
        };
        let by = y + 168.0;
        let bw = ui.button(d, button, self.gamepad_hints, key, tx, by);
        ui.text(d, label, tx + bw + 11.0, by + 6.0, 16.0, MUTED);
        if self.comms.blocking() {
            ui.text(
                d,
                "Acción detenida",
                x + width - 145.0,
                y + 15.0,
                14.0,
                GOLD,
            );
        } else {
            for i in 0..3 {
                let pulse = (self.comms.time * 6.0 + i as f32).sin() * 0.5 + 0.5;
                d.draw_circle(
                    (x + width - 46.0 + i as f32 * 10.0) as i32,
                    (y + 23.0) as i32,
                    2.0 + pulse,
                    MINT,
                );
            }
        }
    }
    fn pause(&self, d: &mut RaylibDrawHandle, w: f32, h: f32) {
        let ui = self.ui;
        d.draw_rectangle(0, 0, w as i32, h as i32, Color::new(3, 11, 22, 190));
        let x = w * 0.5 - 294.0;
        let y = h * 0.5 - 291.0;
        ui.panel(d, Rectangle::new(x, y, 588.0, 582.0));
        leaf(d, Vector2::new(x + 43.0, y + 47.0), 13.0, MINT);
        ui.text(
            d,
            "Una pausa entre estrellas",
            x + 69.0,
            y + 27.0,
            29.0,
            CREAM,
        );
        ui.text(
            d,
            if self.audio_available {
                "Ajusta el sonido de tu expedición"
            } else {
                "Dispositivo de audio no disponible"
            },
            x + 29.0,
            y + 75.0,
            18.0,
            MUTED,
        );
        let labels = [
            "Continuar",
            if self.e.astronaut_visible() {
                "Reubicar a Luma"
            } else {
                "Reubicar nave"
            },
            "Controles y ayuda",
            "Música",
            "Efectos",
            if self.audio_settings.muted {
                "Sonido: silenciado"
            } else {
                "Sonido: activado"
            },
            "Salir del juego",
        ];
        for (i, label) in labels.iter().enumerate() {
            let yy = y + 123.0 + i as f32 * 49.0;
            if self.menu == i as i32 {
                d.draw_rectangle_rounded(
                    Rectangle::new(x + 20.0, yy - 7.0, 548.0, 43.0),
                    0.4,
                    8,
                    Color::new(39, 80, 74, 255),
                );
                leaf(d, Vector2::new(x + 42.0, yy + 13.0), 8.0, MINT);
            }
            ui.text(d, label, x + 63.0, yy, 23.0, CREAM);
            if i == 3 || i == 4 {
                let value = if i == 3 {
                    self.audio_settings.music
                } else {
                    self.audio_settings.effects
                };
                let color = if self.audio_settings.muted {
                    MUTED
                } else {
                    MINT
                };
                d.draw_rectangle_rounded(
                    Rectangle::new(x + 270.0, yy + 10.0, 187.0, 9.0),
                    0.9,
                    8,
                    Color::new(55, 73, 84, 255),
                );
                if value > 0.0 {
                    d.draw_rectangle_rounded(
                        Rectangle::new(x + 270.0, yy + 10.0, 187.0 * value, 9.0),
                        0.9,
                        8,
                        color,
                    );
                }
                d.draw_circle_v(
                    Vector2::new(x + 270.0 + 187.0 * value, yy + 14.5),
                    6.0,
                    color,
                );
                ui.text(
                    d,
                    &format!("{:3.0}%", value * 100.0),
                    x + 477.0,
                    yy + 1.0,
                    21.0,
                    CREAM,
                );
            }
        }
        ui.text(
            d,
            if self.gamepad_hints {
                "Cruceta izquierda / derecha · Volumen"
            } else {
                "Flechas izquierda / derecha · Volumen     M · Silenciar"
            },
            x + 29.0,
            y + 481.0,
            16.0,
            MUTED,
        );
        ui.button(
            d,
            Button::Cross,
            self.gamepad_hints,
            "Enter",
            x + 29.0,
            y + 525.0,
        );
        ui.text(d, "Elegir", x + 106.0, y + 530.0, 17.0, MUTED);
        ui.button(
            d,
            Button::Circle,
            self.gamepad_hints,
            "Esc",
            x + 330.0,
            y + 525.0,
        );
        ui.text(d, "Volver", x + 403.0, y + 530.0, 17.0, MUTED);
    }
    fn sheet(&self, d: &mut RaylibDrawHandle, w: f32, h: f32) {
        let ui = self.ui;
        d.draw_rectangle(0, 0, w as i32, h as i32, Color::new(3, 11, 22, 205));
        let x = w * 0.5 - 405.0;
        let y = h * 0.5 - 290.0;
        ui.panel(d, Rectangle::new(x, y, 810.0, 580.0));
        ui.text(
            d,
            if self.diagnostic {
                "Prueba de mando"
            } else {
                "Bitácora de la expedición"
            },
            x + 30.0,
            y + 25.0,
            32.0,
            CREAM,
        );
        if self.diagnostic {
            ui.text(d, &self.controls.name, x + 30.0, y + 77.0, 21.0, MINT);
            if let Some(id) = self.controls.pad {
                use GamepadAxis::*;
                use GamepadButton::*;
                for (i, (label, axis)) in [
                    ("Stick izquierdo X", GAMEPAD_AXIS_LEFT_X),
                    ("Stick izquierdo Y", GAMEPAD_AXIS_LEFT_Y),
                    ("Stick derecho X", GAMEPAD_AXIS_RIGHT_X),
                    ("Stick derecho Y", GAMEPAD_AXIS_RIGHT_Y),
                    ("L2", GAMEPAD_AXIS_LEFT_TRIGGER),
                    ("R2", GAMEPAD_AXIS_RIGHT_TRIGGER),
                ]
                .into_iter()
                .enumerate()
                {
                    let value = d.get_gamepad_axis_movement(id, axis);
                    let yy = y + 124.0 + i as f32 * 42.0;
                    ui.text(
                        d,
                        &format!("{label}   {value:+.2}"),
                        x + 30.0,
                        yy,
                        20.0,
                        CREAM,
                    );
                    d.draw_rectangle_rounded(
                        Rectangle::new(x + 357.0, yy + 4.0, 370.0, 14.0),
                        0.8,
                        8,
                        Color::new(48, 73, 82, 255),
                    );
                    if value > -0.99 {
                        d.draw_rectangle_rounded(
                            Rectangle::new(x + 357.0, yy + 4.0, (value + 1.0) * 185.0, 14.0),
                            0.8,
                            8,
                            MINT,
                        );
                    }
                }
                for (i, (button, b)) in [
                    (Button::Cross, GAMEPAD_BUTTON_RIGHT_FACE_DOWN),
                    (Button::Triangle, GAMEPAD_BUTTON_RIGHT_FACE_UP),
                    (Button::Circle, GAMEPAD_BUTTON_RIGHT_FACE_RIGHT),
                    (Button::Square, GAMEPAD_BUTTON_RIGHT_FACE_LEFT),
                ]
                .into_iter()
                .enumerate()
                {
                    let xx = x + 40.0 + i as f32 * 180.0;
                    ui.button(d, button, true, "", xx, y + 412.0);
                    ui.text(
                        d,
                        if d.is_gamepad_button_down(id, b) {
                            "Activo"
                        } else {
                            "En reposo"
                        },
                        xx + 40.0,
                        y + 417.0,
                        17.0,
                        MUTED,
                    );
                }
            } else {
                ui.text(
                    d,
                    "Conecta tu mando con un cable USB de datos.",
                    x + 30.0,
                    y + 145.0,
                    23.0,
                    CREAM,
                );
                ui.text(
                    d,
                    "La detección se actualiza sin reiniciar el juego.",
                    x + 30.0,
                    y + 188.0,
                    21.0,
                    MUTED,
                );
            }
        } else {
            let lines = [
                "1. Viaja a Mercurio: Triángulo / F. Desciende: X / E.",
                "2. Apaga los colectores (Triángulo/E); rescata a Brasa.",
                "3. Triángulo / E para abordar; después X / E para despegar.",
                "",
                "A pie: stick izquierdo / WASD. R2 / Shift para correr.",
                "Cámara: stick derecho / flechas. Salto: X / Espacio.",
                "Mantén el salto para cargarlo y alcanzar más altura.",
                "Refugios de techo azul: enfrían el traje en Mercurio.",
                "Diálogo: X / Enter. Radio: cruceta abajo / Enter.",
                "",
                "Vuelo: R2/L2 impulso, sticks orientar, R3 turbo.",
                "Círculo/C: interior. L3/X: frenar. Cuadrado/Tab: mapa.",
                "Pausa: volumen de música/efectos. M: silenciar audio.",
            ];
            for (i, line) in lines.iter().enumerate() {
                ui.text(
                    d,
                    line,
                    x + 30.0,
                    y + 91.0 + i as f32 * 31.0,
                    20.0,
                    if i < 3 { MINT } else { CREAM },
                );
            }
        }
        ui.text(
            d,
            if self.diagnostic {
                "Share / F3 o Círculo para volver"
            } else {
                "H / cruceta arriba o Círculo para volver"
            },
            x + 30.0,
            y + 537.0,
            18.0,
            MUTED,
        );
    }
}
fn leaf<D: RaylibDraw>(d: &mut D, c: Vector2, r: f32, color: Color) {
    d.draw_circle_v(c, r, color);
    d.draw_triangle(
        c,
        c + Vector2::new(r * 1.2, -r * 0.2),
        c + Vector2::new(r * 0.2, -r * 1.2),
        INK,
    );
}
