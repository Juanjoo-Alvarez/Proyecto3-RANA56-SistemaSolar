use raylib::prelude::*;
use std::path::Path;

pub const MINT: Color = Color::new(137, 232, 189, 255);
pub const CREAM: Color = Color::new(248, 244, 220, 255);
pub const MUTED: Color = Color::new(168, 190, 193, 255);
pub const GOLD: Color = Color::new(255, 205, 113, 255);
pub const INK: Color = Color::new(14, 30, 44, 235);
pub struct Ui {
    pub font: Font,
    title: Font,
}
impl Ui {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path) -> Result<Self, String> {
        let glyphs: String = (32..=126)
            .chain(160..=255)
            .filter_map(char::from_u32)
            .collect();
        let mut load = |name: &str| -> Result<Font, String> {
            let font = rl
                .load_font_ex(
                    thread,
                    &root.join("assets/fonts").join(name).to_string_lossy(),
                    52,
                    Some(&glyphs),
                )
                .map_err(|e| e.to_string())?;
            font.texture()
                .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            Ok(font)
        };
        let font = load("Nunito-SemiBold.ttf")?;
        let title = load("Nunito-ExtraBold.ttf")?;
        Ok(Self { font, title })
    }
    pub fn width(&self, text: &str, size: f32) -> f32 {
        self.face(size).measure_text(text, size, 0.3).x
    }
    fn face(&self, size: f32) -> &Font {
        if size >= 26.0 {
            &self.title
        } else {
            &self.font
        }
    }
    pub fn text<D: RaylibDraw>(
        &self,
        d: &mut D,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) {
        d.draw_text_ex(
            self.face(size),
            text,
            Vector2::new(x + 1.0, y + 1.5),
            size,
            0.3,
            Color::new(0, 6, 12, 100),
        );
        d.draw_text_ex(self.face(size), text, Vector2::new(x, y), size, 0.3, color);
    }
    pub fn wrap(&self, text: &str, size: f32, width: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current = String::new();
        for word in text.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if self.width(&candidate, size) > width && !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
        if !current.is_empty() {
            lines.push(current);
        }
        lines
    }
    pub fn panel<D: RaylibDraw>(&self, d: &mut D, r: Rectangle) {
        d.draw_rectangle_rounded(
            Rectangle::new(r.x, r.y + 4.0, r.width, r.height),
            0.18,
            10,
            Color::new(0, 5, 14, 65),
        );
        d.draw_rectangle_rounded(r, 0.18, 10, INK);
        d.draw_rectangle_rounded_lines_ex(r, 0.18, 10, 1.0, Color::new(117, 170, 163, 75));
    }
    pub fn pill<D: RaylibDraw>(&self, d: &mut D, text: &str, x: f32, y: f32, color: Color) {
        let width = self.width(text, 17.0) + 24.0;
        d.draw_rectangle_rounded(
            Rectangle::new(x, y, width, 30.0),
            0.9,
            12,
            Color::new(color.r, color.g, color.b, 25),
        );
        self.text(d, text, x + 12.0, y + 5.0, 17.0, color);
    }
    pub fn button<D: RaylibDraw>(
        &self,
        d: &mut D,
        kind: Button,
        pad: bool,
        key: &str,
        x: f32,
        y: f32,
    ) -> f32 {
        if !pad {
            let w = self.width(key, 18.0).max(14.0) + 18.0;
            d.draw_rectangle_rounded(
                Rectangle::new(x, y, w, 30.0),
                0.35,
                8,
                Color::new(58, 81, 89, 255),
            );
            self.text(d, key, x + 9.0, y + 4.0, 18.0, CREAM);
            return w;
        }
        let c = Vector2::new(x + 15.0, y + 15.0);
        d.draw_circle_v(c, 15.0, Color::new(50, 74, 84, 255));
        match kind {
            Button::Cross => {
                for side in [-1.0, 1.0] {
                    d.draw_line_ex(
                        c + Vector2::new(-6.0, side * 6.0),
                        c + Vector2::new(6.0, -side * 6.0),
                        2.3,
                        Color::new(142, 192, 245, 255),
                    );
                }
            }
            Button::Triangle => {
                let p = [
                    c + Vector2::new(0.0, -7.0),
                    c + Vector2::new(7.0, 6.0),
                    c + Vector2::new(-7.0, 6.0),
                ];
                for i in 0..3 {
                    d.draw_line_ex(p[i], p[(i + 1) % 3], 2.0, MINT);
                }
            }
            Button::Square => d.draw_rectangle_lines_ex(
                Rectangle::new(x + 8.0, y + 8.0, 14.0, 14.0),
                2.0,
                Color::new(230, 159, 207, 255),
            ),
            Button::Circle => d.draw_circle_lines_v(c, 7.0, Color::new(247, 155, 133, 255)),
            Button::Down => {
                d.draw_line_ex(
                    c - Vector2::new(0.0, 7.0),
                    c + Vector2::new(0.0, 6.0),
                    2.0,
                    CREAM,
                );
                d.draw_triangle(
                    c + Vector2::new(-5.0, 2.0),
                    c + Vector2::new(0.0, 8.0),
                    c + Vector2::new(5.0, 2.0),
                    CREAM,
                );
            }
        }
        30.0
    }
    pub fn prompt<D: RaylibDraw>(
        &self,
        d: &mut D,
        label: &str,
        kind: Button,
        pad: bool,
        key: &str,
        at: Vector2,
    ) {
        let (cx, y) = (at.x, at.y);
        let button_width = if pad {
            30.0
        } else {
            self.width(key, 18.0).max(14.0) + 18.0
        };
        let width = button_width + self.width(label, 21.0) + 48.0;
        let x = cx - width * 0.5;
        self.panel(d, Rectangle::new(x, y, width, 50.0));
        self.button(d, kind, pad, key, x + 12.0, y + 10.0);
        self.text(d, label, x + button_width + 24.0, y + 12.0, 21.0, CREAM);
    }
}
#[derive(Clone, Copy)]
pub enum Button {
    Cross,
    Triangle,
    Square,
    Circle,
    Down,
}
