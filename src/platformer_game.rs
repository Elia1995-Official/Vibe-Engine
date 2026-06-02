use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::app::{Input, SoundEvent};

#[allow(dead_code)]
pub const PLATFORMER_ARENA_SIZE: f32 = 42.0;

#[derive(Clone)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

pub struct PlatformerGame {
    pub player_x: f32,
    pub player_y: f32,
    pub player_z: f32,
    pub vel_y: f32,
    pub camera_yaw: f32,
    pub on_ground: bool,
    pub platforms: Vec<Platform>,
    pub score: u32,
    pub level: u32,
    pub game_over: bool,
    rng: StdRng,
}

const PLAYER_RADIUS: f32 = 0.28;
const GRAVITY: f32 = 18.5;
const JUMP_POWER: f32 = 8.2;
const MOVE_SPEED: f32 = 4.8;
const LOOK_SPEED: f32 = 0.0022;

impl PlatformerGame {
    pub fn new() -> Self {
        let mut game = Self {
            player_x: 0.0,
            player_y: 2.0,
            player_z: 0.0,
            vel_y: 0.0,
            camera_yaw: 0.0,
            on_ground: false,
            platforms: Vec::new(),
            score: 0,
            level: 1,
            game_over: false,
            rng: StdRng::seed_from_u64(rand::random::<u64>()),
        };
        game.generate_level();
        game
    }

    pub fn update(&mut self, input: &mut Input, dt: f32) {
        if self.game_over {
            if input.restart {
                *self = Self::new();
            }
            input.restart = false;
            return;
        }

        self.camera_yaw += input.platformer_mouse_dx * LOOK_SPEED;
        input.platformer_mouse_dx = 0.0;

        self.on_ground = self.is_grounded();

        let forward = (self.camera_yaw.cos(), self.camera_yaw.sin());
        let right = (-forward.1, forward.0);
        let mut mx = 0.0;
        let mut mz = 0.0;

        if input.move_forward {
            mx += forward.0;
            mz += forward.1;
        }
        if input.move_back {
            mx -= forward.0;
            mz -= forward.1;
        }
        if input.move_right {
            mx += right.0;
            mz += right.1;
        }
        if input.move_left {
            mx -= right.0;
            mz -= right.1;
        }

        let len = (mx * mx + mz * mz).sqrt();
        if len > 0.001 {
            self.try_move_horizontal(mx / len * MOVE_SPEED * dt, mz / len * MOVE_SPEED * dt);
        }

        if input.firing && self.on_ground {
            self.vel_y = JUMP_POWER;
            self.on_ground = false;
            input.firing = false;
            input.sound_events.push(SoundEvent::Jump);
        }

        self.vel_y -= GRAVITY * dt;
        let was_on_ground = self.on_ground;
        self.try_move_vertical(self.vel_y * dt);
        if !was_on_ground && self.on_ground {
            input.sound_events.push(SoundEvent::Land);
        }

        if self.is_on_goal_platform() {
            self.complete_level();
            input.sound_events.push(SoundEvent::LevelComplete);
        }

        if self.player_y < -8.0 {
            self.game_over = true;
            input.sound_events.push(SoundEvent::PlatformerFall);
            input.sound_events.push(SoundEvent::GameOver);
        }

        input.restart = false;
    }

    fn generate_level(&mut self) {
        self.platforms.clear();

        self.platforms.push(Platform {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 4.0,
            height: 0.5,
            depth: 4.0,
        });

        let mut px = 0.0f32;
        let mut pz = 0.0f32;
        let mut py = 0.0f32;
        for _ in 0..8 + self.level as usize {
            px += self.rng.gen_range(2.5..4.2);
            pz += self.rng.gen_range(-1.8..1.8);
            py += self.rng.gen_range(-0.3..0.8);
            py = py.max(0.2);

            let width = self.rng.gen_range(1.2..2.8);
            let depth = self.rng.gen_range(1.2..2.8);

            self.platforms.push(Platform {
                x: px,
                y: py,
                z: pz,
                width,
                height: 0.4,
                depth,
            });
        }

        self.platforms.push(Platform {
            x: px + 4.0,
            y: py + 1.5,
            z: pz,
            width: 3.0,
            height: 0.5,
            depth: 3.0,
        });

        self.player_x = 0.0;
        self.player_y = 2.0;
        self.player_z = 0.0;
        self.vel_y = 0.0;
        self.camera_yaw = 0.0;
        self.on_ground = false;
    }

