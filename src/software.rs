//! CPU triangle pipeline. Raylib only uploads/presents the resulting RGBA buffer.
#![allow(non_snake_case, clippy::too_many_arguments)]
use crate::math::Vector3;
use crate::simulation::{cross, dot, length, v};
#[inline]
fn unit(p: Vector3) -> Vector3 {
    let n = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
    if n > 0.0001 {
        v(p.x / n, p.y / n, p.z / n)
    } else {
        v(0.0, 0.0, 1.0)
    }
}
#[inline]
fn add(a: Vector3, b: Vector3) -> Vector3 {
    v(a.x + b.x, a.y + b.y, a.z + b.z)
}
#[inline]
fn sub(a: Vector3, b: Vector3) -> Vector3 {
    v(a.x - b.x, a.y - b.y, a.z - b.z)
}
#[inline]
fn scale(a: Vector3, s: f32) -> Vector3 {
    v(a.x * s, a.y * s, a.z * s)
}
#[inline]
fn lerp(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    v(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t,
    )
}
use raylib::prelude::RaylibTexture2D;
use raylib::prelude::{
    Color, Image, RaylibDraw, RaylibHandle, RaylibThread, Rectangle, Texture2D, TextureFilter,
    Vector2,
};
use std::{cell::Cell, path::Path, rc::Rc};

#[derive(Clone, Copy)]
pub struct Camera3D {
    pub position: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub fovy: f32,
}
impl Camera3D {
    pub fn perspective(position: Vector3, target: Vector3, up: Vector3, fovy: f32) -> Self {
        Self {
            position,
            target,
            up,
            fovy,
        }
    }
}
#[derive(Clone, Copy)]
struct Vertex {
    p: Vector3,
    n: Vector3,
    local: Vector3,
    light: f32,
}
#[derive(Clone)]
pub struct Model {
    radius: f32,
    sphere: bool,
    triangles: Vec<([Vertex; 3], Color)>,
}
fn vertex(p: Vector3, n: Vector3) -> Vertex {
    Vertex {
        p,
        n,
        local: p,
        light: 1.0,
    }
}
fn rotate(p: Vector3, axis: Vector3, angle: f32) -> Vector3 {
    let a = unit(axis);
    let (s, c) = angle.sin_cos();
    let k = dot(a, p) * (1.0 - c);
    v(
        p.x * c + (a.y * p.z - a.z * p.y) * s + a.x * k,
        p.y * c + (a.z * p.x - a.x * p.z) * s + a.y * k,
        p.z * c + (a.x * p.y - a.y * p.x) * s + a.z * k,
    )
}
impl Model {
    pub fn sphere(rings: usize, slices: usize) -> Self {
        let mut triangles = Vec::new();
        let point = |i: usize, j: usize| {
            let a = std::f32::consts::PI * i as f32 / rings as f32;
            let b = std::f32::consts::TAU * j as f32 / slices as f32;
            v(a.sin() * b.cos(), a.cos(), a.sin() * b.sin())
        };
        for i in 0..rings {
            for j in 0..slices {
                let p = [
                    point(i, j),
                    point(i + 1, j),
                    point(i + 1, j + 1),
                    point(i, j + 1),
                ];
                for ids in [[0, 1, 2], [0, 2, 3]] {
                    triangles.push((ids.map(|k| vertex(p[k], p[k])), Color::WHITE));
                }
            }
        }
        Self {
            triangles,
            radius: 1.0,
            sphere: true,
        }
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut materials = std::collections::HashMap::new();
        for line in text.lines() {
            if let Some(file) = line.strip_prefix("mtllib ") {
                let mtl = std::fs::read_to_string(path.parent().unwrap().join(file))
                    .map_err(|e| e.to_string())?;
                let mut name = String::new();
                for line in mtl.lines() {
                    if let Some(n) = line.strip_prefix("newmtl ") {
                        name = n.into();
                    } else if let Some(k) = line.strip_prefix("Kd ") {
                        let a: Vec<f32> = k
                            .split_whitespace()
                            .map(|s| s.parse::<f32>().unwrap_or(1.0))
                            .collect();
                        if a.len() == 3 {
                            materials.insert(
                                name.clone(),
                                Color::new(
                                    (a[0] * 255.0) as u8,
                                    (a[1] * 255.0) as u8,
                                    (a[2] * 255.0) as u8,
                                    255,
                                ),
                            );
                        }
                    }
                }
            }
        }
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut triangles = Vec::new();
        let mut color = Color::WHITE;
        let index = |s: &str, len: usize| -> Result<usize, String> {
            let i = s.parse::<isize>().map_err(|_| "Invalid OBJ index")?;
            let k = if i < 0 { len as isize + i } else { i - 1 };
            if k < 0 || k as usize >= len {
                Err("OBJ index out of bounds".into())
            } else {
                Ok(k as usize)
            }
        };
        for line in text.lines() {
            let a: Vec<_> = line.split_whitespace().collect();
            if a.is_empty() {
                continue;
            }
            match a[0] {
                "v" | "vn" if a.len() >= 4 => {
                    let mut xyz = [0.0; 3];
                    for i in 0..3 {
                        xyz[i] = a[i + 1]
                            .parse::<f32>()
                            .map_err(|_| "Invalid OBJ coordinate")?;
                        if !xyz[i].is_finite() {
                            return Err("Nonfinite OBJ coordinate".into());
                        }
                    }
                    let p = v(xyz[0], xyz[1], xyz[2]);
                    if a[0] == "v" {
                        positions.push(p)
                    } else {
                        normals.push(p)
                    }
                }
                "usemtl" if a.len() > 1 => color = *materials.get(a[1]).unwrap_or(&Color::WHITE),
                "f" if a.len() >= 4 => {
                    let mut face = Vec::new();
                    for item in &a[1..] {
                        let ids: Vec<_> = item.split('/').collect();
                        let p = positions[index(ids[0], positions.len())?];
                        let n = if ids.len() > 2 && !ids[2].is_empty() {
                            normals[index(ids[2], normals.len())?]
                        } else {
                            Vector3::zero()
                        };
                        face.push(vertex(p, n));
                    }
                    for i in 1..face.len() - 1 {
                        let mut tri = [face[0], face[i], face[i + 1]];
                        let normal = unit(cross(sub(tri[1].p, tri[0].p), sub(tri[2].p, tri[0].p)));
                        for vert in &mut tri {
                            if length(vert.n) < 0.001 {
                                vert.n = normal;
                            }
                        }
                        triangles.push((tri, color));
                    }
                }
                _ => {}
            }
        }
        if triangles.is_empty() {
            return Err("OBJ contains no faces".into());
        }
        Ok(Self {
            triangles,
            sphere: false,
            radius: positions.iter().map(|p| length(*p)).fold(0.0, f32::max),
        })
    }
}

