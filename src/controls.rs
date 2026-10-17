use crate::simulation::Input;
use raylib::prelude::*;

pub const DEAD_ZONE: f32 = 0.18;

// Zona muerta radial: conserva direccion y devuelve todo el recorrido restante.
pub fn stick(x: f32, y: f32) -> (f32, f32) {
    let magnitude = (x * x + y * y).sqrt();
    if magnitude <= DEAD_ZONE {
        return (0.0, 0.0);
    }
    let scale = ((magnitude.min(1.0) - DEAD_ZONE) / (1.0 - DEAD_ZONE)) / magnitude;
    (x * scale, y * scale)
}
pub fn trigger(value: f32) -> f32 {
    (((value + 1.0) * 0.5 - 0.05) / 0.95).clamp(0.0, 1.0)
}

#[derive(Default)]
pub struct Controls {
    pub pad: Option<i32>,
    pub name: String,
    pub disconnected: bool,
}
#[derive(Default)]
pub struct Actions {
    pub flight: Input,
    pub interact: bool,
    pub route: bool,
    pub map: bool,
    pub pause: bool,
    pub help: bool,
    pub diagnostic: bool,
    pub reset: bool,
    pub next: i32,
    pub menu_step: i32,
    pub confirm: bool,
    pub back: bool,
    pub direct: Option<usize>,
}
fn axis(rl: &RaylibHandle, positive: KeyboardKey, negative: KeyboardKey) -> f32 {
    i32::from(rl.is_key_down(positive)) as f32 - i32::from(rl.is_key_down(negative)) as f32
}
impl Controls {
    pub fn read(&mut self, rl: &RaylibHandle) -> Actions {
        use GamepadAxis::*;
        use GamepadButton::*;
        use KeyboardKey::*;
        let old = self.pad;
        self.pad = self
            .pad
            .filter(|&id| rl.is_gamepad_available(id))
            .or_else(|| (0..4).find(|&id| rl.is_gamepad_available(id)));
        self.disconnected = old.is_some() && self.pad != old;
        self.name = self
            .pad
            .and_then(|id| rl.get_gamepad_name(id))
            .unwrap_or_else(|| "Teclado / mando no detectado".into());
        let down = |button| {
            self.pad
                .is_some_and(|id| rl.is_gamepad_button_down(id, button))
        };
        let pressed = |button| {
            self.pad
                .is_some_and(|id| rl.is_gamepad_button_pressed(id, button))
        };
        let analog = |a| {
            self.pad
                .map_or(0.0, |id| rl.get_gamepad_axis_movement(id, a))
        };
        let (lx, ly) = stick(analog(GAMEPAD_AXIS_LEFT_X), analog(GAMEPAD_AXIS_LEFT_Y));
        let (rx, ry) = stick(analog(GAMEPAD_AXIS_RIGHT_X), analog(GAMEPAD_AXIS_RIGHT_Y));
        let thrust = self.pad.map_or(0.0, |_| {
            trigger(analog(GAMEPAD_AXIS_RIGHT_TRIGGER)) - trigger(analog(GAMEPAD_AXIS_LEFT_TRIGGER))
        });
        let keys = [
            KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR, KEY_FIVE, KEY_SIX, KEY_SEVEN, KEY_EIGHT,
            KEY_NINE,
        ];
        Actions {
            flight: Input {
                thrust: (axis(rl, KEY_W, KEY_S) + thrust).clamp(-1.0, 1.0),
                strafe: (axis(rl, KEY_D, KEY_A) + lx).clamp(-1.0, 1.0),
                vertical: (axis(rl, KEY_SPACE, KEY_LEFT_CONTROL) - ly).clamp(-1.0, 1.0),
                yaw: (axis(rl, KEY_LEFT, KEY_RIGHT) - rx).clamp(-1.0, 1.0),
                pitch: (axis(rl, KEY_UP, KEY_DOWN) - ry).clamp(-1.0, 1.0),
                boost: rl.is_key_down(KEY_LEFT_SHIFT) || down(GAMEPAD_BUTTON_RIGHT_THUMB),
                brake: rl.is_key_down(KEY_X) || down(GAMEPAD_BUTTON_RIGHT_FACE_RIGHT),
            },
            interact: rl.is_key_pressed(KEY_E) || pressed(GAMEPAD_BUTTON_RIGHT_FACE_DOWN),
            route: rl.is_key_pressed(KEY_F) || pressed(GAMEPAD_BUTTON_RIGHT_FACE_UP),
            map: rl.is_key_pressed(KEY_TAB) || pressed(GAMEPAD_BUTTON_RIGHT_FACE_LEFT),
            pause: rl.is_key_pressed(KEY_ESCAPE)
                || rl.is_key_pressed(KEY_P)
                || pressed(GAMEPAD_BUTTON_MIDDLE_RIGHT),
            help: rl.is_key_pressed(KEY_H) || pressed(GAMEPAD_BUTTON_LEFT_FACE_UP),
            diagnostic: rl.is_key_pressed(KEY_F3) || pressed(GAMEPAD_BUTTON_MIDDLE_LEFT),
            reset: rl.is_key_pressed(KEY_R),
            next: i32::from(
                pressed(GAMEPAD_BUTTON_RIGHT_TRIGGER_1) || rl.is_key_pressed(KEY_RIGHT_BRACKET),
            ) - i32::from(
                pressed(GAMEPAD_BUTTON_LEFT_TRIGGER_1) || rl.is_key_pressed(KEY_LEFT_BRACKET),
            ),
            menu_step: i32::from(
                pressed(GAMEPAD_BUTTON_LEFT_FACE_DOWN) || rl.is_key_pressed(KEY_DOWN),
            ) - i32::from(
                pressed(GAMEPAD_BUTTON_LEFT_FACE_UP) || rl.is_key_pressed(KEY_UP),
            ),
            confirm: rl.is_key_pressed(KEY_ENTER) || pressed(GAMEPAD_BUTTON_RIGHT_FACE_DOWN),
            back: rl.is_key_pressed(KEY_BACKSPACE) || pressed(GAMEPAD_BUTTON_RIGHT_FACE_RIGHT),
            direct: keys.into_iter().position(|key| rl.is_key_pressed(key)),
        }
    }
    pub fn report(&self, rl: &RaylibHandle) {
        println!("Mando: {}", self.name);
        if let Some(id) = self.pad {
            println!("Indice: {id}; ejes: {}", rl.get_gamepad_axis_count(id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drift_and_trigger_rest_do_not_accelerate_ship() {
        assert_eq!(stick(0.10, -0.12), (0.0, 0.0));
        assert_eq!(trigger(-1.0), 0.0);
        assert_eq!(trigger(-0.95), 0.0);
        assert_eq!(trigger(1.0), 1.0);
        assert_eq!(stick(1.0, 0.0), (1.0, 0.0));
        let (x, y) = stick(1.0, 1.0);
        assert!((x * x + y * y - 1.0).abs() < 0.0001);
        assert_eq!(stick(0.0, 0.0), (0.0, 0.0));
    }
}
