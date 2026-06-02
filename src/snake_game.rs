use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::app::{Input, SoundEvent};

pub const SNAKE_ARENA_HALF_SIZE: f32 = 7.2;

const SEGMENT_SPACING: f32 = 0.72;
const FOOD_RADIUS: f32 = 0.72;
const SELF_COLLISION_RADIUS: f32 = 0.48;
const BASE_SPEED: f32 = 3.25;
const BOOST_MULTIPLIER: f32 = 1.9;
const TURN_SPEED: f32 = 5.4;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SnakeDirection {
    Up,
    Down,
    Left,
    Right,
}

pub struct SnakeGame {
    pub snake: Vec<(f32, f32)>,
    segment_scales: Vec<f32>,
    pub food: (f32, f32),
    pub bad_food: (f32, f32),
    pub heading: f32,
    pub target_heading: f32,
    pub requested_direction: SnakeDirection,
    pub score: u32,
    pub game_over: bool,
    rng: StdRng,
}

impl SnakeGame {
    pub fn new() -> Self {
        let mut game = Self {
            snake: vec![
                (0.0, 0.0),
                (-SEGMENT_SPACING, 0.0),
                (-SEGMENT_SPACING * 2.0, 0.0),
            ],
            segment_scales: vec![1.0, 1.0, 1.0],
            food: (2.4, 0.0),
            bad_food: (-2.4, 0.0),
            heading: 0.0,
            target_heading: 0.0,
            requested_direction: SnakeDirection::Right,
            score: 0,
            game_over: false,
            rng: StdRng::seed_from_u64(0x51A4E),
        };
        game.food = game.random_food();
        game.bad_food = game.random_food();
        game
    }

    pub fn set_direction(&mut self, direction: SnakeDirection) {
        self.requested_direction = direction;
        self.target_heading = direction_yaw(direction);
    }

    pub fn update(&mut self, input: &mut Input, dt: f32) {
        if self.game_over {
            if input.restart {
                *self = Self::new();
            }
            input.restart = false;
            return;
        }

        let boost = if input.held_snake_direction == Some(self.requested_direction) {
            BOOST_MULTIPLIER
        } else {
            1.0
        };
        let speed = (BASE_SPEED + self.score as f32 * 0.08) * boost;

        self.heading = approach_angle(self.heading, self.target_heading, TURN_SPEED * dt);
        self.snake[0].0 += self.heading.cos() * speed * dt;
        self.snake[0].1 += self.heading.sin() * speed * dt;

        for i in 1..self.snake.len() {
            let previous = self.snake[i - 1];
            let current = self.snake[i];
            let dx = current.0 - previous.0;
            let dy = current.1 - previous.1;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 0.001 {
                let desired_x = previous.0 + dx / distance * SEGMENT_SPACING;
                let desired_y = previous.1 + dy / distance * SEGMENT_SPACING;
                self.snake[i].0 += (desired_x - current.0) * 0.92;
                self.snake[i].1 += (desired_y - current.1) * 0.92;
            }
        }

        for scale in &mut self.segment_scales {
            *scale = (*scale + dt * 3.8).min(1.0);
        }

        let head = self.snake[0];
        if head.0.abs() > SNAKE_ARENA_HALF_SIZE || head.1.abs() > SNAKE_ARENA_HALF_SIZE {
            self.game_over = true;
            input.sound_events.push(SoundEvent::GameOver);
            return;
        }

        if distance(head, self.food) < FOOD_RADIUS {
            self.score += 1;
            let tail = *self.snake.last().expect("snake has a tail");
            self.snake.push(tail);
            self.segment_scales.push(0.05);
            self.food = self.random_food();
            self.bad_food = self.random_food();
            input.sound_events.push(SoundEvent::FoodEaten);
        } else if distance(head, self.bad_food) < FOOD_RADIUS {
            self.score = self.score.saturating_sub(1);
            if self.snake.len() > 3 {
                self.snake.pop();
                self.segment_scales.pop();
            }
            self.bad_food = self.random_food();
            input.sound_events.push(SoundEvent::BadFoodEaten);
        }

        for segment in self.snake.iter().skip(6) {
            if distance(head, *segment) < SELF_COLLISION_RADIUS {
                self.game_over = true;
                input.sound_events.push(SoundEvent::GameOver);
                return;
            }
        }

        input.restart = false;
    }

    pub fn visual_segment_position(&self, index: usize) -> (f32, f32) {
        self.snake[index]
    }

    pub fn visual_segment_yaw(&self, index: usize) -> f32 {
        if index == 0 {
            return self.heading;
        }

        let previous = self.snake[index - 1];
        let current = self.snake[index];
        (previous.1 - current.1).atan2(previous.0 - current.0)
    }

    pub fn visual_segment_scale(&self, index: usize) -> f32 {
        self.segment_scales.get(index).copied().unwrap_or(1.0)
    }

    fn random_food(&mut self) -> (f32, f32) {
        loop {
            let food = (
                self.rng
                    .gen_range(-SNAKE_ARENA_HALF_SIZE + 0.8..SNAKE_ARENA_HALF_SIZE - 0.8),
                self.rng
                    .gen_range(-SNAKE_ARENA_HALF_SIZE + 0.8..SNAKE_ARENA_HALF_SIZE - 0.8),
            );
            if self
                .snake
                .iter()
                .all(|&segment| distance(segment, food) > SEGMENT_SPACING * 1.8)
                && distance(food, self.food) > SEGMENT_SPACING * 2.2
                && distance(food, self.bad_food) > SEGMENT_SPACING * 2.2
            {
                return food;
            }
        }
    }
}

fn direction_yaw(direction: SnakeDirection) -> f32 {
    match direction {
        SnakeDirection::Up => -std::f32::consts::FRAC_PI_2,
        SnakeDirection::Down => std::f32::consts::FRAC_PI_2,
        SnakeDirection::Left => std::f32::consts::PI,
        SnakeDirection::Right => 0.0,
    }
}

fn approach_angle(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta = shortest_angle_delta(current, target);
    if delta.abs() <= max_delta {
        target
    } else {
        current + delta.signum() * max_delta
    }
}

fn shortest_angle_delta(current: f32, target: f32) -> f32 {
    let mut delta = target - current;
    while delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    while delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}