#[derive(Clone, Copy)]
struct Prepared {
    p: [Vector3; 3],
    b: [Vector3; 3],
    q: [Vector3; 3],
    depth: Vector3,
    light: Vector3,
}
impl Prepared {
    fn new(t: [Vertex; 3], p: [Vector3; 3]) -> Option<Self> {
        let edge = |a: Vector3, b: Vector3| v(a.y - b.y, b.x - a.x, a.x * b.y - a.y * b.x);
        let raw = [edge(p[1], p[2]), edge(p[2], p[0]), edge(p[0], p[1])];
        let area = raw[2].x * p[2].x + raw[2].y * p[2].y + raw[2].z;
        if area.abs() < 0.00001 {
            return None;
        }
        let b = raw.map(|e| e / area);
        let q = [b[0] / p[0].z, b[1] / p[1].z, b[2] / p[2].z];
        Some(Self {
            p,
            b,
            q,
            depth: q[0] + q[1] + q[2],
            light: q[0] * t[0].light + q[1] * t[1].light + q[2] * t[2].light,
        })
    }
}
#[inline]
fn plane_value(p: Vector3, x: f32, y: f32) -> f32 {
    p.x * x + p.y * y + p.z
}

#[derive(Clone, Copy)]
// Inline triangles avoid one heap allocation and pointer chase per primitive.
#[allow(clippy::large_enum_variant)]
enum Command {
    Triangle([Vertex; 3], Prepared, Color, i32),
    Line(Vector3, Vector3, Color),
    Point(Vector3, Color),
}
impl Command {
    fn color(&self) -> Color {
        match self {
            Self::Triangle(_, _, c, _) | Self::Line(_, _, c) | Self::Point(_, c) => *c,
        }
    }
    fn rows(&self, height: usize) -> (usize, usize) {
        let (lo, hi) = match self {
            Self::Triangle(_, p, _, _) => (
                p.p.iter().map(|p| p.y).fold(f32::INFINITY, f32::min),
                p.p.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max),
            ),
            Self::Line(a, b, _) => (a.y.min(b.y), a.y.max(b.y)),
            Self::Point(p, _) => (p.y, p.y),
        };
        (
            lo.floor().max(0.0).min(height as f32) as usize,
            hi.ceil().max(0.0).min(height as f32 - 1.0) as usize,
        )
    }
}

