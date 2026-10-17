use crate::{
    astronaut::Astronaut,
    comms::{Comms, Portrait},
    frog::{self, Pose, Species},
    scene::Assets,
    simulation::*,
};
use raylib::prelude::*;

// Camaras reales sobre el mismo rig que se dibuja en el escenario.
// Las texturas se crean una vez y se reutilizan; no hay retratos bitmap.
pub struct Portraits {
    targets: [RenderTexture2D; 3],
}
impl Portraits {
    pub fn load(rl: &mut RaylibHandle, t: &RaylibThread) -> Result<Self, String> {
        let mut make = || {
            rl.load_render_texture(t, 256, 288)
                .map_err(|e| e.to_string())
        };
        let mut targets = [make()?, make()?, make()?];
        for target in &mut targets {
            target.set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
        }
        Ok(Self { targets })
    }
    pub fn update(
        &mut self,
        rl: &mut RaylibHandle,
        t: &RaylibThread,
        assets: &mut Assets,
        c: &Comms,
    ) {
        for (i, target) in self.targets.iter_mut().enumerate() {
            let visible = c
                .active
                .as_ref()
                .is_some_and(|a| a.message.speaker.portrait as usize == i);
            if !visible && !(i == 0 && c.toast_left > 0.0) {
                continue;
            }
            let talking = visible && c.active.as_ref().is_some_and(|a| !a.complete());
            let camera = Camera3D::perspective(
                v(0.45, 2.38, 4.6),
                v(0.0, 2.03, 0.0),
                v(0.0, 1.0, 0.0),
                35.0,
            );
            assets.prepare(camera, c.time);
            assets.kind(11);
            let mut d = rl.begin_texture_mode(t, target);
            d.clear_background(if i == 1 {
                Color::new(69, 40, 38, 255)
            } else {
                Color::new(27, 56, 66, 255)
            });
            let mut s = d.begin_mode3D(camera);
            if i == 2 {
                s.draw_cube(
                    v(0.0, 2.0, 0.0),
                    1.65,
                    1.8,
                    0.8,
                    Color::new(47, 83, 99, 255),
                );
                s.draw_cube(
                    v(0.0, 2.1, 0.42),
                    1.38,
                    1.12,
                    0.08,
                    Color::new(95, 230, 183, 255),
                );
                s.draw_cylinder_ex(
                    v(0.0, 2.9, 0.0),
                    v(0.0, 3.25, 0.0),
                    0.06,
                    0.06,
                    8,
                    Color::GRAY,
                );
                s.draw_sphere_ex(v(0.0, 3.28, 0.0), 0.12, 8, 10, Color::GOLD);
                for x in [-0.3, 0.3] {
                    s.draw_cube(
                        v(x, 2.25, 0.48),
                        0.15,
                        0.21,
                        0.03,
                        Color::new(22, 60, 65, 255),
                    );
                }
            } else {
                let mut a = Astronaut::default();
                a.position = Vector3::zero();
                a.time = c.time;
                a.yaw = (c.time * 1.3).sin() * 0.08;
                frog::draw_actor(
                    &mut s,
                    &assets.frog_mesh,
                    &a,
                    if i == 0 {
                        Species::Tree
                    } else {
                        Species::Tomato
                    },
                    &Pose {
                        talking,
                        high_five: 0.0,
                    },
                );
            }
        }
    }
    pub fn draw<D: RaylibDraw>(&self, d: &mut D, r: Rectangle, p: Portrait) {
        d.draw_texture_pro(
            self.targets[p as usize].texture(),
            Rectangle::new(0.0, 0.0, 256.0, -288.0),
            r,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        d.draw_rectangle_lines_ex(r, 1.0, Color::new(101, 168, 167, 180));
    }
}
