use glium::winit::event::{ElementState, KeyEvent};
use glium::winit::event_loop::ActiveEventLoop;
use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::winit::window::Window;

use crate::app::{AppState, Input};
use crate::fps_game::FpsGame;
use crate::starship_game::Game;
use crate::platformer_game::PlatformerGame;
use crate::snake_game::{SnakeDirection, SnakeGame};
use crate::tetris_game::TetrisGame;

pub fn handle_keyboard_input(
    event: &KeyEvent,
    app_state: &mut AppState,
    show_help: &mut bool,
    input: &mut Input,
    game: &mut Game,
    snake: &mut SnakeGame,
    fps: &mut FpsGame,
    platformer: &mut PlatformerGame,
    tetris: &mut TetrisGame,
    window: &Window,
    window_target: &ActiveEventLoop,
) {
    let pressed = event.state == ElementState::Pressed;
    let snake_direction_key = match event.physical_key {
        PhysicalKey::Code(KeyCode::ArrowUp) | PhysicalKey::Code(KeyCode::KeyW) => {
            Some(SnakeDirection::Up)
        }
        PhysicalKey::Code(KeyCode::ArrowDown) | PhysicalKey::Code(KeyCode::KeyS) => {
            Some(SnakeDirection::Down)
        }
        PhysicalKey::Code(KeyCode::ArrowLeft) | PhysicalKey::Code(KeyCode::KeyA) => {
            Some(SnakeDirection::Left)
        }
        PhysicalKey::Code(KeyCode::ArrowRight) | PhysicalKey::Code(KeyCode::KeyD) => {
            Some(SnakeDirection::Right)
        }
        _ => None,
    };

    if *app_state == AppState::SnakePlaying {
        if let Some(direction) = snake_direction_key {
            if pressed {
                input.held_snake_direction = Some(direction);
                snake.set_direction(direction);
            } else if input.held_snake_direction == Some(direction) {
                input.held_snake_direction = None;
            }
        }
    }

    if *app_state == AppState::FpsPlaying {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::KeyW) => input.move_forward = pressed,
            PhysicalKey::Code(KeyCode::KeyS) => input.move_back = pressed,
            PhysicalKey::Code(KeyCode::KeyA) => input.move_left = pressed,
            PhysicalKey::Code(KeyCode::KeyD) => input.move_right = pressed,
            _ => {}
        }
    }

    if *app_state == AppState::PlatformerPlaying {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::KeyW) => input.move_forward = pressed,
            PhysicalKey::Code(KeyCode::KeyS) => input.move_back = pressed,
            PhysicalKey::Code(KeyCode::KeyA) => input.move_left = pressed,
            PhysicalKey::Code(KeyCode::KeyD) => input.move_right = pressed,
            _ => {}
        }
    }

    if *app_state == AppState::TetrisPlaying {
        if let PhysicalKey::Code(key) = event.physical_key {
            tetris.handle_key(key, pressed);
        }
    }

    match event.physical_key {
        PhysicalKey::Code(KeyCode::F1) if pressed => {
            *show_help = !*show_help;
        }
        PhysicalKey::Code(KeyCode::Escape) if pressed => {
            if *show_help {
                *show_help = false;
            } else if *app_state == AppState::SpacePlaying {
                enter_game_menu(app_state, AppState::SpaceMenu, input, window);
            } else if *app_state == AppState::SnakePlaying {
                enter_game_menu(app_state, AppState::SnakeMenu, input, window);
            } else if *app_state == AppState::FpsPlaying {
                enter_game_menu(app_state, AppState::FpsMenu, input, window);
            } else if *app_state == AppState::PlatformerPlaying {
                enter_game_menu(app_state, AppState::PlatformerMenu, input, window);
            } else if *app_state == AppState::TetrisPlaying {
                enter_game_menu(app_state, AppState::TetrisMenu, input, window);
            } else if matches!(
                *app_state,
                AppState::SpaceMenu
                    | AppState::SnakeMenu
                    | AppState::FpsMenu
                    | AppState::PlatformerMenu
                    | AppState::TetrisMenu
                    | AppState::Settings
            ) {
                enter_select(app_state, input, window);
            } else {
                window_target.exit();
            }
        }
        PhysicalKey::Code(KeyCode::Enter) if pressed && *app_state == AppState::GameSelect => {
            enter_game_menu(app_state, AppState::SpaceMenu, input, window);
        }
        PhysicalKey::Code(KeyCode::Enter) if pressed && *app_state == AppState::SpaceMenu => {
            enter_space_game(app_state, game, input, window);
        }
        PhysicalKey::Code(KeyCode::Enter) if pressed && *app_state == AppState::SnakeMenu => {
            enter_snake_game(app_state, snake, input, window);
        }
        PhysicalKey::Code(KeyCode::Enter) if pressed && *app_state == AppState::FpsMenu => {
            enter_fps_game(app_state, fps, input, window);
        }
        PhysicalKey::Code(KeyCode::Enter) if pressed && *app_state == AppState::PlatformerMenu => {
            enter_platformer_game(app_state, platformer, input, window);
        }
        PhysicalKey::Code(KeyCode::Enter) if pressed && *app_state == AppState::TetrisMenu => {
            enter_tetris_game(app_state, tetris, input, window);
        }
        PhysicalKey::Code(KeyCode::Space) => {
            if *app_state == AppState::SpaceMenu && pressed {
                enter_space_game(app_state, game, input, window);
            } else if *app_state == AppState::SnakeMenu && pressed {
                enter_snake_game(app_state, snake, input, window);
            } else if *app_state == AppState::FpsMenu && pressed {
                enter_fps_game(app_state, fps, input, window);
            } else if *app_state == AppState::PlatformerMenu && pressed {
                enter_platformer_game(app_state, platformer, input, window);
            } else if *app_state == AppState::TetrisMenu && pressed {
                enter_tetris_game(app_state, tetris, input, window);
            } else if *app_state == AppState::TetrisPlaying {
                // Space hard drop is consumed by TetrisGame::handle_key.
            } else if *app_state == AppState::PlatformerPlaying {
                input.firing = pressed;
            } else {
                input.firing = pressed;
            }
        }
        PhysicalKey::Code(KeyCode::KeyS) if pressed && *app_state == AppState::GameSelect => {
            enter_game_menu(app_state, AppState::SnakeMenu, input, window);
        }
        PhysicalKey::Code(KeyCode::KeyF) if pressed && *app_state == AppState::GameSelect => {
            enter_game_menu(app_state, AppState::FpsMenu, input, window);
        }
        PhysicalKey::Code(KeyCode::KeyT) if pressed && *app_state == AppState::GameSelect => {
            enter_game_menu(app_state, AppState::TetrisMenu, input, window);
        }
        PhysicalKey::Code(KeyCode::KeyR) if pressed => input.restart = true,
        _ => {}
    }
}