type TileOutput = (usize, Vec<u8>, Vec<f32>);
struct RenderJob {
    width: usize,
    height: usize,
    camera: Camera3D,
    time: f32,
    aspect: f32,
    commands: Vec<Command>,
    bins: Vec<Vec<usize>>,
    next: std::sync::atomic::AtomicUsize,
    done: std::sync::mpsc::Sender<Vec<TileOutput>>,
}
struct RenderPool {
    senders: Vec<std::sync::mpsc::Sender<std::sync::Arc<RenderJob>>>,
}
impl RenderPool {
    fn shared() -> &'static Self {
        static POOL: std::sync::OnceLock<RenderPool> = std::sync::OnceLock::new();
        POOL.get_or_init(|| {
            let count = std::env::var("RANA_RENDER_THREADS")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map_or(1, |n| n.get())
                        .min(4)
                })
                .clamp(1, 8);
            let mut senders = Vec::new();
            for worker in 0..count {
                let (tx, rx) = std::sync::mpsc::channel::<std::sync::Arc<RenderJob>>();
                std::thread::Builder::new()
                    .name(format!("raster-cpu-{worker}"))
                    .spawn(move || {
                        while let Ok(job) = rx.recv() {
                            let mut tiles = Vec::new();
                            loop {
                                let tile =
                                    job.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                if tile >= job.bins.len() {
                                    break;
                                }
                                let start = tile * 32;
                                let end = (start + 32).min(job.height);
                                let mut r = Renderer::new(
                                    job.width,
                                    end - start,
                                    job.camera,
                                    Rc::new(Cell::new(11)),
                                    job.time,
                                );
                                r.height = job.height;
                                r.row_start = start;
                                r.row_end = end;
                                r.aspect_correction = job.aspect;
                                r.focal = job.height as f32
                                    / (2.0 * (job.camera.fovy.to_radians() * 0.5).tan());
                                r.winners.resize(job.width * (end - start), usize::MAX);
                                r.visibility = true;
                                for &index in &job.bins[tile] {
                                    if job.commands[index].color().a == 255 {
                                        r.current_command = index;
                                        r.execute(job.commands[index]);
                                    }
                                }
                                r.visibility = false;
                                r.shade_visible(&job.commands);
                                for &index in &job.bins[tile] {
                                    if job.commands[index].color().a < 255 {
                                        r.execute(job.commands[index]);
                                    }
                                }
                                #[cfg(test)]
                                let depth = r.depth;
                                #[cfg(not(test))]
                                let depth = Vec::new();
                                tiles.push((start, r.pixels, depth));
                            }
                            let done = job.done.clone();
                            drop(job);
                            if done.send(tiles).is_err() {
                                break;
                            }
                        }
                    })
                    .expect("start raster worker");
                senders.push(tx);
            }
            RenderPool { senders }
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CullingStats {
    pub objects: usize,
    pub rejected: usize,
    pub avoided_triangles: usize,
    pub submitted_triangles: usize,
}
pub struct Renderer {
    pub object_culling: bool,
    pub stats: CullingStats,
    deferred: bool,
    visibility: bool,
    skip_clip: bool,
    current_command: usize,
    winners: Vec<usize>,
    commands: Vec<Command>,
    row_start: usize,
    row_end: usize,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    depth: Vec<f32>,
    camera: Camera3D,
    right: Vector3,
    up: Vector3,
    forward: Vector3,
    focal: f32,
    aspect_correction: f32,
    pub kind: Rc<Cell<i32>>,
    pub time: f32,
    pub triangles: usize,
    #[cfg(test)]
    reference_scan: bool,
}
impl Renderer {
    pub fn new(
        width: usize,
        height: usize,
        camera: Camera3D,
        kind: Rc<Cell<i32>>,
        time: f32,
    ) -> Self {
        let forward = unit(sub(camera.target, camera.position));
        let right = unit(cross(forward, camera.up));
        Self {
            object_culling: true,
            stats: CullingStats::default(),
            deferred: false,
            visibility: false,
            skip_clip: false,
            current_command: usize::MAX,
            winners: Vec::new(),
            commands: Vec::new(),
            row_start: 0,
            row_end: height,
            width,
            height,
            pixels: vec![0; width * height * 4],
            depth: vec![f32::INFINITY; width * height],
            camera,
            right,
            up: cross(right, forward),
            forward,
            focal: height as f32 / (2.0 * (camera.fovy.to_radians() * 0.5).tan()),
            aspect_correction: 1.0,
            kind,
            time,
            triangles: 0,
            #[cfg(test)]
            reference_scan: false,
        }
    }
    fn execute(&mut self, command: Command) {
        match command {
            Command::Triangle(t, p, c, k) => self.raster_projected(t, p, c, k),
            Command::Line(a, b, c) => self.line_projected(a, b, c),
            Command::Point(p, c) => {
                if p.x >= 0.0
                    && p.x < self.width as f32
                    && p.y >= self.row_start as f32
                    && p.y < self.row_end as f32
                {
                    self.put(p.x as usize, p.y as usize, p.z, c);
                }
            }
        }
    }
    fn shade_visible(&mut self, commands: &[Command]) {
        for i in 0..self.winners.len() {
            let index = self.winners[i];
            if index == usize::MAX {
                continue;
            }
            let Command::Triangle(t, p, color, kind) = commands[index] else {
                continue;
            };
            let x = (i % self.width) as f32 + 0.5;
            let y = (i / self.width + self.row_start) as f32 + 0.5;
            let q = p.q.map(|plane| plane_value(plane, x, y));
            let z = self.depth[i];
            let mix = |a: Vector3, b: Vector3, c: Vector3| (a * q[0] + b * q[1] + c * q[2]) * z;
            let c = if kind == 9 || kind == 11 {
                lit_color(color, plane_value(p.light, x, y) * z)
            } else {
                shade(
                    kind,
                    mix(t[0].local, t[1].local, t[2].local),
                    mix(t[0].p, t[1].p, t[2].p),
                    unit(mix(t[0].n, t[1].n, t[2].n)),
                    self.camera.position,
                    self.time,
                    color,
                )
            };
            self.pixels[i * 4..i * 4 + 4].copy_from_slice(&[c.r, c.g, c.b, c.a]);
        }
    }
    fn line_projected(&mut self, a: Vector3, b: Vector3, c: Color) {
        let steps = ((b.x - a.x).abs().max((b.y - a.y).abs()).ceil() as usize).clamp(1, 4096);
        // A long line may cross many tiles. Each worker only visits the original
        // samples in its own rectangle, rather than traversing the entire line.
        let mut lo = 0.0f32;
        let mut hi = 1.0f32;
        for (origin, delta, min, max) in [
            (a.x, b.x - a.x, 0.0, self.width as f32),
            (a.y, b.y - a.y, self.row_start as f32, self.row_end as f32),
        ] {
            if delta.abs() < 1e-8 {
                if origin < min || origin >= max {
                    return;
                }
            } else {
                let t0 = (min - origin) / delta;
                let t1 = (max - origin) / delta;
                lo = lo.max(t0.min(t1));
                hi = hi.min(t0.max(t1));
            }
        }
        if lo > hi {
            return;
        }
        let first = ((lo * steps as f32).floor().max(0.0) as usize).saturating_sub(1);
        let last = ((hi * steps as f32).ceil().max(0.0) as usize)
            .saturating_add(1)
            .min(steps);
        for i in first..=last {
            let t = i as f32 / steps as f32;
            let p = lerp(a, b, t);
            if p.x >= 0.0
                && p.x < self.width as f32
                && p.y >= self.row_start as f32
                && p.y < self.row_end as f32
            {
                self.put(
                    p.x as usize,
                    p.y as usize,
                    1.0 / ((1.0 - t) / a.z + t / b.z),
                    c,
                );
            }
        }
    }
    fn flush(&mut self) {
        if !self.deferred {
            return;
        }
        const TILE_ROWS: usize = 32;
        let tile_count = self.height.div_ceil(TILE_ROWS);
        let mut bins = vec![Vec::new(); tile_count];
        for (i, cmd) in self.commands.iter().enumerate() {
            let (lo, hi) = cmd.rows(self.height);
            if lo <= hi {
                for bin in &mut bins[lo / TILE_ROWS..=hi / TILE_ROWS] {
                    bin.push(i);
                }
            }
        }
        let pool = RenderPool::shared();
        let width = self.width;
        let (done, receive) = std::sync::mpsc::channel();
        let job = std::sync::Arc::new(RenderJob {
            width,
            height: self.height,
            camera: self.camera,
            time: self.time,
            aspect: self.aspect_correction,
            commands: std::mem::take(&mut self.commands),
            bins,
            next: std::sync::atomic::AtomicUsize::new(0),
            done,
        });
        self.triangles = job
            .commands
            .iter()
            .filter(|c| matches!(c, Command::Triangle(..)))
            .count();
        for sender in &pool.senders {
            sender.send(job.clone()).expect("raster worker available");
        }
        let mut output = Vec::new();
        for _ in &pool.senders {
            output.extend(receive.recv().expect("raster result"));
        }
        for (start, pixels, depth) in output {
            self.pixels[start * width * 4..start * width * 4 + pixels.len()]
                .copy_from_slice(&pixels);
            self.depth[start * width..start * width + depth.len()].copy_from_slice(&depth);
        }
        self.commands = std::sync::Arc::try_unwrap(job)
            .ok()
            .expect("workers released frame")
            .commands;
        self.commands.clear();
        self.deferred = false;
    }
    fn view(&self, p: Vector3) -> Vector3 {
        let d = v(
            p.x - self.camera.position.x,
            p.y - self.camera.position.y,
            p.z - self.camera.position.z,
        );
        v(dot(d, self.right), dot(d, self.up), dot(d, self.forward))
    }
    fn screen(&self, p: Vector3) -> Vector3 {
        v(
            self.width as f32 * 0.5 + p.x * self.focal * self.aspect_correction / p.z,
            self.height as f32 * 0.5 - p.y * self.focal / p.z,
            p.z,
        )
    }
    fn put(&mut self, x: usize, y: usize, z: f32, c: Color) {
        if y < self.row_start || y >= self.row_end {
            return;
        }
        let i = (y - self.row_start) * self.width + x;
        if z >= self.depth[i] || c.a == 0 {
            return;
        }
        let k = i * 4;
        if c.a == 255 {
            if self.visibility {
                self.winners[i] = usize::MAX;
            }
            self.pixels[k..k + 4].copy_from_slice(&[c.r, c.g, c.b, 255]);
            self.depth[i] = z;
            return;
        }
        let a = c.a as f32 / 255.0;
        let old_alpha = self.pixels[k + 3] as f32 / 255.0;
        let alpha = a + old_alpha * (1.0 - a);
        for (j, ch) in [c.r, c.g, c.b].iter().enumerate() {
            self.pixels[k + j] = ((*ch as f32 * a
                + self.pixels[k + j] as f32 * old_alpha * (1.0 - a))
                / alpha) as u8;
        }
        self.pixels[k + 3] = (alpha * 255.0).round() as u8;
        if c.a >= 128 {
            self.depth[i] = z;
        }
    }
    fn object_bounds(&mut self, center: Vector3, radius: f32, triangles: usize) -> Option<bool> {
        self.stats.objects += 1;
        if !self.object_culling {
            return Some(false);
        }
        let result = self.bounds(center, radius);
        if result.is_none() {
            self.stats.rejected += 1;
            self.stats.avoided_triangles += triangles;
        }
        result
    }
    fn bounds(&self, center: Vector3, radius: f32) -> Option<bool> {
        let p = self.view(center);
        let sx = self.width as f32 / (2.0 * self.focal * self.aspect_correction);
        let sy = self.height as f32 / (2.0 * self.focal);
        let dx = (1.0 + sx * sx).sqrt();
        let dy = (1.0 + sy * sy).sqrt();
        let planes = [
            p.z - 0.05,
            2000.0 - p.z,
            (p.x + p.z * sx) / dx,
            (p.z * sx - p.x) / dx,
            (p.y + p.z * sy) / dy,
            (p.z * sy - p.y) / dy,
        ];
        if planes.iter().any(|d| *d < -radius) {
            None
        } else {
            Some(planes.iter().all(|d| *d > radius))
        }
    }
    fn triangle(&mut self, tri: [Vertex; 3], color: Color, kind: i32) {
        self.stats.submitted_triangles += 1;
        if self.skip_clip {
            self.raster(tri, color, kind);
            return;
        }
        // Sutherland-Hodgman clipping in camera space against all six frustum planes.
        let sx = self.width as f32 / (2.0 * self.focal * self.aspect_correction);
        let sy = self.height as f32 / (2.0 * self.focal);
        let planes = tri.map(|a| {
            let p = self.view(a.p);
            [
                p.z - 0.05,
                2000.0 - p.z,
                p.x + p.z * sx,
                p.z * sx - p.x,
                p.y + p.z * sy,
                p.z * sy - p.y,
            ]
        });
        if (0..6).any(|i| planes.iter().all(|p| p[i] < 0.0)) {
            return;
        }
        if planes.iter().all(|p| p.iter().all(|d| *d >= 0.0)) {
            self.raster(tri, color, kind);
            return;
        }
        let mut poly = tri.to_vec();
        for plane in 0..6 {
            if poly.is_empty() {
                return;
            }
            let distance = |p: Vector3| {
                let p = self.view(p);
                match plane {
                    0 => p.z - 0.05,
                    1 => 2000.0 - p.z,
                    2 => p.x + p.z * sx,
                    3 => p.z * sx - p.x,
                    4 => p.y + p.z * sy,
                    _ => p.z * sy - p.y,
                }
            };
            let mut out = Vec::new();
            let mut prev = *poly.last().unwrap();
            let mut dp = distance(prev.p);
            for cur in poly {
                let dc = distance(cur.p);
                if (dc >= 0.0) != (dp >= 0.0) {
                    let t = dp / (dp - dc);
                    out.push(Vertex {
                        p: lerp(prev.p, cur.p, t),
                        n: lerp(prev.n, cur.n, t),
                        local: lerp(prev.local, cur.local, t),
                        light: 1.0,
                    });
                }
                if dc >= 0.0 {
                    out.push(cur);
                }
                prev = cur;
                dp = dc;
            }
            poly = out;
        }
        for i in 1..poly.len().saturating_sub(1) {
            self.raster([poly[0], poly[i], poly[i + 1]], color, kind);
        }
    }
    fn raster(&mut self, mut t: [Vertex; 3], color: Color, kind: i32) {
        if kind == 9 || kind == 11 {
            for a in &mut t {
                a.light = lighting(kind, a.p, a.n, self.camera.position);
            }
        }
        let screen = t.map(|a| self.screen(self.view(a.p)));
        let Some(p) = Prepared::new(t, screen) else {
            return;
        };
        if self.deferred {
            self.commands.push(Command::Triangle(t, p, color, kind));
            return;
        }
        self.raster_projected(t, p, color, kind);
    }
    fn raster_projected(&mut self, t: [Vertex; 3], prepared: Prepared, color: Color, kind: i32) {
        let p = prepared.p;
        self.triangles += 1;
        let xmin = p
            .iter()
            .map(|p| p.x)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let xmax = p
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.width as f32 - 1.0) as usize;
        let ymin = p
            .iter()
            .map(|p| p.y)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(self.row_start as f32) as usize;
        let ymax = p
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.row_end as f32 - 1.0) as usize;
        if ymin > ymax {
            return;
        }
        // Intersect each row with the triangle's three half-planes. Thin ground
        // strips otherwise test an almost screen-sized bounding box per triangle.
        for y in ymin..=ymax {
            let mut left = xmin as f32;
            let mut right = xmax as f32;
            for plane in prepared.b {
                let slope = plane.x;
                let intercept = plane_value(plane, 0.5, y as f32 + 0.5);
                if slope > 0.0 {
                    left = left.max((-0.00001 - intercept) / slope - 1.0);
                } else if slope < 0.0 {
                    right = right.min((-0.00001 - intercept) / slope + 1.0);
                } else if intercept < -0.00001 {
                    right = -1.0;
                }
            }
            #[cfg(test)]
            if self.reference_scan {
                left = xmin as f32;
                right = xmax as f32;
            }
            if right < left {
                continue;
            }
            let row_min = (left.floor().max(xmin as f32) as usize).min(self.width);
            let row_max = (right.ceil().min(xmax as f32) as usize).min(self.width - 1);
            // Resolve coverage at the span boundaries once. The interior of a
            // convex triangle cannot leave any of its three half-planes.
            let inside = |x: usize| {
                prepared
                    .b
                    .iter()
                    .all(|plane| plane_value(*plane, x as f32 + 0.5, y as f32 + 0.5) >= -0.00001)
            };
            let mut first = row_min;
            let mut end = row_max + 1;
            while first < end && !inside(first) {
                first += 1;
            }
            while end > first && !inside(end - 1) {
                end -= 1;
            }
            if self.visibility {
                let offset = (y - self.row_start) * self.width;
                let depths = &mut self.depth[offset + first..offset + end];
                let winners = &mut self.winners[offset + first..offset + end];
                for (j, (depth, winner)) in depths.iter_mut().zip(winners).enumerate() {
                    let z =
                        1.0 / plane_value(prepared.depth, (first + j) as f32 + 0.5, y as f32 + 0.5);
                    if z < *depth {
                        *depth = z;
                        *winner = self.current_command;
                    }
                }
                continue;
            }
            for x in first..end {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let check_edges = x <= row_min.saturating_add(2) || x.saturating_add(2) >= row_max;
                #[cfg(test)]
                let check_edges = check_edges || self.reference_scan;
                if check_edges
                    && prepared
                        .b
                        .iter()
                        .any(|plane| plane_value(*plane, px, py) < -0.00001)
                {
                    continue;
                }
                let z = 1.0 / plane_value(prepared.depth, px, py);
                if z >= self.depth[(y - self.row_start) * self.width + x] {
                    continue;
                }
                if self.visibility {
                    let i = (y - self.row_start) * self.width + x;
                    self.depth[i] = z;
                    self.winners[i] = self.current_command;
                    continue;
                }
                let q = prepared.q.map(|plane| plane_value(plane, px, py));
                let mix = |a: Vector3, b: Vector3, c: Vector3| {
                    v(
                        (a.x * q[0] + b.x * q[1] + c.x * q[2]) * z,
                        (a.y * q[0] + b.y * q[1] + c.y * q[2]) * z,
                        (a.z * q[0] + b.z * q[1] + c.z * q[2]) * z,
                    )
                };
                let world = mix(t[0].p, t[1].p, t[2].p);
                let n = unit(mix(t[0].n, t[1].n, t[2].n));
                let local = mix(t[0].local, t[1].local, t[2].local);
                let c = if kind == 9 || kind == 11 {
                    lit_color(color, plane_value(prepared.light, px, py) * z)
                } else {
                    shade(
                        kind,
                        local,
                        world,
                        n,
                        self.camera.position,
                        self.time,
                        color,
                    )
                };
                self.put(x, y, z, c);
            }
        }
    }
}
// Distinct CPU fragment programs: animated sun, rocky noise, continents/clouds,
// atmospheric clouds and gas bands. No GLSL or GPU shader is involved.
fn noise(p: Vector3) -> f32 {
    let base = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
    let smooth = |x: f32| {
        let f = x - x.floor();
        f * f * (3.0 - 2.0 * f)
    };
    let f = [smooth(p.x), smooth(p.y), smooth(p.z)];
    let mut result = 0.0;
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                let mut h = (base[0] + x) as u32;
                h = h.wrapping_mul(374761393)
                    ^ ((base[1] + y) as u32).wrapping_mul(668265263)
                    ^ ((base[2] + z) as u32).wrapping_mul(2246822519);
                h = (h ^ (h >> 13)).wrapping_mul(1274126177);
                h ^= h >> 16;
                let weight = |i: usize, k: i32| if k == 0 { 1.0 - f[i] } else { f[i] };
                result += (h as f32 / u32::MAX as f32) * weight(0, x) * weight(1, y) * weight(2, z);
            }
        }
    }
    result
}

