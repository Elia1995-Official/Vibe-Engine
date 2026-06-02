use crate::snake_game::SnakeDirection;
use cgmath::Vector2;

/// Every audible gameplay event.
/// Games push these into `Input::sound_events`; `main.rs` drains and plays them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundEvent {
    // Shared / menus
    MenuClick,
    GameOver,
    // Space Shooter
    SpaceShoot,
    AsteroidHit,
    PickupCollected,
    SpacePlayerHurt,
    // Snake
    FoodEaten,
    BadFoodEaten,
    // FPS Arena
    FpsShoot,
    EnemyHit,
    EnemyKill,
    FpsPlayerHurt,
    FpsLevelComplete,
    // Platformer
    Jump,
    Land,
    LevelComplete,
    PlatformerFall,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AppState {
    GameSelect,
    Settings,
    SpaceMenu,
    SpacePlaying,
    SnakeMenu,
    SnakePlaying,
    FpsMenu,
    FpsPlaying,
    PlatformerMenu,
    PlatformerPlaying,
    TetrisMenu,
    TetrisPlaying,
}

pub struct SoundSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 1.0,
        }
    }
}

pub struct Input {
    pub mouse_ndc: Vector2<f32>,
    pub firing: bool,
    pub restart: bool,
    pub held_snake_direction: Option<SnakeDirection>,
    pub move_forward: bool,
    pub move_back: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub platformer_mouse_look: bool,
    pub platformer_mouse_dx: f32,
    pub sound_events: Vec<SoundEvent>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            mouse_ndc: Vector2::new(0.0, -0.25),
            firing: false,
            restart: false,
            held_snake_direction: None,
            move_forward: false,
            move_back: false,
            move_left: false,
            move_right: false,
            platformer_mouse_look: false,
            platformer_mouse_dx: 0.0,
            sound_events: Vec::new(),
        }
    }
}
