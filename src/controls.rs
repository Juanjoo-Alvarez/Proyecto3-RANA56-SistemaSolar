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

pub fn pad_heading(left: (f32, f32), right: (f32, f32)) -> Input {
    let (lx, ly) = stick(left.0, left.1);
    let (rx, ry) = stick(right.0, right.1);
    Input {
        steer_yaw: -lx * lx.abs(),
        steer_pitch: -ly * ly.abs(),
        yaw: -rx,
        pitch: -ry,
        ..Default::default()
    }
}

#[derive(Default)]
pub struct Controls {
    pub pad: Option<i32>,
    pub name: String,
    pub disconnected: bool,
    pub using_gamepad: bool,
}
#[derive(Default)]
pub struct Actions {
    pub flight: Input,
    pub walk: crate::astronaut::WalkInput,
    pub foot_interact: bool,
    pub disembark: bool,
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
    pub radio_advance: bool,
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
        let heading = pad_heading(
            (analog(GAMEPAD_AXIS_LEFT_X), analog(GAMEPAD_AXIS_LEFT_Y)),
            (analog(GAMEPAD_AXIS_RIGHT_X), analog(GAMEPAD_AXIS_RIGHT_Y)),
        );
        let (walk_x, walk_y) = stick(analog(GAMEPAD_AXIS_LEFT_X), analog(GAMEPAD_AXIS_LEFT_Y));
        let thrust = self.pad.map_or(0.0, |_| {
            trigger(analog(GAMEPAD_AXIS_RIGHT_TRIGGER)) - trigger(analog(GAMEPAD_AXIS_LEFT_TRIGGER))
        });
        let keys = [
            KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR, KEY_FIVE, KEY_SIX, KEY_SEVEN, KEY_EIGHT,
            KEY_NINE,
        ];
        let pad_activity = self.pad.is_some()
            && (old != self.pad
                || walk_x != 0.0
                || walk_y != 0.0
                || heading.yaw != 0.0
                || heading.pitch != 0.0
                || trigger(analog(GAMEPAD_AXIS_LEFT_TRIGGER))
                    + trigger(analog(GAMEPAD_AXIS_RIGHT_TRIGGER))
                    > 0.05
                || [
                    GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
                    GAMEPAD_BUTTON_RIGHT_FACE_UP,
                    GAMEPAD_BUTTON_RIGHT_FACE_LEFT,
                    GAMEPAD_BUTTON_RIGHT_FACE_RIGHT,
                    GAMEPAD_BUTTON_MIDDLE_RIGHT,
                    GAMEPAD_BUTTON_MIDDLE_LEFT,
                    GAMEPAD_BUTTON_LEFT_FACE_UP,
                    GAMEPAD_BUTTON_LEFT_FACE_DOWN,
                    GAMEPAD_BUTTON_LEFT_TRIGGER_1,
                    GAMEPAD_BUTTON_RIGHT_TRIGGER_1,
                    GAMEPAD_BUTTON_RIGHT_THUMB,
                ]
                .into_iter()
                .any(pressed));
        if pad_activity {
            self.using_gamepad = true;
        }
        if self.pad.is_none()
            || [
                KEY_W,
                KEY_A,
                KEY_S,
                KEY_D,
                KEY_E,
                KEY_Q,
                KEY_F,
                KEY_H,
                KEY_TAB,
                KEY_SPACE,
                KEY_ENTER,
                KEY_ESCAPE,
                KEY_LEFT,
                KEY_RIGHT,
                KEY_UP,
                KEY_DOWN,
                KEY_R,
                KEY_P,
                KEY_O,
                KEY_X,
                KEY_F3,
                KEY_F12,
                KEY_LEFT_SHIFT,
                KEY_LEFT_CONTROL,
                KEY_LEFT_BRACKET,
                KEY_RIGHT_BRACKET,
                KEY_BACKSPACE,
            ]
            .into_iter()
            .any(|k| rl.is_key_pressed(k))
            || keys.into_iter().any(|k| rl.is_key_pressed(k))
        {
            self.using_gamepad = false;
        }
        Actions {
            walk: crate::astronaut::WalkInput {
                horizontal: (axis(rl, KEY_D, KEY_A) + walk_x).clamp(-1.0, 1.0),
                forward: (axis(rl, KEY_W, KEY_S) - walk_y).clamp(-1.0, 1.0),
                look_x: (axis(rl, KEY_LEFT, KEY_RIGHT) + heading.yaw).clamp(-1.0, 1.0),
                look_y: (axis(rl, KEY_UP, KEY_DOWN) + heading.pitch).clamp(-1.0, 1.0),
                jump_pressed: rl.is_key_pressed(KEY_SPACE)
                    || pressed(GAMEPAD_BUTTON_RIGHT_FACE_DOWN),
                jump_held: rl.is_key_down(KEY_SPACE) || down(GAMEPAD_BUTTON_RIGHT_FACE_DOWN),
                run: rl.is_key_down(KEY_LEFT_SHIFT)
                    || (self.pad.is_some() && trigger(analog(GAMEPAD_AXIS_RIGHT_TRIGGER)) > 0.25),
            },
            foot_interact: rl.is_key_pressed(KEY_E) || pressed(GAMEPAD_BUTTON_RIGHT_FACE_UP),
            disembark: rl.is_key_pressed(KEY_Q) || pressed(GAMEPAD_BUTTON_RIGHT_FACE_LEFT),
            flight: Input {
                thrust: (axis(rl, KEY_W, KEY_S) + thrust).clamp(-1.0, 1.0),
                strafe: axis(rl, KEY_D, KEY_A),
                vertical: axis(rl, KEY_SPACE, KEY_LEFT_CONTROL),
                yaw: (axis(rl, KEY_LEFT, KEY_RIGHT) + heading.yaw).clamp(-1.0, 1.0),
                pitch: (axis(rl, KEY_UP, KEY_DOWN) + heading.pitch).clamp(-1.0, 1.0),
                steer_yaw: heading.steer_yaw,
                steer_pitch: heading.steer_pitch,
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
            radio_advance: rl.is_key_pressed(KEY_ENTER) || pressed(GAMEPAD_BUTTON_LEFT_FACE_DOWN),
            back: rl.is_key_pressed(KEY_BACKSPACE) || pressed(GAMEPAD_BUTTON_RIGHT_FACE_RIGHT),
            direct: keys.into_iter().position(|key| rl.is_key_pressed(key)),
        }
    }
    pub fn pad_hints(&self) -> bool {
        self.using_gamepad && self.pad.is_some()
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
    use crate::{
        expedition::{Expedition, Phase},
        simulation::{World, length, v},
    };
    #[test]
    fn left_stick_aims_without_translation_then_trigger_follows_heading() {
        let aim = pad_heading((0.0, -1.0), (0.0, 0.0));
        assert_eq!((aim.thrust, aim.strafe, aim.vertical), (0.0, 0.0, 0.0));
        let mut world = World {
            position: v(100.0, 50.0, 0.0),
            ..Default::default()
        };
        let start = world.position;
        for _ in 0..120 {
            world.update(aim, 1.0 / 60.0);
        }
        assert_eq!(world.position, start);
        assert_eq!(length(world.velocity), 0.0);
        assert!(world.pitch > 0.9);
        let accelerated = Input { thrust: 1.0, ..aim };
        for _ in 0..60 {
            world.update(accelerated, 1.0 / 60.0);
        }
        assert!(world.position.y > start.y + 5.0);
        let mut e = Expedition {
            phase: Phase::Surface,
            ..Default::default()
        };
        let lunar_start = e.local.position;
        for _ in 0..120 {
            e.update(
                &mut world,
                aim,
                crate::astronaut::WalkInput::default(),
                1.0 / 60.0,
            );
        }
        assert_eq!(e.local.position, lunar_start);
        assert!(e.local.pitch > 0.5);
        for _ in 0..60 {
            e.update(
                &mut world,
                accelerated,
                crate::astronaut::WalkInput::default(),
                1.0 / 60.0,
            );
        }
        assert!(e.local.position.y > lunar_start.y + 2.0);
    }
    #[test]
    fn left_turn_is_smoothed_and_right_stick_keeps_direct_response() {
        let mut world = World::default();
        let left = pad_heading((0.0, -1.0), (0.0, 0.0));
        world.steer(left, 1.0 / 60.0, 1.3, 1.15);
        assert!(world.pitch > 0.0 && world.pitch < 0.005);
        let mut right_world = World::default();
        let right = pad_heading((0.0, 0.0), (1.0, -1.0));
        let expected = stick(1.0, -1.0);
        assert_eq!((right.yaw, right.pitch), (-expected.0, -expected.1));
        right_world.steer(right, 1.0 / 60.0, 1.3, 1.15);
        assert!((right_world.pitch - right.pitch / 60.0).abs() < 0.00001);
    }
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