pub fn enter_space_game(
    app_state: &mut AppState,
    game: &mut Game,
    input: &mut Input,
    window: &Window,
) {
    *app_state = AppState::SpacePlaying;
    *game = Game::new();
    input.firing = false;
    input.held_snake_direction = None;
    input.platformer_mouse_look = false;
    input.platformer_mouse_dx = 0.0;
    let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::Confined);
    window.set_cursor_visible(false);
}

pub fn enter_snake_game(
    app_state: &mut AppState,
    snake: &mut SnakeGame,
    input: &mut Input,
    window: &Window,
) {
    *app_state = AppState::SnakePlaying;
    *snake = SnakeGame::new();
    input.firing = false;
    input.held_snake_direction = None;
    input.platformer_mouse_look = false;
    input.platformer_mouse_dx = 0.0;
    let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::None);
    window.set_cursor_visible(true);
}

pub fn enter_fps_game(
    app_state: &mut AppState,
    fps: &mut FpsGame,
    input: &mut Input,
    window: &Window,
) {
    *app_state = AppState::FpsPlaying;
    *fps = FpsGame::new();
    input.firing = false;
    input.held_snake_direction = None;
    input.platformer_mouse_look = false;
    input.platformer_mouse_dx = 0.0;
    let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::Confined);
    window.set_cursor_visible(false);
}

pub fn enter_platformer_game(
    app_state: &mut AppState,
    platformer: &mut PlatformerGame,
    input: &mut Input,
    window: &Window,
) {
    *app_state = AppState::PlatformerPlaying;
    *platformer = PlatformerGame::new();
    input.firing = false;
    input.held_snake_direction = None;
    input.platformer_mouse_look = false;
    input.platformer_mouse_dx = 0.0;
    let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::Confined);
    window.set_cursor_visible(false);
}

pub fn enter_tetris_game(
    app_state: &mut AppState,
    tetris: &mut TetrisGame,
    input: &mut Input,
    window: &Window,
) {
    *app_state = AppState::TetrisPlaying;
    *tetris = TetrisGame::new();
    input.firing = false;
    input.held_snake_direction = None;
    input.move_forward = false;
    input.move_back = false;
    input.move_left = false;
    input.move_right = false;
    input.platformer_mouse_look = false;
    input.platformer_mouse_dx = 0.0;
    let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::None);
    window.set_cursor_visible(true);
}

pub fn enter_select(app_state: &mut AppState, input: &mut Input, window: &Window) {
    *app_state = AppState::GameSelect;
    input.firing = false;
    input.held_snake_direction = None;
    input.move_forward = false;
    input.move_back = false;
    input.move_left = false;
    input.move_right = false;
    input.platformer_mouse_look = false;
    input.platformer_mouse_dx = 0.0;
    let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::None);
    window.set_cursor_visible(true);
}

pub fn enter_game_menu(
    app_state: &mut AppState,
    menu_state: AppState,
    input: &mut Input,
    window: &Window,
) {
    *app_state = menu_state;
    input.firing = false;
    input.held_snake_direction = None;
    input.move_forward = false;
    input.move_back = false;
    input.move_left = false;
    input.move_right = false;
    input.platformer_mouse_look = false;
    input.platformer_mouse_dx = 0.0;
    let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::None);
    window.set_cursor_visible(true);
}