fn lighting(kind: i32, world: Vector3, n: Vector3, eye: Vector3) -> f32 {
    let light = if kind >= 9 {
        unit(v(-0.4, 0.8, 0.6))
    } else {
        unit(-world)
    };
    (0.35 + 0.6 * dot(n, light).max(0.0) + 0.1 * dot(n, unit(eye - world)).max(0.0)).min(1.0)
}
fn lit_color(c: Color, intensity: f32) -> Color {
    Color::new(
        (c.r as f32 * intensity).clamp(0.0, 255.0) as u8,
        (c.g as f32 * intensity).clamp(0.0, 255.0) as u8,
        (c.b as f32 * intensity).clamp(0.0, 255.0) as u8,
        c.a,
    )
}
fn shade(
    kind: i32,
    p: Vector3,
    world: Vector3,
    n: Vector3,
    eye: Vector3,
    time: f32,
    c: Color,
) -> Color {
    let mut rgb = v(c.r as f32, c.g as f32, c.b as f32);
    let p = if kind < 9 || kind == 10 { unit(p) } else { p };
    let f = if kind < 9 || kind == 10 {
        noise(p * 3.0) * 0.7 + noise(p * 7.0) * 0.3
    } else {
        0.0
    };
    match kind {
        0 => {
            return Color::new(
                255,
                (100.0 + 140.0 * noise(p * 4.0 + v(time * 0.08, 0.0, 0.0))) as u8,
                30,
                c.a,
            );
        }
        1 | 10 => rgb = v(105.0, 100.0, 94.0) * (0.7 + f),
        2 => rgb = v(230.0, 174.0, 90.0) * (0.65 + noise(p + v(time * 0.01, 0.0, 0.0)) * 0.4),
        3 => {
            rgb = if f > 0.49 {
                v(48.0, 125.0, 55.0)
            } else {
                v(20.0, 74.0, 165.0)
            };
            if p.y.abs() > 0.88 || noise(p * 8.0 + v(time * 0.02, 0.0, 0.0)) > 0.72 {
                rgb = v(225.0, 238.0, 245.0);
            }
        }
        4 => rgb = v(193.0, 81.0, 43.0) * (0.65 + f * 0.5),
        5 | 6 => {
            let b = (p.y * 55.0 + f * 3.0).sin() * 0.5 + 0.5;
            rgb = if kind == 5 {
                v(164.0 + 75.0 * b, 105.0 + 100.0 * b, 70.0 + 100.0 * b)
            } else {
                v(210.0, 185.0, 125.0) * (0.7 + b * 0.3)
            };
        }
        7 => rgb = v(95.0, 195.0, 207.0) * (0.8 + f * 0.2),
        8 => rgb = v(30.0, 75.0, 205.0) * (0.7 + f * 0.4),
        _ => {}
    }
    let intensity = lighting(kind, world, n, eye);
    Color::new(
        (rgb.x * intensity).clamp(0.0, 255.0) as u8,
        (rgb.y * intensity).clamp(0.0, 255.0) as u8,
        (rgb.z * intensity).clamp(0.0, 255.0) as u8,
        c.a,
    )
}

