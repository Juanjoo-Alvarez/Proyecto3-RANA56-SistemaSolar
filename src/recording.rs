//! Reproducible flight through the ordinary input/physics path; no in-flight teleports.
use crate::simulation::*;
use std::{
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
};
pub const FPS: u32 = 30;
pub const FRAMES: u32 = FPS * 48;
fn waypoint(t: f32) -> crate::math::Vector3 {
    let smooth = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    let radius = if t < 12.0 {
        45.0 - 31.0 * smooth(t / 12.0)
    } else {
        14.0 + 46.0 * smooth((t - 36.0) / 12.0)
    };
    let angle = std::f32::consts::TAU * smooth((t - 12.0) / 24.0);
    PLANETS[2].position(t) + v(radius * angle.sin(), 4.0, radius * angle.cos())
}
pub fn setup(w: &mut World) {
    w.selected = 2;
    w.position = waypoint(0.0);
    w.yaw = std::f32::consts::PI;
    w.pitch = (-4.0f32 / 45.0).atan();
    w.autopilot = None;
    w.message = "Recorrido automatico de demostracion".into();
}
pub fn input(w: &World) -> Input {
    let dt = 1.0 / FPS as f32;
    let desired =
        (waypoint(w.time + dt) - waypoint(w.time)) / dt + (waypoint(w.time) - w.position) * 2.0;
    let gaze = PLANETS[2].position(w.time) - w.position;
    let yaw = gaze.x.atan2(gaze.z);
    let error = (yaw - w.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let pitch = (gaze.y / length(gaze)).asin();
    let thrust = dot(desired, w.forward()) / 30.0;
    Input {
        thrust,
        strafe: dot(desired, w.right()) / 30.0,
        vertical: (desired.y - thrust * 30.0 * w.forward().y) / 30.0,
        yaw: (error * 3.0 / 1.3).clamp(-1.0, 1.0),
        pitch: ((pitch - w.pitch) * 3.0).clamp(-1.0, 1.0),
        ..Input::default()
    }
}
pub fn caption(t: f32) -> &'static str {
    if t < 12.0 {
        "1 / VUELO Y ACERCAMIENTO A LA TIERRA"
    } else if t < 36.0 {
        "2 / VUELTA COMPLETA ALREDEDOR DEL PLANETA"
    } else {
        "3 / ALEJAMIENTO - SIN CAMBIOS DE ESCENA"
    }
}
pub struct Recorder {
    child: Child,
}
impl Recorder {
    pub fn new(path: &Path, width: i32, height: i32) -> Result<Self, String> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let child =
            Command::new(std::env::var_os("RANA_FFMPEG").unwrap_or_else(|| "ffmpeg".into()))
                .args([
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-y",
                    "-f",
                    "rawvideo",
                    "-pixel_format",
                    "rgba",
                    "-video_size",
                    &format!("{width}x{height}"),
                    "-framerate",
                    &FPS.to_string(),
                    "-i",
                    "pipe:0",
                    "-an",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "fast",
                    "-crf",
                    "18",
                    "-pix_fmt",
                    "yuv420p",
                    "-movflags",
                    "+faststart",
                ])
                .arg(path)
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|e| format!("FFmpeg: {e}"))?;
        Ok(Self { child })
    }
    pub fn frame(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.child
            .stdin
            .as_mut()
            .ok_or("Encoder cerrado")?
            .write_all(bytes)
            .map_err(|e| e.to_string())
    }
    pub fn finish(&mut self) -> Result<(), String> {
        drop(self.child.stdin.take());
        if !self.child.wait().map_err(|e| e.to_string())?.success() {
            return Err("FFmpeg no pudo finalizar el video".into());
        }
        Ok(())
    }
}
impl Drop for Recorder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tour_circles_earth_and_departs_using_normal_physics() {
        let mut w = World::default();
        setup(&mut w);
        let mut accumulated = 0.0;
        let mut previous = 0.0;
        let mut closest = f32::INFINITY;
        for _ in 0..FRAMES {
            w.update(input(&w), 1.0 / FPS as f32);
            let offset = w.position - PLANETS[2].position(w.time);
            let angle = offset.x.atan2(offset.z);
            accumulated += (angle - previous + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            previous = angle;
            closest = closest.min(length(offset));
            assert!(length(offset) > PLANETS[2].radius + 1.5);
        }
        assert!(closest < 16.0);
        assert!(accumulated > 6.1);
        assert!(length(w.position - PLANETS[2].position(w.time)) > 55.0);
    }
}
