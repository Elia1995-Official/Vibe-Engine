use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::app::{Input, SoundEvent};

pub const FPS_MAP_SIZE: usize = 17;
const ENEMY_MESH_HALF_EXTENT: f32 = 0.31;
const ENEMY_COLLISION_RADIUS: f32 = 0.24;
const FPS_PROJECTILE_RADIUS: f32 = 0.09;
const FPS_PROJECTILE_SPEED: f32 = 14.0;

#[derive(Clone, PartialEq)]
pub enum MapType {
    Indoor,
    Outdoor,
}

#[derive(Clone)]
pub struct Enemy {
    pub x: f32,
    pub z: f32,
    pub health: i32,
    pub heading: f32,
    vel_x: f32,
    vel_z: f32,
    wander_angle: f32,
    decision_timer: f32,
    strafe_sign: f32,
}

#[derive(Clone)]
pub struct Projectile {
    pub x: f32,
    pub z: f32,
    pub vx: f32,
    pub vz: f32,
    pub ttl: f32,
}

pub struct FpsGame {
    pub map: Vec<Vec<bool>>,
    pub map_type: MapType,
    pub indoor_windows: Vec<Vec<bool>>,
    pub indoor_doors: Vec<Vec<bool>>,
    pub player_x: f32,
    pub player_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub health: i32,
    pub ammo: i32,
    pub score: u32,
    pub level: u32,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub game_over: bool,
    pub shot_feedback: f32,
    shot_cooldown: f32,
    hurt_cooldown: f32,
    rng: StdRng,
}

impl FpsGame {
    pub fn new() -> Self {
        let seed = rand::random::<u64>();
        let mut game = Self {
            map: vec![vec![true; FPS_MAP_SIZE]; FPS_MAP_SIZE],
            map_type: MapType::Indoor,
            indoor_windows: vec![vec![false; FPS_MAP_SIZE]; FPS_MAP_SIZE],
            indoor_doors: vec![vec![false; FPS_MAP_SIZE]; FPS_MAP_SIZE],
            player_x: 1.5,
            player_z: 1.5,
            yaw: 0.0,
            pitch: 0.0,
            health: 100,
            ammo: 40,
            score: 0,
            level: 1,
            enemies: Vec::new(),
            projectiles: Vec::new(),
            game_over: false,
            shot_feedback: 0.0,
            shot_cooldown: 0.0,
            hurt_cooldown: 0.0,
            rng: StdRng::seed_from_u64(seed),
        };
        game.generate_level();
        game
    }

    pub fn look(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.0022;
        self.pitch = (self.pitch - dy * 0.0018).clamp(-0.8, 0.8);
    }

    pub fn update(&mut self, input: &mut Input, dt: f32) {
        if self.game_over {
            if input.restart {
                *self = Self::new();
            }
            input.restart = false;
            return;
        }

        self.shot_cooldown = (self.shot_cooldown - dt).max(0.0);
        self.hurt_cooldown = (self.hurt_cooldown - dt).max(0.0);
        self.shot_feedback = (self.shot_feedback - dt * 8.0).max(0.0);

        let forward = (self.yaw.cos(), self.yaw.sin());
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
            let speed = 4.2;
            self.try_move(mx / len * speed * dt, mz / len * speed * dt);
        }

        if input.firing && self.shot_cooldown <= 0.0 && self.ammo > 0 {
            self.shot_cooldown = 0.16;
            self.ammo -= 1;
            self.spawn_projectile();
            self.shot_feedback = 1.0;
            input.sound_events.push(SoundEvent::FpsShoot);
        }

        let proj_sounds = self.update_projectiles(dt);
        let enemy_sounds = self.update_enemies(dt);
        for e in proj_sounds.into_iter().chain(enemy_sounds) {
            input.sound_events.push(e);
        }

        if self.enemies.is_empty() {
            self.level += 1;
            self.ammo += 20;
            self.generate_level();
            input.sound_events.push(SoundEvent::FpsLevelComplete);
        }