pub trait Draw3D {
    fn renderer(&mut self) -> &mut Renderer;
    fn draw_model(&mut self, m: &Model, p: Vector3, s: f32, c: Color) {
        self.draw_model_ex(m, p, v(0.0, 1.0, 0.0), 0.0, v(s, s, s), c);
    }
    fn draw_model_ex(
        &mut self,
        m: &Model,
        pos: Vector3,
        axis: Vector3,
        angle: f32,
        scale: Vector3,
        c: Color,
    ) {
        let r = self.renderer();
        let kind = r.kind.get();
        let radius = m.radius * scale.x.abs().max(scale.y.abs()).max(scale.z.abs());
        let Some(inside) = r.object_bounds(pos, radius, m.triangles.len()) else {
            return;
        };
        let old = r.skip_clip;
        r.skip_clip = inside;
        let rx = rotate(v(1.0, 0.0, 0.0), axis, angle.to_radians());
        let ry = rotate(v(0.0, 1.0, 0.0), axis, angle.to_radians());
        let rz = rotate(v(0.0, 0.0, 1.0), axis, angle.to_radians());
        let rotation = |p: Vector3| {
            v(
                rx.x * p.x + ry.x * p.y + rz.x * p.z,
                rx.y * p.x + ry.y * p.y + rz.y * p.z,
                rx.z * p.x + ry.z * p.y + rz.z * p.z,
            )
        };
        let outside = length(sub(r.camera.position, pos)) > radius;
        for (tri, material) in &m.triangles {
            let tri = tri.map(|a| Vertex {
                p: add(
                    pos,
                    rotation(v(a.p.x * scale.x, a.p.y * scale.y, a.p.z * scale.z)),
                ),
                n: unit(rotation(v(
                    a.n.x / scale.x,
                    a.n.y / scale.y,
                    a.n.z / scale.z,
                ))),
                local: a.local,
                light: 1.0,
            });
            // Spheres have inward winding, outward normals. Keep OBJ, alpha,
            // reflected transforms and views from inside two-sided.
            if m.sphere
                && c.a == 255
                && outside
                && scale.x * scale.y * scale.z > 0.0
                && dot(
                    cross(sub(tri[1].p, tri[0].p), sub(tri[2].p, tri[0].p)),
                    sub(r.camera.position, tri[0].p),
                ) >= 0.0
            {
                continue;
            }
            let mul = |a: u8, b: u8| ((a as u16 * b as u16) / 255) as u8;
            r.triangle(
                tri,
                Color::new(
                    mul(c.r, material.r),
                    mul(c.g, material.g),
                    mul(c.b, material.b),
                    mul(c.a, material.a),
                ),
                kind,
            );
        }
        r.skip_clip = old;
    }
    fn draw_triangle3D(&mut self, a: Vector3, b: Vector3, c: Vector3, color: Color) {
        let n = unit(cross(sub(b, a), sub(c, a)));
        self.renderer()
            .triangle([vertex(a, n), vertex(b, n), vertex(c, n)], color, 11);
    }
    fn draw_cube(&mut self, p: Vector3, w: f32, h: f32, d: f32, c: Color) {
        self.draw_cube_v(p, v(w, h, d), c);
    }
    fn draw_cube_v(&mut self, p: Vector3, s: Vector3, c: Color) {
        let Some(inside) = self.renderer().object_bounds(p, length(s) * 0.5, 12) else {
            return;
        };
        let old = self.renderer().skip_clip;
        self.renderer().skip_clip = inside;
        let a = [
            v(-1.0, -1.0, -1.0),
            v(1.0, -1.0, -1.0),
            v(1.0, 1.0, -1.0),
            v(-1.0, 1.0, -1.0),
            v(-1.0, -1.0, 1.0),
            v(1.0, -1.0, 1.0),
            v(1.0, 1.0, 1.0),
            v(-1.0, 1.0, 1.0),
        ]
        .map(|q| {
            v(
                p.x + q.x * s.x * 0.5,
                p.y + q.y * s.y * 0.5,
                p.z + q.z * s.z * 0.5,
            )
        });
        for [i, j, k, l] in [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 4, 7, 3],
            [1, 2, 6, 5],
            [3, 7, 6, 2],
            [0, 1, 5, 4],
        ] {
            self.draw_triangle3D(a[i], a[j], a[k], c);
            self.draw_triangle3D(a[i], a[k], a[l], c);
        }
        self.renderer().skip_clip = old;
    }
    fn draw_sphere_ex(&mut self, p: Vector3, r: f32, rings: i32, slices: i32, c: Color) {
        thread_local! {static SPHERES: std::cell::RefCell<std::collections::HashMap<(i32,i32),Rc<Model>>> = std::cell::RefCell::new(std::collections::HashMap::new());}
        let m = SPHERES.with(|cache| {
            cache
                .borrow_mut()
                .entry((rings, slices))
                .or_insert_with(|| {
                    Rc::new(Model::sphere(rings.max(3) as usize, slices.max(3) as usize))
                })
                .clone()
        });
        let saved = self.renderer().kind.replace(11);
        self.draw_model(&m, p, r, c);
        self.renderer().kind.set(saved);
    }
    fn draw_cylinder(&mut self, p: Vector3, rt: f32, rb: f32, h: f32, s: i32, c: Color) {
        self.draw_cylinder_ex(p, p + v(0.0, h, 0.0), rb, rt, s, c);
    }
    fn draw_cylinder_ex(&mut self, a: Vector3, b: Vector3, ra: f32, rb: f32, s: i32, c: Color) {
        let radius = ((length(b - a) * 0.5).powi(2) + ra.abs().max(rb.abs()).powi(2)).sqrt();
        let Some(inside) =
            self.renderer()
                .object_bounds((a + b) * 0.5, radius, s.max(0) as usize * 4)
        else {
            return;
        };
        let old = self.renderer().skip_clip;
        self.renderer().skip_clip = inside;
        let axis = unit(sub(b, a));
        let u = unit(cross(
            axis,
            if axis.y.abs() < 0.9 {
                v(0.0, 1.0, 0.0)
            } else {
                v(1.0, 0.0, 0.0)
            },
        ));
        let w = cross(axis, u);
        for i in 0..s {
            let point = |j: i32| {
                let t = j as f32 / s as f32 * std::f32::consts::TAU;
                add(scale(u, t.cos()), scale(w, t.sin()))
            };
            let x = point(i);
            let y = point(i + 1);
            self.draw_triangle3D(
                add(a, scale(x, ra)),
                add(a, scale(y, ra)),
                add(b, scale(y, rb)),
                c,
            );
            self.draw_triangle3D(
                add(a, scale(x, ra)),
                add(b, scale(y, rb)),
                add(b, scale(x, rb)),
                c,
            );
            self.draw_triangle3D(a, add(a, scale(y, ra)), add(a, scale(x, ra)), c);
            self.draw_triangle3D(b, add(b, scale(x, rb)), add(b, scale(y, rb)), c);
        }
        self.renderer().skip_clip = old;
    }
    fn draw_point3D(&mut self, p: Vector3, c: Color) {
        let r = self.renderer();
        let q = r.view(p);
        if q.z < 0.05 {
            return;
        }
        let q = r.screen(q);
        if r.deferred {
            r.commands.push(Command::Point(q, c));
            return;
        }
        if q.x >= 0.0 && q.y >= 0.0 && q.x < r.width as f32 && q.y < r.height as f32 {
            r.put(q.x as usize, q.y as usize, q.z, c);
        }
    }
    fn draw_line3D(&mut self, a: Vector3, b: Vector3, c: Color) {
        let r = self.renderer();
        let mut a = r.view(a);
        let mut b = r.view(b);
        if a.z < 0.05 && b.z < 0.05 {
            return;
        }
        if a.z < 0.05 {
            a = lerp(a, b, (0.05 - a.z) / (b.z - a.z));
        }
        if b.z < 0.05 {
            b = lerp(b, a, (0.05 - b.z) / (a.z - b.z));
        }
        let a = r.screen(a);
        let b = r.screen(b);
        if r.deferred {
            r.commands.push(Command::Line(a, b, c));
            return;
        }
        r.line_projected(a, b, c);
    }
    fn draw_circle3D(&mut self, p: Vector3, r: f32, axis: Vector3, angle: f32, c: Color) {
        for i in 0..48 {
            let point = |j: i32| {
                let t = j as f32 / 48.0 * std::f32::consts::TAU;
                p + rotate(v(t.cos() * r, t.sin() * r, 0.0), axis, angle.to_radians())
            };
            self.draw_line3D(point(i), point(i + 1), c);
        }
    }
    fn draw_sphere_wires(&mut self, p: Vector3, r: f32, _rings: i32, _slices: i32, c: Color) {
        for axis in [v(1.0, 0.0, 0.0), v(0.0, 1.0, 0.0), v(0.0, 0.0, 1.0)] {
            self.draw_circle3D(p, r, axis, 90.0, c);
        }
    }
}
impl Draw3D for Renderer {
    fn renderer(&mut self) -> &mut Renderer {
        self
    }
}
pub struct Display {
    pub object_culling: bool,
    pub stats: CullingStats,
    cached: Option<Renderer>,
    pub frames: u64,
    pub total_ms: f64,
    pub geometry_ms: f64,
    pub pixels_ms: f64,
    pub triangles: usize,
    pub texture: Texture2D,
}
impl Display {
    pub fn new(rl: &mut RaylibHandle, t: &RaylibThread, w: i32, h: i32) -> Result<Self, String> {
        let image = Image::gen_image_color(w, h, Color::BLANK);
        let texture = rl
            .load_texture_from_image(t, &image)
            .map_err(|e| e.to_string())?;
        texture.set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
        Ok(Self {
            object_culling: true,
            stats: CullingStats::default(),
            cached: None,
            frames: 0,
            total_ms: 0.0,
            geometry_ms: 0.0,
            pixels_ms: 0.0,
            triangles: 0,
            texture,
        })
    }
}
pub struct Frame<'a, D: RaylibDraw> {
    started: std::time::Instant,
    r: Option<Renderer>,
    d: &'a mut D,
    display: Rc<std::cell::RefCell<Display>>,
    w: i32,
    h: i32,
}
impl<'a, D: RaylibDraw> Frame<'a, D> {
    pub fn new(
        d: &'a mut D,
        camera: Camera3D,
        display: Rc<std::cell::RefCell<Display>>,
        kind: Rc<Cell<i32>>,
        time: f32,
        w: i32,
        h: i32,
    ) -> Self {
        let started = std::time::Instant::now();
        let mut tex = display.borrow_mut();
        let mut r = tex.cached.take().unwrap_or_else(|| {
            Renderer::new(
                tex.texture.width as usize,
                tex.texture.height as usize,
                camera,
                kind.clone(),
                time,
            )
        });
        r.object_culling = tex.object_culling;
        r.stats = CullingStats::default();
        r.camera = camera;
        r.forward = unit(sub(camera.target, camera.position));
        r.right = unit(cross(r.forward, camera.up));
        r.up = cross(r.right, r.forward);
        r.focal = r.height as f32 / (2.0 * (camera.fovy.to_radians() * 0.5).tan());
        r.kind = kind;
        r.time = time;
        r.triangles = 0;
        r.deferred = true;
        r.aspect_correction = r.width as f32 * h as f32 / (r.height as f32 * w as f32);
        drop(tex);
        Self {
            started,
            r: Some(r),
            d,
            display,
            w,
            h,
        }
    }
}
impl<D: RaylibDraw> Draw3D for Frame<'_, D> {
    fn renderer(&mut self) -> &mut Renderer {
        self.r.as_mut().expect("active frame")
    }
}
impl<D: RaylibDraw> Drop for Frame<'_, D> {
    fn drop(&mut self) {
        let geometry_ms = self.started.elapsed().as_secs_f64() * 1000.0;
        let raster_started = std::time::Instant::now();
        let mut r = self.r.take().expect("active frame");
        r.flush();
        let pixels_ms = raster_started.elapsed().as_secs_f64() * 1000.0;
        let mut display = self.display.borrow_mut();
        display.frames += 1;
        display.geometry_ms += geometry_ms;
        display.pixels_ms += pixels_ms;
        display.total_ms += self.started.elapsed().as_secs_f64() * 1000.0;
        display.triangles = r.triangles;
        display.stats = r.stats;
        display
            .texture
            .update_texture(&r.pixels)
            .expect("RGBA framebuffer size");
        self.d.draw_texture_pro(
            &display.texture,
            Rectangle::new(0.0, 0.0, r.width as f32, r.height as f32),
            Rectangle::new(0.0, 0.0, self.w as f32, self.h as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        display.cached = Some(r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reused_deferred_frame_clears_color_depth_and_commands() {
        let mut r = renderer();
        r.deferred = true;
        r.draw_sphere_ex(v(0.0, 0.0, 3.0), 1.0, 8, 12, Color::WHITE);
        r.flush();
        assert!(r.pixels.iter().any(|&c| c != 0));
        assert!(r.commands.is_empty());
        let capacity = r.commands.capacity();
        r.deferred = true;
        r.flush();
        assert!(r.pixels.iter().all(|&c| c == 0));
        assert!(r.depth.iter().all(|&z| z == f32::INFINITY));
        assert_eq!(r.commands.capacity(), capacity);
    }
    #[test]
    fn parallel_tiles_match_serial_depth_color_and_primitive_order() {
        let mut serial = renderer();
        serial.deferred = true;
        let mut parallel = renderer();
        parallel.deferred = true;
        let mut seed = 24856u32;
        let mut sample = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        for i in 0..150 {
            let t = [0; 3].map(|_| {
                vertex(
                    v(
                        sample() * 5.0 - 2.5,
                        sample() * 5.0 - 2.5,
                        sample() * 4.0 - 0.3,
                    ),
                    v(0.0, 0.0, -1.0),
                )
            });
            let c = Color::new(150, 80, 210, if i % 2 == 0 { 255 } else { 70 });
            serial.triangle(t, c, 11);
            parallel.triangle(t, c, 11);
            serial.draw_line3D(t[0].p, t[1].p, Color::YELLOW);
            parallel.draw_line3D(t[0].p, t[1].p, Color::YELLOW);
            serial.draw_point3D(t[2].p, Color::WHITE);
            parallel.draw_point3D(t[2].p, Color::WHITE);
        }
        let commands = std::mem::take(&mut serial.commands);
        serial.deferred = false;
        for command in &commands {
            if command.color().a == 255 {
                serial.execute(*command);
            }
        }
        for command in &commands {
            if command.color().a < 255 {
                serial.execute(*command);
            }
        }
        parallel.flush();
        assert_eq!(serial.pixels, parallel.pixels);
        assert_eq!(serial.depth, parallel.depth);
    }
    #[test]
    fn sphere_face_culling_preserves_pixels_including_inside_and_alpha_views() {
        let sphere = Model::sphere(12, 20);
        let mut reference_model = sphere.clone();
        reference_model.sphere = false;
        for (z, alpha) in [(3.0, 255), (0.0, 255), (3.0, 70)] {
            let mut fast = renderer();
            let mut reference = renderer();
            let color = Color::new(100, 160, 200, alpha);
            fast.draw_model(&sphere, v(0.0, 0.0, z), 1.0, color);
            reference.draw_model(&reference_model, v(0.0, 0.0, z), 1.0, color);
            assert_eq!(fast.pixels, reference.pixels);
            assert_eq!(fast.depth, reference.depth);
            if z > 2.0 && alpha == 255 {
                assert!(fast.triangles < reference.triangles);
            }
        }
    }
    #[test]
    fn scanline_spans_match_reference_for_clipped_thin_and_transparent_triangles() {
        let mut fast = renderer();
        let mut reference = renderer();
        reference.reference_scan = true;
        let mut seed = 24856u32;
        let mut sample = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        for i in 0..250 {
            let mut t = [0; 3].map(|_| {
                vertex(
                    v(
                        sample() * 6.0 - 3.0,
                        sample() * 6.0 - 3.0,
                        sample() * 5.0 - 0.5,
                    ),
                    v(0.0, 0.0, -1.0),
                )
            });
            if i % 3 == 0 {
                t[2].p = v(t[0].p.x + 0.002, t[0].p.y + 0.002, t[0].p.z);
            }
            let c = Color::new(
                30 + (i % 200) as u8,
                150,
                210,
                if i % 2 == 0 { 255 } else { 70 },
            );
            fast.triangle(t, c, 11);
            reference.triangle(t, c, 11);
        }
        assert_eq!(fast.pixels, reference.pixels);
        assert_eq!(fast.depth, reference.depth);
    }
    fn renderer() -> Renderer {
        Renderer::new(
            64,
            64,
            Camera3D::perspective(v(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0), v(0.0, 1.0, 0.0), 60.0),
            Rc::new(Cell::new(11)),
            0.0,
        )
    }
    fn triangle(z: f32) -> [Vertex; 3] {
        [v(-0.8, -0.8, z), v(0.8, -0.8, z), v(0.0, 0.8, z)].map(|p| vertex(p, v(0.0, 0.0, -1.0)))
    }
    #[test]
    fn object_culling_preserves_image_and_counts_avoided_work() {
        let mut enabled = renderer();
        let mut disabled = renderer();
        disabled.object_culling = false;
        for r in [&mut enabled, &mut disabled] {
            r.draw_cube(v(0.0, 0.0, 3.0), 1.0, 1.0, 1.0, Color::GREEN);
            r.draw_cube(v(100.0, 0.0, 3.0), 1.0, 1.0, 1.0, Color::RED);
            r.draw_cylinder(v(0.0, 0.0, -5.0), 1.0, 1.0, 2.0, 8, Color::RED);
            r.draw_sphere_ex(v(-100.0, 0.0, 3.0), 1.0, 8, 12, Color::RED);
            // Intersecting bounds must retain geometry at viewport/near-plane edges.
            r.draw_cube(v(1.7, 0.0, 3.0), 1.0, 1.0, 1.0, Color::BLUE);
            r.draw_cube(v(0.0, 0.0, 0.06), 0.1, 0.1, 0.1, Color::YELLOW);
        }
        assert_eq!(enabled.pixels, disabled.pixels);
        assert_eq!(enabled.depth, disabled.depth);
        assert_eq!(enabled.stats.objects, 6);
        assert_eq!(enabled.stats.rejected, 3);
        assert_eq!(
            enabled.stats.avoided_triangles,
            12 + 32 + Model::sphere(8, 12).triangles.len()
        );
        assert_eq!(disabled.stats.rejected, 0);
        assert_eq!(disabled.stats.avoided_triangles, 0);
        assert!(enabled.stats.submitted_triangles < disabled.stats.submitted_triangles);
    }
    #[test]
    fn depth_is_independent_of_submission_order() {
        let mut a = renderer();
        let mut b = renderer();
        a.triangle(triangle(2.0), Color::RED, 11);
        a.triangle(triangle(1.0), Color::GREEN, 11);
        b.triangle(triangle(1.0), Color::GREEN, 11);
        b.triangle(triangle(2.0), Color::RED, 11);
        assert_eq!(a.pixels, b.pixels);
        assert!((a.depth[32 * 64 + 32] - 1.0).abs() < 0.001);
    }
    #[test]
    fn near_plane_clips_crossing_triangles_and_rejects_behind_camera() {
        let mut r = renderer();
        r.triangle(triangle(-1.0), Color::WHITE, 11);
        assert_eq!(r.triangles, 0);
        let mut t = triangle(1.0);
        t[0].p.z = -0.2;
        r.triangle(t, Color::WHITE, 11);
        assert!(r.triangles > 0);
        assert!(r.depth.iter().any(|d| d.is_finite()));
        assert!(r.depth.iter().all(|d| *d >= 0.0499));
    }
    #[test]
    fn perspective_depth_uses_reciprocal_interpolation() {
        let mut r = renderer();
        let mut t = triangle(2.0);
        t[2].p.z = 4.0;
        r.triangle(t, Color::WHITE, 11);
        let p = t.map(|a| r.screen(r.view(a.p)));
        let x = 32.5;
        let y = 32.5;
        let edge = |a: Vector3, b: Vector3, x: f32, y: f32| {
            (b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x)
        };
        let area = edge(p[0], p[1], p[2].x, p[2].y);
        let b = [
            edge(p[1], p[2], x, y) / area,
            edge(p[2], p[0], x, y) / area,
            edge(p[0], p[1], x, y) / area,
        ];
        let expected = 1.0 / (b[0] / 2.0 + b[1] / 2.0 + b[2] / 4.0);
        assert!((r.depth[32 * 64 + 32] - expected).abs() < 0.0001);
    }
    #[test]
    fn obj_materials_and_cpu_geometry_are_loaded_without_graphics_context() {
        let model = Model::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/RANA_56.obj")
                .as_path(),
        )
        .unwrap();
        assert!(model.triangles.len() > 100);
        assert!(model.triangles.iter().any(|(_, c)| c.g > c.r));
        let sphere = Model::sphere(12, 20);
        assert_eq!(sphere.triangles.len(), 480);
        assert!(
            sphere
                .triangles
                .iter()
                .all(|(t, _)| t.iter().all(|a| (length(a.p) - 1.0).abs() < 0.0001))
        );
    }
    #[test]
    fn model_translation_rotation_and_scale_reach_rasterizer() {
        let mut r = renderer();
        let model = Model::sphere(8, 12);
        r.draw_model_ex(
            &model,
            v(0.0, 0.0, 3.0),
            v(0.0, 1.0, 0.0),
            45.0,
            v(0.5, 0.8, 0.5),
            Color::WHITE,
        );
        assert!(r.depth[32 * 64 + 32] > 2.0 && r.depth[32 * 64 + 32] < 3.0);
        assert!(r.depth[0].is_infinite());
    }
}
