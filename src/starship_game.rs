use cgmath::{vec3, InnerSpace, Vector3};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::app::{Input, SoundEvent};
use crate::config::{ASTEROID_SPEED, BULLET_SPEED, PLAYER_Z, WORLD_HALF_HEIGHT, WORLD_HALF_WIDTH};

pub struct Bullet {
    pub position: Vector3<f32>,
}

pub struct Asteroid {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub radius: f32,
    pub rotation: Vector3<f32>,
    pub spin: Vector3<f32>,
    pub mesh_id: usize,
}

#[derive(Copy, Clone)]
pub enum PickupKind {
    Repair,
    RapidFire,
    Shield,
}

pub struct Pickup {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub kind: PickupKind,
}

pub struct Player {
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub cooldown: f32,
    pub health: i32,
}

pub struct Game {
    pub player: Player,
    pub bullets: Vec<Bullet>,
    pub asteroids: Vec<Asteroid>,
    pub pickups: Vec<Pickup>,
    pub score: u32,
    pub combo: u32,
    pub best_combo: u32,
    pub kills: u32,
    pub rapid_fire_timer: f32,
    pub shield_timer: f32,
    pub game_over: bool,
    spawn_timer: f32,
    rng: StdRng,
}

impl Game {
    pub fn new() -> Self {
        Self {
            player: Player {
                position: vec3(0.0, -2.7, PLAYER_Z),
                target: vec3(0.0, -2.7, PLAYER_Z),
                cooldown: 0.0,
                health: 5,
            },
            bullets: Vec::new(),
            asteroids: Vec::new(),
            pickups: Vec::new(),
            spawn_timer: 0.3,
            score: 0,
            combo: 0,
            best_combo: 0,
            kills: 0,
            rapid_fire_timer: 0.0,
            shield_timer: 0.0,
            game_over: false,
            rng: StdRng::seed_from_u64(0x5EED_5A9E),
        }
    }

    pub fn update(&mut self, input: &mut Input, dt: f32, asteroid_mesh_count: usize) {
        if self.game_over {
            if input.restart {
                *self = Self::new();
            }
            input.restart = false;
            return;
        }

        self.player.target.x = input.mouse_ndc.x * WORLD_HALF_WIDTH;
        self.player.target.y = input.mouse_ndc.y * WORLD_HALF_HEIGHT;
        self.player.target.y = self.player.target.y.clamp(-4.4, 3.7);
        self.player.position +=
            (self.player.target - self.player.position) * (1.0 - (-14.0 * dt).exp());

        self.rapid_fire_timer = (self.rapid_fire_timer - dt).max(0.0);
        self.shield_timer = (self.shield_timer - dt).max(0.0);
        self.player.cooldown = (self.player.cooldown - dt).max(0.0);
        if input.firing && self.player.cooldown <= 0.0 {
            self.player.cooldown = if self.rapid_fire_timer > 0.0 {
                0.055
            } else {
                0.12
            };
            self.bullets.push(Bullet {
                position: self.player.position + vec3(-0.28, 0.35, -0.4),
            });
            self.bullets.push(Bullet {
                position: self.player.position + vec3(0.28, 0.35, -0.4),
            });
            input.sound_events.push(SoundEvent::SpaceShoot);
        }

        for bullet in &mut self.bullets {
            bullet.position.z -= BULLET_SPEED * dt;
        }
        self.bullets.retain(|bullet| bullet.position.z > -65.0);

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            let difficulty = 1.0 + self.score as f32 * 0.018;
            self.spawn_timer = self.rng.gen_range(0.24..0.72) / difficulty.min(3.5);
            self.asteroids.push(Asteroid {
                position: vec3(
                    self.rng.gen_range(-WORLD_HALF_WIDTH..WORLD_HALF_WIDTH),
                    self.rng.gen_range(-3.8..4.8),
                    -58.0,
                ),
                velocity: vec3(
                    self.rng.gen_range(-0.9..0.9),
                    self.rng.gen_range(-0.35..0.35),
                    ASTEROID_SPEED + self.rng.gen_range(0.0..6.0) + difficulty,
                ),
                radius: self.rng.gen_range(0.45..1.15),
                rotation: vec3(0.0, 0.0, 0.0),
                spin: vec3(
                    self.rng.gen_range(-2.2..2.2),
                    self.rng.gen_range(-2.2..2.2),
                    self.rng.gen_range(-2.2..2.2),
                ),
                mesh_id: self.rng.gen_range(0..asteroid_mesh_count),
            });
        }