        input.restart = false;
    }

    pub fn is_wall(&self, x: usize, z: usize) -> bool {
        self.map
            .get(z)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(true)
    }

    pub fn is_indoor_window(&self, x: usize, z: usize) -> bool {
        self.indoor_windows
            .get(z)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(false)
    }

    pub fn is_indoor_door(&self, x: usize, z: usize) -> bool {
        self.indoor_doors
            .get(z)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(false)
    }

    fn generate_level(&mut self) {
        self.map_type = if self.level % 2 == 0 {
            MapType::Outdoor
        } else {
            MapType::Indoor
        };

        match self.map_type {
            MapType::Indoor => {
                // Maze-like corridors via random walk
                self.map = vec![vec![true; FPS_MAP_SIZE]; FPS_MAP_SIZE];
                self.indoor_windows = vec![vec![false; FPS_MAP_SIZE]; FPS_MAP_SIZE];
                self.indoor_doors = vec![vec![false; FPS_MAP_SIZE]; FPS_MAP_SIZE];
                let mut x = 1usize;
                let mut z = 1usize;
                self.map[z][x] = false;
                for _ in 0..240 {
                    match self.rng.gen_range(0..4) {
                        0 if x > 1 => x -= 1,
                        1 if x < FPS_MAP_SIZE - 2 => x += 1,
                        2 if z > 1 => z -= 1,
                        3 if z < FPS_MAP_SIZE - 2 => z += 1,
                        _ => {}
                    }
                    self.map[z][x] = false;
                    if x + 1 < FPS_MAP_SIZE - 1 {
                        self.map[z][x + 1] = false;
                    }
                    if z + 1 < FPS_MAP_SIZE - 1 {
                        self.map[z + 1][x] = false;
                    }
                }

                // Promote some wall junctions to real passable doors.
                for z in 2..FPS_MAP_SIZE - 2 {
                    for x in 2..FPS_MAP_SIZE - 2 {
                        if !self.map[z][x] {
                            continue;
                        }
                        let left_open = !self.map[z][x - 1];
                        let right_open = !self.map[z][x + 1];
                        let up_open = !self.map[z - 1][x];
                        let down_open = !self.map[z + 1][x];
                        let is_corridor_break = (left_open && right_open) || (up_open && down_open);
                        if is_corridor_break && self.rng.gen_bool(0.09) && (x > 3 || z > 3) {
                            self.map[z][x] = false;
                            self.indoor_doors[z][x] = true;
                        }
                    }
                }

                // Mark some walls that border walkable corridors as windows.
                for z in 2..FPS_MAP_SIZE - 2 {
                    for x in 2..FPS_MAP_SIZE - 2 {
                        if !self.map[z][x] || self.indoor_doors[z][x] {
                            continue;
                        }
                        let adjacent_open = !self.map[z][x - 1]
                            || !self.map[z][x + 1]
                            || !self.map[z - 1][x]
                            || !self.map[z + 1][x];
                        if adjacent_open && self.rng.gen_bool(0.16) {
                            self.indoor_windows[z][x] = true;
                        }
                    }
                }
            }
            MapType::Outdoor => {
                // Open field with scattered building blocks
                self.map = vec![vec![false; FPS_MAP_SIZE]; FPS_MAP_SIZE];
                self.indoor_windows = vec![vec![false; FPS_MAP_SIZE]; FPS_MAP_SIZE];
                self.indoor_doors = vec![vec![false; FPS_MAP_SIZE]; FPS_MAP_SIZE];
                // Border walls
                for i in 0..FPS_MAP_SIZE {
                    self.map[0][i] = true;
                    self.map[FPS_MAP_SIZE - 1][i] = true;
                    self.map[i][0] = true;
                    self.map[i][FPS_MAP_SIZE - 1] = true;
                }
                // Scatter building clusters across the open map
                let clusters = 14 + self.level as usize / 2;
                for _ in 0..clusters {
                    let bx = self.rng.gen_range(2..FPS_MAP_SIZE - 3);
                    let bz = self.rng.gen_range(2..FPS_MAP_SIZE - 3);
                    let w = self.rng.gen_range(1usize..3);
                    let h = self.rng.gen_range(1usize..3);
                    for dz in 0..h {
                        for dx in 0..w {
                            if bx + dx < FPS_MAP_SIZE - 1 && bz + dz < FPS_MAP_SIZE - 1 {
                                self.map[bz + dz][bx + dx] = true;
                            }
                        }
                    }
                }
                // Ensure player start area is clear
                self.map[1][1] = false;
                self.map[1][2] = false;
                self.map[2][1] = false;
                self.map[2][2] = false;
            }
        }

        self.player_x = 1.5;
        self.player_z = 1.5;
        self.yaw = 0.0;
        self.enemies.clear();
        self.projectiles.clear();
        let enemy_count = 3 + self.level as usize;
        for _ in 0..enemy_count {
            for _ in 0..100 {
                let ex = self.rng.gen_range(3..FPS_MAP_SIZE - 2);
                let ez = self.rng.gen_range(3..FPS_MAP_SIZE - 2);
                if !self.is_wall(ex, ez) {
                    self.enemies.push(Enemy {
                        x: ex as f32 + 0.5,
                        z: ez as f32 + 0.5,
                        health: 2 + self.level as i32 / 3,
                        heading: self.rng.gen_range(0.0..std::f32::consts::TAU),
                        vel_x: 0.0,
                        vel_z: 0.0,
                        wander_angle: self.rng.gen_range(0.0..std::f32::consts::TAU),
                        decision_timer: self.rng.gen_range(0.4..1.15),
                        strafe_sign: if self.rng.gen_bool(0.5) { 1.0 } else { -1.0 },
                    });
                    break;
                }
            }
        }
    }

    fn try_move(&mut self, dx: f32, dz: f32) {
        let nx = self.player_x + dx;
        if !self.collides(nx, self.player_z) {
            self.player_x = nx;
        }
        let nz = self.player_z + dz;
        if !self.collides(self.player_x, nz) {
            self.player_z = nz;
        }
    }

    fn collides(&self, x: f32, z: f32) -> bool {
        let radius = 0.24;
        for (cx, cz) in [
            (x - radius, z - radius),
            (x + radius, z - radius),
            (x - radius, z + radius),
            (x + radius, z + radius),
        ] {
            if self.is_wall(cx.floor() as usize, cz.floor() as usize) {
                return true;
            }
        }
        false
    }

    fn spawn_projectile(&mut self) {
        let dir = (self.yaw.cos(), self.yaw.sin());
        self.projectiles.push(Projectile {
            x: self.player_x + dir.0 * 0.48,
            z: self.player_z + dir.1 * 0.48,
            vx: dir.0 * FPS_PROJECTILE_SPEED,
            vz: dir.1 * FPS_PROJECTILE_SPEED,
            ttl: 1.15,
        });
    }

    fn update_projectiles(&mut self, dt: f32) -> Vec<SoundEvent> {
        let mut sounds = Vec::new();
        let map = self.map.clone();
        let mut i = 0usize;
        while i < self.projectiles.len() {
            let mut alive = true;
            let step_x = self.projectiles[i].vx * dt;
            let step_z = self.projectiles[i].vz * dt;
            let steps = ((step_x.abs().max(step_z.abs()) / 0.08).ceil() as u32).max(1);
            let sub_x = step_x / steps as f32;
            let sub_z = step_z / steps as f32;

            for _ in 0..steps {
                let nx = self.projectiles[i].x + sub_x;
                let nz = self.projectiles[i].z + sub_z;
                if collides_in_map(&map, nx, nz, FPS_PROJECTILE_RADIUS) {
                    alive = false;
                    break;
                }
                self.projectiles[i].x = nx;
                self.projectiles[i].z = nz;
            }

            if alive {
                let mut hit_enemy: Option<usize> = None;
                for enemy_index in 0..self.enemies.len() {
                    let enemy = &self.enemies[enemy_index];
                    if circle_hits_enemy_obb(
                        self.projectiles[i].x,
                        self.projectiles[i].z,
                        FPS_PROJECTILE_RADIUS,
                        enemy.x,
                        enemy.z,
                        enemy.heading,
                        ENEMY_MESH_HALF_EXTENT,
                    ) && self.has_line_of_sight(enemy.x, enemy.z)
                    {
                        hit_enemy = Some(enemy_index);
                        break;
                    }
                }
                if let Some(enemy_index) = hit_enemy {
                    self.enemies[enemy_index].health -= 1;
                    if self.enemies[enemy_index].health <= 0 {
                        self.score += 25;
                        self.enemies.remove(enemy_index);
                        sounds.push(SoundEvent::EnemyKill);
                    } else {
                        self.score += 5;
                        sounds.push(SoundEvent::EnemyHit);
                    }
                    alive = false;
                }
            }

            self.projectiles[i].ttl -= dt;
            if self.projectiles[i].ttl <= 0.0 {
                alive = false;
            }

            if alive {
                i += 1;
            } else {
                self.projectiles.remove(i);
            }
        }
        sounds
    }

    fn update_enemies(&mut self, dt: f32) -> Vec<SoundEvent> {
        let mut sounds = Vec::new();
        let player = (self.player_x, self.player_z);
        let map = self.map.clone();
        let positions: Vec<(f32, f32)> = self.enemies.iter().map(|e| (e.x, e.z)).collect();
        let level_speed = 1.0 + self.level as f32 * 0.05;

        for index in 0..self.enemies.len() {
            let rand_wander = self.rng.gen_range(0.0..std::f32::consts::TAU);
            let rand_timer = self.rng.gen_range(0.35..1.05);
            let should_flip_strafe = self.rng.gen_bool(0.26);

            let enemy = &mut self.enemies[index];
            enemy.decision_timer -= dt;
            if enemy.decision_timer <= 0.0 {
                enemy.wander_angle = rand_wander;
                enemy.decision_timer = rand_timer;
                if should_flip_strafe {
                    enemy.strafe_sign *= -1.0;
                }
            }

            let to_player_x = player.0 - enemy.x;
            let to_player_z = player.1 - enemy.z;
            let dist = (to_player_x * to_player_x + to_player_z * to_player_z).sqrt();
            let (seek_x, seek_z) = normalize2(to_player_x, to_player_z);
            let has_los = line_of_sight_in_map(&map, enemy.x, enemy.z, player.0, player.1);

            let mut desired_x = 0.0;
            let mut desired_z = 0.0;

            // Intent blend: pursue, orbit, and wander so enemies do not look lock-on.
            if has_los {
                if dist > 4.5 {
                    desired_x += seek_x * 0.95;
                    desired_z += seek_z * 0.95;
                } else {
                    let orbit_x = -seek_z * enemy.strafe_sign;
                    let orbit_z = seek_x * enemy.strafe_sign;
                    desired_x += seek_x * 0.35 + orbit_x * 0.95;
                    desired_z += seek_z * 0.35 + orbit_z * 0.95;
                }
            } else {
                desired_x += enemy.wander_angle.cos() * 0.95;
                desired_z += enemy.wander_angle.sin() * 0.95;
                if dist < 6.2 {
                    desired_x += seek_x * 0.35;
                    desired_z += seek_z * 0.35;
                }
            }

            // Lightweight local separation avoids clumping and motion overlap.
            for (other_idx, (ox, oz)) in positions.iter().enumerate() {
                if other_idx == index {
                    continue;
                }
                let sx = enemy.x - *ox;
                let sz = enemy.z - *oz;
                let d2 = sx * sx + sz * sz;
                if d2 > 0.0001 && d2 < 1.55 {
                    let inv_d = 1.0 / d2.sqrt();
                    let weight = (1.55 - d2) * 0.9;
                    desired_x += sx * inv_d * weight;
                    desired_z += sz * inv_d * weight;
                }
            }

            let (mut desired_nx, mut desired_nz) = normalize2(desired_x, desired_z);
            if desired_nx.abs() + desired_nz.abs() < 0.0001 {
                desired_nx = enemy.wander_angle.cos();
                desired_nz = enemy.wander_angle.sin();
            }

            let probe_x = enemy.x + desired_nx * 0.42;
            let probe_z = enemy.z + desired_nz * 0.42;
            if collides_in_map(&map, probe_x, probe_z, ENEMY_MESH_HALF_EXTENT) {
                enemy.wander_angle += enemy.strafe_sign * 0.85;
                let side_x = -desired_nz * enemy.strafe_sign;
                let side_z = desired_nx * enemy.strafe_sign;
                desired_nx = side_x;
                desired_nz = side_z;
            }

            let move_speed = level_speed + if has_los { 0.28 } else { -0.06 };
            let target_vx = desired_nx * move_speed;
            let target_vz = desired_nz * move_speed;
            let accel = (dt * 5.2).clamp(0.0, 1.0);
            enemy.vel_x += (target_vx - enemy.vel_x) * accel;
            enemy.vel_z += (target_vz - enemy.vel_z) * accel;

            let step_x = enemy.vel_x * dt;
            let step_z = enemy.vel_z * dt;
            let steps = ((step_x.abs().max(step_z.abs()) / 0.07).ceil() as u32).max(1);
            let sub_x = step_x / steps as f32;
            let sub_z = step_z / steps as f32;
            for _ in 0..steps {
                let nx = enemy.x + sub_x;
                if !collides_in_map(&map, nx, enemy.z, ENEMY_MESH_HALF_EXTENT) {
                    enemy.x = nx;
                } else {
                    enemy.vel_x *= -0.18;
                }
                let nz = enemy.z + sub_z;
                if !collides_in_map(&map, enemy.x, nz, ENEMY_MESH_HALF_EXTENT) {
                    enemy.z = nz;
                } else {
                    enemy.vel_z *= -0.18;
                }
            }

            if enemy.vel_x.abs() + enemy.vel_z.abs() > 0.03 {
                enemy.heading = enemy.vel_z.atan2(enemy.vel_x);
            }

            if circle_hits_enemy_obb(
                player.0,
                player.1,
                ENEMY_COLLISION_RADIUS,
                enemy.x,
                enemy.z,
                enemy.heading,
                ENEMY_MESH_HALF_EXTENT,
            ) && self.hurt_cooldown <= 0.0
            {
                self.hurt_cooldown = 0.65;
                self.health -= 10;
                if self.health <= 0 {
                    self.game_over = true;
                    sounds.push(SoundEvent::GameOver);
                } else {
                    sounds.push(SoundEvent::FpsPlayerHurt);
                }
            }
        }
        sounds
    }

    fn has_line_of_sight(&self, tx: f32, tz: f32) -> bool {
        line_of_sight_in_map(&self.map, self.player_x, self.player_z, tx, tz)
    }
}

