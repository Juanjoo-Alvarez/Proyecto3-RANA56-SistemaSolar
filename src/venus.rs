use crate::{
    astronaut::{Astronaut, Platform, SPAWN, WalkInput},
    expedition::{Expedition, Phase},
    frog::{self, Pose, Species},
    rescue::Status,
    scene::Assets,
    simulation::*,
};
use raylib::prelude::*;

const fn platform(x: f32, z: f32, w: f32, h: f32, depth: f32) -> Platform {
    Platform {
        center: Vector3 { x, y: h * 0.5, z },
        size: Vector3 {
            x: w,
            y: h,
            z: depth,
        },
    }
}
pub const PLATFORMS: [Platform; 6] = [
    platform(-23.0, 8.0, 8.0, 1.6, 8.0),
    platform(-12.0, -5.0, 6.0, 1.2, 6.0),
    platform(-16.0, -12.0, 6.0, 2.6, 6.0),
    platform(-22.0, -20.0, 10.0, 4.0, 9.0),
    platform(14.0, -12.0, 7.0, 1.8, 7.0),
    platform(23.0, -18.0, 9.0, 3.2, 9.0),
];
pub const SEEDS: [Vector3; 3] = [
    Vector3 {
        x: -23.0,
        y: 1.6,
        z: 8.0,
    },
    Vector3 {
        x: -22.0,
        y: 4.0,
        z: -20.0,
    },
    Vector3 {
        x: 23.0,
        y: 3.2,
        z: -18.0,
    },
];
pub const REFUGE: Vector3 = Vector3 {
    x: 2.0,
    y: 0.0,
    z: -35.0,
};
pub const CONTACT: f32 = 0.56;
pub const SMALL_SCALE: f32 = 0.72;
pub fn luma_mark() -> Vector3 {
    REFUGE + v(-1.2, 0.0, 3.0)
}
pub fn lirio_mark() -> Vector3 {
    REFUGE + v(1.2 * SMALL_SCALE, 0.0, 3.0 + 0.2 * (1.0 - SMALL_SCALE))
}
pub fn contact_point() -> Vector3 {
    REFUGE + v(0.0, 2.08, 3.2)
}
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub struct Mission {
    pub seeds: [bool; 3],
    pub status: Status,
    pub lirio: Astronaut,
    pub celebration: f32,
    start: Vector3,
    start_yaw: f32,
    escort_step: usize,
    pub boarding_start: Vector3,
}
impl Default for Mission {
    fn default() -> Self {
        let mut lirio = Astronaut::default();
        lirio.venus = true;
        lirio.position = REFUGE;
        lirio.yaw = 0.0;
        Self {
            seeds: [false; 3],
            status: Status::Waiting,
            lirio,
            celebration: 0.0,
            start: REFUGE,
            start_yaw: 0.0,
            escort_step: 0,
            boarding_start: SPAWN,
        }
    }
}
impl Mission {
    pub fn unlocked(&self) -> bool {
        self.seeds.iter().all(|&b| b)
    }
    pub fn complete(&self) -> bool {
        self.status == Status::Complete
    }
    pub fn near_refuge(&self, a: &Astronaut) -> bool {
        a.grounded && length(a.position - (REFUGE + v(0.0, 0.0, 3.0))) < 4.5
    }
    pub fn interact(&mut self, a: &Astronaut) -> bool {
        if !self.near_refuge(a) || self.status != Status::Waiting {
            return false;
        }
        if self.unlocked() {
            self.status = Status::Celebration;
            self.start = a.position;
            self.start_yaw = a.yaw;
            self.celebration = 0.0;
        }
        true
    }
    pub fn high_five(&self) -> f32 {
        if self.status != Status::Celebration {
            return 0.0;
        }
        let t = self.celebration;
        if t < CONTACT {
            smooth((t - 0.30) / (CONTACT - 0.30))
        } else if t < 0.585 {
            1.0
        } else {
            1.0 - smooth((t - 0.585) / 0.32)
        }
    }
    pub fn hop(&self) -> f32 {
        if self.status != Status::Celebration {
            return 0.0;
        }
        let t = ((self.celebration - 0.38) / 0.36).clamp(0.0, 1.0);
        (std::f32::consts::PI * t).sin().max(0.0) * 2.08 * (1.0 - SMALL_SCALE)
    }
    pub fn update(&mut self, a: &mut Astronaut, dt: f32) {
        if self.status == Status::Celebration {
            self.celebration = (self.celebration + dt / 5.5).min(1.0);
            let t = smooth(self.celebration / 0.28);
            a.position = self.start + (luma_mark() - self.start) * t;
            self.lirio.position = REFUGE + (lirio_mark() - REFUGE) * t + v(0.0, self.hop(), 0.0);
            a.yaw = ((self.start_yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI)
                * (1.0 - t);
            self.lirio.yaw = 0.0;
            a.velocity = Vector3::zero();
            self.lirio.velocity = Vector3::zero();
            a.charging = false;
            a.charge = 0.0;
            a.landing = 0.0;
            a.grounded = true;
            self.lirio.grounded = self.hop() < 0.01;
            a.time += dt;
            self.lirio.time += dt;
            if self.celebration >= 1.0 {
                self.status = Status::Escort;
            }
            return;
        }
        if self.status == Status::Waiting {
            for (i, p) in SEEDS.iter().enumerate() {
                let delta = a.position - *p;
                if a.grounded && length(v(delta.x, 0.0, delta.z)) < 2.5 && delta.y.abs() < 0.3 {
                    self.seeds[i] = true;
                }
            }
        }
        if self.status == Status::Escort {
            let route = [
                v(12.0, 0.0, -30.0),
                v(18.0, 0.0, -10.0),
                v(20.0, 0.0, 18.0),
                SPAWN,
            ];
            let delta = route[self.escort_step] - self.lirio.position;
            let flat = v(delta.x, 0.0, delta.z);
            if length(flat) < 1.0 && self.escort_step < 3 {
                self.escort_step += 1;
            }
            let dir = flat * (1.0 / length(flat).max(1.5));
            self.lirio.camera_yaw = 0.0;
            self.lirio.update(
                WalkInput {
                    horizontal: -dir.x,
                    forward: dir.z,
                    ..Default::default()
                },
                dt,
            );
        }
    }
}

#[derive(Default)]
pub struct Pilot {
    pub waypoint: usize,
    hold: f32,
}
impl Pilot {
    pub fn input(&mut self, a: &Astronaut, m: &Mission, dt: f32) -> WalkInput {
        let route = [
            SEEDS[0],
            v(-12.0, 1.2, -5.0),
            v(-16.0, 2.6, -12.0),
            SEEDS[1],
            v(0.0, 0.0, -30.0),
            v(14.0, 1.8, -12.0),
            SEEDS[2],
            v(20.0, 0.0, -30.0),
            REFUGE + v(0.0, 0.0, 3.0),
            v(32.0, 0.0, -30.0),
            v(32.0, 0.0, 18.0),
            SPAWN,
        ];
        if m.status == Status::Escort {
            self.waypoint = self.waypoint.max(9);
        }
        let delta = route[self.waypoint] - a.position;
        let flat = v(delta.x, 0.0, delta.z);
        let distance = length(flat);
        if distance < 1.3
            && delta.y.abs() < 0.25
            && a.grounded
            && self.waypoint < 11
            && self.waypoint != 8
        {
            self.waypoint += 1;
        }
        if self.hold <= 0.0 && a.grounded && !a.charging && delta.y > 0.3 && distance < 10.0 {
            self.hold = 0.75;
        }
        let held = self.hold > 0.0;
        let pressed = self.hold == 0.75;
        self.hold = (self.hold - dt).max(0.0);
        let direction = flat * (1.0 / distance.max(2.0));
        WalkInput {
            horizontal: dot(direction, v(-a.camera_yaw.cos(), 0.0, a.camera_yaw.sin())),
            forward: dot(direction, v(a.camera_yaw.sin(), 0.0, a.camera_yaw.cos())),
            jump_pressed: pressed,
            jump_held: held,
            run: true,
            ..Default::default()
        }
    }
    pub fn interact(&self, a: &Astronaut, m: &Mission) -> bool {
        (m.near_refuge(a) && m.unlocked() && m.status == Status::Waiting)
            || (m.status == Status::Escort && a.near_ship() && m.lirio.near_ship())
    }
}

pub fn draw(d: &mut RaylibDrawHandle, assets: &mut Assets, e: &Expedition, camera: Camera3D) {
    let (w, h) = (d.get_screen_width(), d.get_screen_height());
    d.clear_background(Color::new(222, 174, 134, 255));
    d.draw_rectangle_gradient_v(
        0,
        0,
        w,
        h,
        Color::new(93, 109, 145, 255),
        Color::new(249, 210, 164, 255),
    );
    assets.prepare(camera, e.local.time);
    assets.kind(11);
    let mut s = d.begin_mode3D(camera);
    let ivory = Color::new(218, 214, 185, 255);
    let green = Color::new(72, 132, 113, 255);
    let gold = Color::new(245, 200, 103, 255);
    let mint = Color::new(132, 241, 202, 255);
    for i in 0..36 {
        let a = i as f32 * 2.39996 + e.local.time * 0.002;
        let radius = 100.0 + (i % 5) as f32 * 17.0;
        let p = v(
            a.sin() * radius,
            -12.0 + (i % 4) as f32 * 6.0,
            a.cos() * radius,
        );
        s.draw_sphere_ex(
            p,
            15.0 + (i % 3) as f32 * 5.0,
            8,
            12,
            Color::new(235, 213, 184, 255),
        );
        s.draw_sphere_ex(
            p + v(18.0, -2.0, 0.0),
            12.0,
            8,
            12,
            Color::new(222, 198, 176, 255),
        );
    }
    s.draw_cylinder(v(0.0, -5.0, 0.0), 70.0, 62.0, 5.0, 64, ivory);
    s.draw_cylinder(
        v(0.0, -0.02, 0.0),
        68.0,
        68.0,
        0.04,
        64,
        Color::new(99, 149, 123, 255),
    );
    for i in -8..=8 {
        s.draw_cube(v(i as f32 * 6.0, 0.02, 0.0), 0.08, 0.04, 98.0, ivory);
        s.draw_cube(v(0.0, 0.02, i as f32 * 6.0), 98.0, 0.04, 0.08, ivory);
    }
    for (i, b) in PLATFORMS.iter().enumerate() {
        s.draw_cube_v(b.center, b.size, Color::new(83, 116, 120, 255));
        s.draw_cube_v(
            v(b.center.x, b.top() + 0.03, b.center.z),
            v(b.size.x + 0.1, 0.06, b.size.z + 0.1),
            ivory,
        );
        s.draw_cube(
            v(b.center.x, b.top() + 0.08, b.center.z + b.size.z * 0.4),
            b.size.x * 0.7,
            0.05,
            0.3,
            mint,
        );
        if [0, 3, 5].contains(&i) {
            for side in [-1.0, 1.0] {
                s.draw_cylinder(
                    b.center
                        + v(
                            side * (b.size.x * 0.5 - 0.4),
                            b.size.y * 0.5,
                            -b.size.z * 0.4,
                        ),
                    0.12,
                    0.12,
                    5.0,
                    8,
                    ivory,
                );
            }
            s.draw_cube(
                v(b.center.x, b.top() + 5.0, b.center.z),
                b.size.x + 0.6,
                0.25,
                b.size.z + 0.6,
                green,
            );
        }
    }
    for (i, p) in SEEDS.iter().enumerate() {
        let color = if e.venus.seeds[i] { mint } else { gold };
        s.draw_cylinder(*p, 0.6, 0.8, 0.25, 12, green);
        if !e.venus.seeds[i] {
            let center = *p + v(0.0, 1.5 + (e.local.time * 1.6 + i as f32).sin() * 0.16, 0.0);
            s.draw_sphere_ex(center, 0.42, 8, 12, color);
            s.draw_circle3D(center, 0.65, v(0.0, 1.0, 0.0), e.local.time * 35.0, color);
        }
    }
    // Invernadero central: la puerta se enciende al traer las tres semillas.
    for side in [-1.0, 1.0] {
        s.draw_cylinder(
            REFUGE + v(side * 4.5, 0.0, -2.0),
            0.35,
            0.35,
            7.0,
            12,
            ivory,
        );
    }
    s.draw_cube(REFUGE + v(0.0, 7.0, -2.0), 10.0, 0.45, 8.0, green);
    s.draw_cube(REFUGE + v(0.0, 3.4, -5.8), 10.0, 6.8, 0.4, ivory);
    s.draw_cube(REFUGE + v(-4.0, 0.6, 3.0), 2.0, 1.2, 0.8, green);
    s.draw_cube(
        REFUGE + v(-4.0, 1.25, 3.0),
        1.5,
        0.1,
        0.6,
        if e.venus.unlocked() { mint } else { gold },
    );
    for i in 0..24 {
        let a = i as f32 * std::f32::consts::TAU / 24.0;
        let p = v(a.sin() * 58.0, 0.0, a.cos() * 58.0);
        s.draw_cylinder(p, 1.4, 1.1, 1.2, 8, ivory);
        s.draw_cylinder(p + v(0.0, 1.2, 0.0), 0.18, 0.18, 4.0, 8, green);
        for side in [-1.0, 1.0] {
            s.draw_sphere_ex(p + v(side * 1.0, 3.6, 0.0), 1.1, 8, 10, green);
        }
        s.draw_sphere_ex(p + v(0.0, 5.0, 0.0), 0.5, 8, 10, gold);
    }
    s.draw_cylinder(v(0.0, 0.0, 28.0), 9.0, 9.0, 0.55, 40, ivory);
    crate::ship::draw(&mut s, &assets.ship, &e.local, crate::ship::SURFACE_SCALE);
    if e.astronaut_visible() {
        crate::ship::boarding_marker(&mut s);
        frog::draw_actor(
            &mut s,
            &assets.frog_mesh,
            &e.astronaut,
            Species::Tree,
            &Pose {
                talking: false,
                high_five: e.venus.high_five(),
            },
        );
    }
    if e.phase == Phase::Boarding || !matches!(e.venus.status, Status::Aboard | Status::Complete) {
        frog::draw_actor(
            &mut s,
            &assets.frog_mesh,
            &e.venus.lirio,
            Species::Dart,
            &Pose {
                talking: false,
                high_five: e.venus.high_five(),
            },
        );
    }
    if e.venus.high_five() > 0.98 {
        s.draw_sphere_wires(contact_point(), 0.32, 6, 8, gold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seeds_require_landing_on_the_correct_platform_and_unlock_the_refuge() {
        let mut m = Mission::default();
        let mut a = Astronaut::default();
        a.venus = true;
        a.position = REFUGE + v(0.0, 0.0, 3.0);
        assert!(m.interact(&a));
        assert_eq!(m.status, Status::Waiting);
        for (i, p) in SEEDS.iter().enumerate() {
            a.position = v(p.x, 0.0, p.z);
            m.update(&mut a, 0.0);
            assert!(!m.seeds[i]);
            a.position = *p;
            a.grounded = false;
            m.update(&mut a, 0.0);
            assert!(!m.seeds[i]);
            a.grounded = true;
            m.update(&mut a, 0.0);
            assert!(m.seeds[i]);
        }
        a.position = REFUGE + v(0.0, 0.0, 3.0);
        assert!(m.interact(&a));
        assert_eq!(m.status, Status::Celebration);
    }
    #[test]
    fn venus_rescue_completes_with_real_movement_and_keeps_brasa() {
        let mut world = World::default();
        world.travel(1);
        let mut e = Expedition::default();
        e.rescue.status = Status::Complete;
        e.cabin.recruit(0);
        let mut pilot = Pilot::default();
        let mut comms = crate::comms::Comms::default();
        let mut jumped = false;
        for frame in 0..12000 {
            let mut walk = WalkInput::default();
            if comms.blocking() {
                comms.reveal_all();
                comms.advance();
            } else {
                match e.phase {
                    Phase::Orbit if e.can_land(&world) => e.interact(&mut world),
                    Phase::OnFoot => {
                        walk = pilot.input(&e.astronaut, &e.venus, 1.0 / 60.0);
                        if pilot.interact(&e.astronaut, &e.venus) {
                            e.interact(&mut world);
                        }
                    }
                    Phase::Surface => e.interact(&mut world),
                    _ => {}
                }
                e.update(&mut world, Input::default(), walk, 1.0 / 60.0);
            }
            comms.observe(&e, e.beacons);
            comms.update(1.0 / 60.0);
            jumped |= e.venus.hop() > 0.5;
            if e.venus.complete() {
                break;
            }
            assert!(
                frame < 11999,
                "Incomplete {:?}, waypoint {}, Luma {:?}, Lirio {:?}, seeds {:?}",
                e.venus.status,
                pilot.waypoint,
                e.astronaut.position,
                e.venus.lirio.position,
                e.venus.seeds
            );
        }
        assert!(jumped);
        assert_eq!(e.phase, Phase::Orbit);
        assert_eq!(e.zone, crate::expedition::Zone::Venus);
        assert_eq!(e.cabin.count(), 2);
        assert_eq!(comms.conversations_finished, 1);
        assert!(e.rescue.thermal_shield());
        assert!(length(world.position - PLANETS[1].position(world.time)) < 18.0);
        e.toggle_cabin(&mut world);
        e.cabin.captain.position = crate::cabin::station(1) + v(2.0, 0.0, 0.0);
        assert!(e.cabin.near_lirio());
        e.cabin.talk(&mut comms);
        assert!(comms.blocking());
    }
}
