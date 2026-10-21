use crate::{
    expedition::{Expedition, Phase},
    simulation::*,
};
use std::collections::{HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Radio,
    Conversation,
}
#[derive(Clone, Copy)]
pub struct Speaker {
    pub name: &'static str,
    pub role: &'static str,
    pub portrait: Portrait,
}
#[derive(Clone, Copy)]
pub enum Portrait {
    Luma,
    Brasa,
    Terminal,
    Lirio,
}
pub const LUMA: Speaker = Speaker {
    name: "Luma",
    role: "Capitana · RANA-56",
    portrait: Portrait::Luma,
};
pub const BRASA: Speaker = Speaker {
    name: "Brasa",
    role: "Mecánica · Rana tomate",
    portrait: Portrait::Brasa,
};
pub const LIRIO: Speaker = Speaker {
    name: "Lirio",
    role: "Botánico · Rana dardo azul",
    portrait: Portrait::Lirio,
};
pub const BASE: Speaker = Speaker {
    name: "Nenúfar",
    role: "Terminal de exploración",
    portrait: Portrait::Terminal,
};
pub struct Message {
    pub speaker: Speaker,
    pub text: String,
    pub kind: Kind,
}
pub struct Active {
    pub message: Message,
    pub elapsed: f32,
    pub revealed: f32,
}
impl Active {
    pub fn complete(&self) -> bool {
        self.revealed as usize >= self.message.text.chars().count()
    }
    pub fn visible(&self) -> String {
        self.message
            .text
            .chars()
            .take(self.revealed as usize)
            .collect()
    }
}
#[derive(Default)]
pub struct Comms {
    pub active: Option<Active>,
    radio: VecDeque<Message>,
    conversation: VecDeque<Message>,
    suspended: Option<Active>,
    seen: HashSet<&'static str>,
    pub time: f32,
    pub toast_left: f32,
    pub toast: String,
    pub conversations_finished: u32,
    pub messages_shown: u32,
}
impl Comms {
    fn select_next(&mut self) {
        if self.active.is_some() {
            return;
        }
        if let Some(message) = self.conversation.pop_front() {
            self.active = Some(Active {
                message,
                elapsed: 0.0,
                revealed: 0.0,
            });
        } else if let Some(saved) = self.suspended.take() {
            self.active = Some(saved);
            return;
        } else if let Some(message) = self.radio.pop_front() {
            self.active = Some(Active {
                message,
                elapsed: 0.0,
                revealed: 0.0,
            });
        }
        if self.active.is_some() {
            self.messages_shown += 1;
        }
    }
    pub fn radio_once(&mut self, id: &'static str, text: impl Into<String>) {
        self.radio_from_once(id, LUMA, text);
    }
    pub fn radio_from_once(&mut self, id: &'static str, speaker: Speaker, text: impl Into<String>) {
        if !self.seen.insert(id) {
            return;
        }
        self.radio.push_back(Message {
            speaker,
            text: text.into(),
            kind: Kind::Radio,
        });
        self.select_next();
    }
    pub fn blocking(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|a| a.message.kind == Kind::Conversation)
            || !self.conversation.is_empty()
    }
    pub fn begin_conversation(&mut self) {
        self.begin_script(&[
            (LUMA,"Base Nenúfar, aquí Luma. ¿Reciben mi señal?"),
            (BASE,"Activa las tres balizas. Usa los escalones y un salto cargado para alcanzar la plataforma central."),
            (LUMA,"¡Entendido! Revisaré la señal y regresaré a la RANA-56."),
        ]);
    }
    pub fn begin_script(&mut self, script: &[(Speaker, &str)]) {
        if self.blocking() {
            return;
        }
        self.suspended = self.active.take();
        for &(speaker, text) in script {
            self.conversation.push_back(Message {
                speaker,
                text: text.into(),
                kind: Kind::Conversation,
            });
        }
        self.select_next();
    }
    pub fn reveal_all(&mut self) {
        if let Some(a) = &mut self.active {
            a.revealed = a.message.text.chars().count() as f32;
        }
    }
    pub fn advance(&mut self) {
        if self.active.as_ref().is_some_and(|a| !a.complete()) {
            self.reveal_all();
            return;
        }
        if self
            .active
            .take()
            .is_some_and(|a| a.message.kind == Kind::Conversation)
            && self.conversation.is_empty()
        {
            self.conversations_finished += 1;
        }
        self.select_next();
    }
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.toast_left = (self.toast_left - dt).max(0.0);
        self.select_next();
        if let Some(a) = &mut self.active {
            a.elapsed += dt;
            a.revealed = (a.revealed + dt * 35.0).min(a.message.text.chars().count() as f32);
            if a.message.kind == Kind::Radio
                && a.elapsed > a.message.text.chars().count() as f32 / 35.0 + 4.5
            {
                self.active = None;
                self.select_next();
            }
        }
    }
    fn observe_rescue(&mut self, e: &Expedition) {
        use crate::rescue::Status;
        if e.phase == Phase::OnFoot {
            self.radio_from_once("mercury_intro",BRASA,"¡Capitana! El refugio está bloqueado. Apaga los tres colectores. Bajo los techos azules puedes enfriar tu traje.");
        }
        for (i, done) in e.rescue.collectors.iter().enumerate() {
            if *done {
                self.radio_once(
                    ["solar1", "solar2", "solar3"][i],
                    format!(
                        "Colector {} apagado. {}/3.{}",
                        i + 1,
                        e.rescue.collectors.iter().filter(|b| **b).count(),
                        if e.rescue.unlocked() {
                            " ¡El refugio está abierto!"
                        } else {
                            ""
                        }
                    ),
                );
            }
        }
        if e.rescue.overheats > 0 {
            self.radio_once("overheat","El sistema de emergencia me devolvió a la nave. Usaré las sombras para enfriar el traje.");
        }
        if e.rescue.status == Status::Celebration && self.seen.insert("rescue_dialog") {
            self.begin_script(&[
                (
                    BRASA,
                    "¡Luma! Sabía que vendrías. Hasta una rana tomate necesita un poco de sombra.",
                ),
                (LUMA, "Ninguna rana se queda atrás. ¡Choca esa pata, Brasa!"),
                (
                    BRASA,
                    "Volvamos a la nave. Tengo un escudo térmico que instalar en la RANA-56.",
                ),
            ]);
        }
        if e.rescue.status == Status::Escort {
            self.radio_from_once("escort",BRASA,"¡Tripulación reunida! Iré al punto de abordaje. Nos vemos junto a la nave, capitana.");
        }
        if e.rescue.thermal_shield() {
            self.radio_from_once("rescued",BRASA,"¡De vuelta en casa! Escudo térmico instalado: el turbo de la RANA-56 ahora alcanza un 15 % más de velocidad.");
            if self.seen.insert("rescue_toast") {
                self.toast = "Brasa a salvo · Escudo térmico".into();
                self.toast_left = 3.5;
            }
        }
    }
    fn observe_venus(&mut self, e: &Expedition) {
        use crate::rescue::Status;
        if e.phase == Phase::OnFoot && e.venus.status == Status::Waiting {
            self.radio_from_once("venus_intro", LIRIO, "¡Capitana! Estoy en el refugio del jardín. Recupera las tres semillas de luz de los invernaderos y podremos volver a casa.");
            if e.astronaut.charging {
                self.radio_from_once(
                    "venus_jump",
                    LUMA,
                    "Un salto cargado y esos escalones me llevarán hasta las semillas.",
                );
            }
            for (i, &found) in e.venus.seeds.iter().enumerate() {
                if found && self.seen.insert(["seed1", "seed2", "seed3"][i]) {
                    self.toast = format!("Semilla de luz {} recuperada", i + 1);
                    self.toast_left = 3.5;
                }
            }
            if e.venus.unlocked() {
                self.radio_from_once("venus_ready", LIRIO, "¡Las tres! Ven al refugio, al fondo del jardín. Tengo muchas ganas de ver a la tripulación.");
            }
        }
        if e.venus.status == Status::Celebration && self.seen.insert("venus_greeting") {
            self.begin_script(&[
                (
                    LIRIO,
                    "¡Luma! Pensé que iba a tener que cultivar una nave para salir de aquí.",
                ),
                (
                    LUMA,
                    "Tu lugar sigue esperándote a bordo, Lirio. ¡Choca esa pata!",
                ),
                (LIRIO, "¡Voy! Aunque tenga que saltar un poquito."),
            ]);
        }
        if e.venus.status == Status::Escort {
            self.radio_from_once(
                "venus_escort",
                LIRIO,
                "Te sigo hasta la nave. ¡Estas semillas harán florecer nuestro jardín!",
            );
        }
        if e.venus.complete() {
            self.radio_from_once(
                "venus_home",
                LIRIO,
                "¡En casa! Ven a visitarme: ya estoy cuidando las plantas de nuestra nave.",
            );
        }
    }
    pub fn observe(&mut self, e: &Expedition, before: [bool; 3]) {
        if e.zone == crate::expedition::Zone::Venus {
            self.observe_venus(e);
            return;
        }
        if e.zone == crate::expedition::Zone::Mercury {
            self.observe_rescue(e);
            return;
        }
        if e.phase == Phase::OnFoot {
            self.radio_once(
                "boots",
                "¡Botas en la Luna! Primero recuperaré la señal de estas tres balizas.",
            );
            if length(e.astronaut.position - v(0.0, 0.0, -18.0)) < 16.0 || e.astronaut.charging {
                self.radio_once(
                    "jump",
                    "Esa plataforma está alta. Cargaré el salto y aprovecharé los escalones.",
                );
            }
        }
        for (i, (&was, &now)) in before.iter().zip(e.beacons.iter()).enumerate() {
            if !was && now {
                self.toast = format!("Baliza {} conectada", i + 1);
                self.toast_left = 3.5;
                let id = ["beacon1", "beacon2", "beacon3"][i];
                let count = e.beacons.iter().filter(|&&b| b).count();
                if count == 3 {
                    self.radio_once("all","¡Señal recuperada! Las tres balizas están listas. Es hora de volver a la nave.");
                } else {
                    self.radio_once(
                        id,
                        format!(
                            "¡Baliza {} conectada! Ya recuperé {count} de las tres señales.",
                            i + 1
                        ),
                    );
                }
            }
        }
        if e.completed {
            self.radio_once("complete","Prueba lunar completada. RANA-56, listos para seguir buscando a nuestra tripulación.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_reveal_and_single_delivery_are_safe() {
        let mut c = Comms::default();
        c.radio_once("test", "¡Señal recuperada, capitán! 🐸");
        c.radio_once("test", "duplicado");
        c.update(0.08);
        assert_eq!(c.active.as_ref().unwrap().visible(), "¡S");
        c.advance();
        assert!(c.active.as_ref().unwrap().complete());
        assert!(c.active.is_some());
        c.advance();
        assert!(c.active.is_none());
        assert_eq!(c.messages_shown, 1);
    }
    #[test]
    fn conversation_pauses_and_restores_radio_without_losing_progress() {
        let mut c = Comms::default();
        c.radio_once("first", "Mensaje de radio en progreso");
        c.update(0.15);
        let visible = c.active.as_ref().unwrap().visible();
        c.begin_conversation();
        assert!(c.blocking());
        c.radio_once("later", "Otro mensaje");
        for _ in 0..3 {
            c.advance();
            assert!(c.active.as_ref().unwrap().complete());
            c.advance();
        }
        assert!(!c.blocking());
        assert_eq!(c.conversations_finished, 1);
        assert_eq!(c.active.as_ref().unwrap().visible(), visible);
        c.update(20.0);
        assert_eq!(c.active.as_ref().unwrap().message.text, "Otro mensaje");
    }
    #[test]
    fn messages_do_not_repeat_when_revisiting_or_restart() {
        let mut c = Comms::default();
        let mut e = Expedition {
            phase: Phase::OnFoot,
            ..Default::default()
        };
        c.observe(&e, [false; 3]);
        c.observe(&e, [false; 3]);
        assert_eq!(c.messages_shown, 1);
        e.beacons = [true; 3];
        c.observe(&e, [false; 3]);
        let count = c.radio.len();
        c.observe(&e, [true; 3]);
        assert_eq!(c.radio.len(), count);
        assert!(c.toast_left > 0.0);
    }
}