        for asteroid in &mut self.asteroids {
            asteroid.position += asteroid.velocity * dt;
            asteroid.rotation += asteroid.spin * dt;
        }
        for pickup in &mut self.pickups {
            pickup.position += pickup.velocity * dt;
            pickup.rotation += vec3(1.0, 1.7, 0.4) * dt;
        }

        let mut destroyed_asteroids = vec![false; self.asteroids.len()];
        let mut destroyed_bullets = vec![false; self.bullets.len()];
        let mut spawned_pickup_positions = Vec::new();

        for (ai, asteroid) in self.asteroids.iter().enumerate() {
            for (bi, bullet) in self.bullets.iter().enumerate() {
                if destroyed_bullets[bi] {
                    continue;
                }
                if (asteroid.position - bullet.position).magnitude2()
                    < asteroid.radius * asteroid.radius
                {
                    destroyed_asteroids[ai] = true;
                    destroyed_bullets[bi] = true;
                    self.combo += 1;
                    self.best_combo = self.best_combo.max(self.combo);
                    self.kills += 1;
                    self.score += 10 + (self.combo.min(10) - 1) * 2;
                    input.sound_events.push(SoundEvent::AsteroidHit);
                    if self.rng.gen_bool(0.18) {
                        spawned_pickup_positions.push(asteroid.position);
                    }
                    break;
                }
            }
        }

        for (ai, asteroid) in self.asteroids.iter().enumerate() {
            if destroyed_asteroids[ai] {
                continue;
            }
            let player_radius = 0.85;
            let dist = asteroid.position - self.player.position;
            if dist.z > -1.3
                && dist.z < 1.2
                && (dist.x * dist.x + dist.y * dist.y).sqrt() < asteroid.radius + player_radius
            {
                destroyed_asteroids[ai] = true;
                if self.shield_timer > 0.0 {
                    self.score += 3;
                    self.shield_timer = (self.shield_timer - 1.1).max(0.0);
                } else {
                    self.combo = 0;
                    self.player.health -= 1;
                    if self.player.health <= 0 {
                        self.game_over = true;
                        input.sound_events.push(SoundEvent::GameOver);
                    } else {
                        input.sound_events.push(SoundEvent::SpacePlayerHurt);
                    }
                }
            }
        }

        let mut asteroid_index = 0;
        self.asteroids.retain(|asteroid| {
            let keep = asteroid.position.z < 7.0 && !destroyed_asteroids[asteroid_index];
            asteroid_index += 1;
            keep
        });

        let mut bullet_index = 0;
        self.bullets.retain(|_| {
            let keep = !destroyed_bullets[bullet_index];
            bullet_index += 1;
            keep
        });
        let spawned_pickups: Vec<_> = spawned_pickup_positions
            .into_iter()
            .map(|position| self.make_pickup(position))
            .collect();
        self.pickups.extend(spawned_pickups);

        let mut collected_pickup_indices = vec![false; self.pickups.len()];
        let mut collected_kinds = Vec::new();
        for (index, pickup) in self.pickups.iter().enumerate() {
            let dist = pickup.position - self.player.position;
            if dist.z > -1.1 && dist.z < 1.2 && (dist.x * dist.x + dist.y * dist.y).sqrt() < 0.95 {
                collected_pickup_indices[index] = true;
                collected_kinds.push(pickup.kind);
            }
        }
        for kind in collected_kinds {
            self.apply_pickup(kind);
            input.sound_events.push(SoundEvent::PickupCollected);
        }

        let mut pickup_index = 0;
        self.pickups.retain(|pickup| {
            let keep = pickup.position.z < 7.0 && !collected_pickup_indices[pickup_index];
            pickup_index += 1;
            keep
        });

        input.restart = false;
    }

    pub fn wave(&self) -> u32 {
        self.score / 120 + 1
    }

    fn make_pickup(&mut self, position: Vector3<f32>) -> Pickup {
        let kind = match self.rng.gen_range(0..3) {
            0 => PickupKind::Repair,
            1 => PickupKind::RapidFire,
            _ => PickupKind::Shield,
        };
        Pickup {
            position,
            velocity: vec3(0.0, 0.0, 6.2),
            rotation: vec3(0.0, 0.0, 0.0),
            kind,
        }
    }

    fn apply_pickup(&mut self, kind: PickupKind) {
        self.score += 5;
        match kind {
            PickupKind::Repair => {
                self.player.health = (self.player.health + 1).min(5);
            }
            PickupKind::RapidFire => {
                self.rapid_fire_timer = 7.0;
            }
            PickupKind::Shield => {
                self.shield_timer = 6.0;
            }
        }
    }
}
