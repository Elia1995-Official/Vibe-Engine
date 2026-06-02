mod app;
mod config;
mod fps_game;
mod starship_game;
mod graphics;
mod keybindings;
mod platformer_game;
mod procedural;
mod renderer;
mod shaders;
mod snake_game;
mod sound;
mod tetris_game;
mod ui;

use std::time::Instant;

use glium::winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent};

use app::{AppState, Input, SoundEvent, SoundSettings};
use sound::SoundEngine;
use fps_game::FpsGame;
use starship_game::Game;
use platformer_game::PlatformerGame;
use keybindings::{
    enter_fps_game, enter_game_menu, enter_platformer_game, enter_select, enter_snake_game,
    enter_space_game, enter_tetris_game, handle_keyboard_input,
};
use renderer::Renderer;
use snake_game::SnakeGame;
use tetris_game::TetrisGame;
use ui::{menu_button, mouse_to_ui, slider_track_rect, MenuButton, Rect};

fn main() {
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .expect("event loop");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Vibe Engine - Game Collection")
        .with_inner_size(1280, 720)
        .build(&event_loop);

    window.set_cursor_visible(true);

    let mut renderer = Renderer::new(&display);
    let mut game = Game::new();
    let mut snake = SnakeGame::new();
    let mut fps = FpsGame::new();
    let mut platformer = PlatformerGame::new();
    let mut tetris = TetrisGame::new();
    let mut input = Input::default();
    let mut app_state = AppState::GameSelect;
    let mut show_help = false;
    let mut sound_settings = SoundSettings::default();
    let mut dragging_slider: Option<usize> = None;
    let sound_engine = SoundEngine::new();
    let start = Instant::now();
    let mut last_frame = start;

    #[allow(deprecated)]
    let _ = event_loop.run(move |event, window_target| match event {
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            if app_state == AppState::SpacePlaying {
                input.mouse_ndc.x = (input.mouse_ndc.x + delta.0 as f32 * 0.0016).clamp(-1.0, 1.0);
                input.mouse_ndc.y = (input.mouse_ndc.y - delta.1 as f32 * 0.0021).clamp(-1.0, 1.0);
            } else if app_state == AppState::FpsPlaying {
                fps.look(delta.0 as f32, delta.1 as f32);
            } else if app_state == AppState::PlatformerPlaying {
                input.platformer_mouse_dx += delta.0 as f32;
            }
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => window_target.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                let size = window.inner_size();
                input.mouse_ndc.x = (position.x as f32 / size.width.max(1) as f32) * 2.0 - 1.0;
                input.mouse_ndc.y = 1.0 - (position.y as f32 / size.height.max(1) as f32) * 2.0;
                if app_state == AppState::Settings {
                    if let Some(idx) = dragging_slider {
                        let aspect = size.width as f32 / size.height.max(1) as f32;
                        let mouse = mouse_to_ui(input.mouse_ndc, aspect);
                        let track = slider_track_rect(idx);
                        let value = ((mouse.x - track.x) / track.w).clamp(0.0, 1.0);
                        match idx {
                            0 => sound_settings.master_volume = value,
                            1 => sound_settings.music_volume = value,
                            2 => sound_settings.sfx_volume = value,
                            _ => {}
                        }
                    }
                }
                if app_state == AppState::TetrisPlaying {
                    let aspect = size.width as f32 / size.height.max(1) as f32;
                    let mouse = mouse_to_ui(input.mouse_ndc, aspect);
                    tetris.handle_mouse_move(mouse.x, mouse.y);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                if app_state == AppState::TetrisPlaying {
                    if pressed {
                        match button {
                            MouseButton::Left => tetris.mouse_rotate(),
                            MouseButton::Right => tetris.mouse_hard_drop(),
                            MouseButton::Middle => tetris.mouse_hold(),
                            _ => {}
                        }
                    }
                } else if button == MouseButton::Left {
                    if !pressed {
                        dragging_slider = None;
                        input.firing = false;
                    } else if app_state == AppState::Settings {
                        let size = window.inner_size();
                        let aspect = size.width as f32 / size.height.max(1) as f32;
                        let mouse = mouse_to_ui(input.mouse_ndc, aspect);
                        if menu_button(MenuButton::BackFromSettings).contains(mouse) {
                            enter_select(&mut app_state, &mut input, &window);
                        } else {
                            for idx in 0..3usize {
                                let track = slider_track_rect(idx);
                                // Expand hit area slightly for easier grabbing
                                let hit = Rect::new(track.x, track.y - 0.15, track.w, track.h + 0.3);
                                if hit.contains(mouse) {
                                    dragging_slider = Some(idx);
                                    let value = ((mouse.x - track.x) / track.w).clamp(0.0, 1.0);
                                    match idx {
                                        0 => sound_settings.master_volume = value,
                                        1 => sound_settings.music_volume = value,
                                        2 => sound_settings.sfx_volume = value,
                                        _ => {}
                                    }
                                    break;
                                }
                            }
                        }
                    } else if matches!(
                        app_state,
                        AppState::GameSelect
                            | AppState::SpaceMenu
                            | AppState::SnakeMenu
                            | AppState::FpsMenu
                            | AppState::PlatformerMenu
                            | AppState::TetrisMenu
                    ) {
                        let size = window.inner_size();
                        let aspect = size.width as f32 / size.height.max(1) as f32;
                        let mouse = mouse_to_ui(input.mouse_ndc, aspect);
                        if let Some(ref engine) = sound_engine {
                            engine.play(SoundEvent::MenuClick, &sound_settings);
                        }
                        match app_state {
                            AppState::GameSelect => {
                                if menu_button(MenuButton::SpaceShooter).contains(mouse) {
                                    enter_game_menu(
                                        &mut app_state,
                                        AppState::SpaceMenu,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Snake).contains(mouse) {
                                    enter_game_menu(
                                        &mut app_state,
                                        AppState::SnakeMenu,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Fps).contains(mouse) {
                                    enter_game_menu(
                                        &mut app_state,
                                        AppState::FpsMenu,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Platformer).contains(mouse) {
                                    enter_game_menu(
                                        &mut app_state,
                                        AppState::PlatformerMenu,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Tetris).contains(mouse) {
                                    enter_game_menu(
                                        &mut app_state,
                                        AppState::TetrisMenu,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Settings).contains(mouse) {
                                    app_state = AppState::Settings;
                                } else if menu_button(MenuButton::Quit).contains(mouse) {
                                    window_target.exit();
                                }
                            }
                            AppState::SpaceMenu => {
                                if menu_button(MenuButton::Start).contains(mouse) {
                                    enter_space_game(
                                        &mut app_state,
                                        &mut game,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Quit).contains(mouse) {
                                    enter_select(&mut app_state, &mut input, &window);
                                }
                            }
                            AppState::SnakeMenu => {
                                if menu_button(MenuButton::Start).contains(mouse) {
                                    enter_snake_game(
                                        &mut app_state,
                                        &mut snake,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Quit).contains(mouse) {
                                    enter_select(&mut app_state, &mut input, &window);
                                }
                            }
                            AppState::FpsMenu => {
                                if menu_button(MenuButton::Start).contains(mouse) {
                                    enter_fps_game(&mut app_state, &mut fps, &mut input, &window);
                                } else if menu_button(MenuButton::Quit).contains(mouse) {
                                    enter_select(&mut app_state, &mut input, &window);
                                }
                            }
                            AppState::PlatformerMenu => {
                                if menu_button(MenuButton::Start).contains(mouse) {
                                    enter_platformer_game(
                                        &mut app_state,
                                        &mut platformer,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Quit).contains(mouse) {
                                    enter_select(&mut app_state, &mut input, &window);
                                }
                            }
                            AppState::TetrisMenu => {
                                if menu_button(MenuButton::Start).contains(mouse) {
                                    enter_tetris_game(
                                        &mut app_state,
                                        &mut tetris,
                                        &mut input,
                                        &window,
                                    );
                                } else if menu_button(MenuButton::Quit).contains(mouse) {
                                    enter_select(&mut app_state, &mut input, &window);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        input.firing = true;
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => handle_keyboard_input(
                &event,
                &mut app_state,
                &mut show_help,
                &mut input,
                &mut game,
                &mut snake,
                &mut fps,
                &mut platformer,
                &mut tetris,
                &window,
                window_target,
            ),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - last_frame).as_secs_f32().min(1.0 / 20.0);
                last_frame = now;
                if app_state == AppState::SpacePlaying {
                    game.update(&mut input, dt, renderer.asteroid_mesh_count());
                } else if app_state == AppState::SnakePlaying {
                    snake.update(&mut input, dt);
                } else if app_state == AppState::FpsPlaying {
                    fps.update(&mut input, dt);
                } else if app_state == AppState::PlatformerPlaying {
                    platformer.update(&mut input, dt);
                } else if app_state == AppState::TetrisPlaying {
                    tetris.update(&mut input, dt);
                }

                if let Some(ref engine) = sound_engine {
                    for event in input.sound_events.drain(..) {
                        engine.play(event, &sound_settings);
                    }
                } else {
                    input.sound_events.clear();
                }

                let mut frame = display.draw();
                match app_state {
                    AppState::GameSelect => renderer.render_game_select(
                        &display,
                        &mut frame,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                        input.mouse_ndc,
                    ),
                    AppState::Settings => renderer.render_settings(
                        &display,
                        &mut frame,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                        input.mouse_ndc,
                        &sound_settings,
                    ),
                    AppState::SpaceMenu => renderer.render_menu(
                        &display,
                        &mut frame,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                        input.mouse_ndc,
                    ),
                    AppState::SpacePlaying => renderer.render(
                        &display,
                        &mut frame,
                        &game,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                    ),
                    AppState::SnakeMenu => renderer.render_snake_menu(
                        &display,
                        &mut frame,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                        input.mouse_ndc,
                    ),
                    AppState::SnakePlaying => renderer.render_snake(
                        &display,
                        &mut frame,
                        &snake,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                    ),
                    AppState::FpsMenu => renderer.render_fps_menu(
                        &display,
                        &mut frame,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                        input.mouse_ndc,
                    ),
                    AppState::FpsPlaying => renderer.render_fps(
                        &display,
                        &mut frame,
                        &fps,
                        display.get_framebuffer_dimensions(),
                    ),
                    AppState::PlatformerMenu => renderer.render_platformer_menu(
                        &display,
                        &mut frame,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                        input.mouse_ndc,
                    ),
                    AppState::PlatformerPlaying => renderer.render_platformer(
                        &display,
                        &mut frame,
                        &platformer,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                    ),
                    AppState::TetrisMenu => renderer.render_tetris_menu(
                        &display,
                        &mut frame,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                        input.mouse_ndc,
                    ),
                    AppState::TetrisPlaying => renderer.render_tetris(
                        &display,
                        &mut frame,
                        &tetris,
                        start.elapsed().as_secs_f32(),
                        display.get_framebuffer_dimensions(),
                    ),
                }
                if show_help {
                    renderer.render_help_dialog(
                        &display,
                        &mut frame,
                        display.get_framebuffer_dimensions(),
                        app_state,
                    );
                }
                frame.finish().expect("swap buffers");
            }
            WindowEvent::Resized(size) => {
                display.resize((size.width, size.height));
            }
            _ => {}
        },
        Event::AboutToWait => {
            window.request_redraw();
        }
        _ => {}
    });
}