    fn try_move_horizontal(&mut self, dx: f32, dz: f32) {
        let support_platform = self.support_platform_index();
        let nx = self.player_x + dx;
        if !self.player_collides_at(nx, self.player_y, self.player_z, support_platform) {
            self.player_x = nx;
        }

        let nz = self.player_z + dz;
        if !self.player_collides_at(self.player_x, self.player_y, nz, support_platform) {
            self.player_z = nz;
        }
    }

    fn try_move_vertical(&mut self, dy: f32) {
        if dy.abs() <= f32::EPSILON {
            return;
        }

        let ny = self.player_y + dy;
        let mut collided_ground = false;
        let mut new_y = ny;

        for platform in &self.platforms {
            if self.player_collides_platform(self.player_x, ny, self.player_z, platform) {
                let px_min = platform.x - platform.width * 0.5;
                let px_max = platform.x + platform.width * 0.5;
                let py_min = platform.y;
                let py_max = platform.y + platform.height;
                let pz_min = platform.z - platform.depth * 0.5;
                let pz_max = platform.z + platform.depth * 0.5;

                if dy < 0.0
                    && self.player_y - PLAYER_RADIUS < py_max
                    && self.player_y - PLAYER_RADIUS >= py_max - 0.25
                    && self.player_x > px_min
                    && self.player_x < px_max
                    && self.player_z > pz_min
                    && self.player_z < pz_max
                {
                    new_y = py_max + PLAYER_RADIUS;
                    self.vel_y = 0.0;
                    collided_ground = true;
                } else if dy > 0.0
                    && self.player_y + PLAYER_RADIUS <= py_min + 0.25
                    && self.player_x > px_min
                    && self.player_x < px_max
                    && self.player_z > pz_min
                    && self.player_z < pz_max
                {
                    new_y = py_min - PLAYER_RADIUS;
                    self.vel_y = 0.0;
                }
            }
        }

        self.player_y = new_y;
        self.on_ground = collided_ground || self.support_platform_index().is_some();
    }

    fn is_grounded(&self) -> bool {
        self.support_platform_index().is_some()
    }

    fn support_platform_index(&self) -> Option<usize> {
        self.platforms.iter().enumerate().find_map(|(index, platform)| {
            let px_min = platform.x - platform.width * 0.5;
            let px_max = platform.x + platform.width * 0.5;
            let py_max = platform.y + platform.height;
            let pz_min = platform.z - platform.depth * 0.5;
            let pz_max = platform.z + platform.depth * 0.5;

            if self.player_x > px_min
                && self.player_x < px_max
                && self.player_z > pz_min
                && self.player_z < pz_max
                && (self.player_y - PLAYER_RADIUS - py_max).abs() <= 0.06
            {
                Some(index)
            } else {
                None
            }
        })
    }

    fn is_on_goal_platform(&self) -> bool {
        self.support_platform_index() == self.platforms.len().checked_sub(1)
    }

    fn complete_level(&mut self) {
        self.score += 100;
        self.level += 1;
        self.generate_level();
    }

    fn player_collides_at(&self, x: f32, y: f32, z: f32, ignore: Option<usize>) -> bool {
        self.platforms.iter().enumerate().any(|(index, platform)| {
            if ignore == Some(index) {
                false
            } else {
                self.player_collides_platform(x, y, z, platform)
            }
        })
    }

    fn player_collides_platform(&self, x: f32, y: f32, z: f32, platform: &Platform) -> bool {
        let px_min = platform.x - platform.width * 0.5;
        let px_max = platform.x + platform.width * 0.5;
        let py_min = platform.y;
        let py_max = platform.y + platform.height;
        let pz_min = platform.z - platform.depth * 0.5;
        let pz_max = platform.z + platform.depth * 0.5;

        let cx = x.clamp(px_min, px_max);
        let cy = y.clamp(py_min, py_max);
        let cz = z.clamp(pz_min, pz_max);

        let dx = x - cx;
        let dy = y - cy;
        let dz = z - cz;

        dx * dx + dy * dy + dz * dz <= PLAYER_RADIUS * PLAYER_RADIUS
    }
}