fn circle_hits_enemy_obb(
    cx: f32,
    cz: f32,
    radius: f32,
    ex: f32,
    ez: f32,
    heading: f32,
    half_extent: f32,
) -> bool {
    let cos_h = heading.cos();
    let sin_h = heading.sin();
    let rel_x = cx - ex;
    let rel_z = cz - ez;

    let local_x = rel_x * cos_h + rel_z * sin_h;
    let local_z = -rel_x * sin_h + rel_z * cos_h;

    let nearest_x = local_x.clamp(-half_extent, half_extent);
    let nearest_z = local_z.clamp(-half_extent, half_extent);
    let dx = local_x - nearest_x;
    let dz = local_z - nearest_z;
    dx * dx + dz * dz <= radius * radius
}

fn collides_in_map(map: &[Vec<bool>], x: f32, z: f32, radius: f32) -> bool {
    for (cx, cz) in [
        (x - radius, z - radius),
        (x + radius, z - radius),
        (x - radius, z + radius),
        (x + radius, z + radius),
    ] {
        if map
            .get(cz.floor() as usize)
            .and_then(|row| row.get(cx.floor() as usize))
            .copied()
            .unwrap_or(true)
        {
            return true;
        }
    }
    false
}

fn line_of_sight_in_map(map: &[Vec<bool>], sx: f32, sz: f32, tx: f32, tz: f32) -> bool {
    let steps = 24;
    for i in 1..steps {
        let t = i as f32 / steps as f32;
        let x = sx + (tx - sx) * t;
        let z = sz + (tz - sz) * t;
        if map
            .get(z.floor() as usize)
            .and_then(|row| row.get(x.floor() as usize))
            .copied()
            .unwrap_or(true)
        {
            return false;
        }
    }
    true
}

fn normalize2(x: f32, z: f32) -> (f32, f32) {
    let len = (x * x + z * z).sqrt();
    if len > 0.0001 {
        (x / len, z / len)
    } else {
        (0.0, 0.0)
    }
}
