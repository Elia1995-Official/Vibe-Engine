use cgmath::{perspective, vec3, Deg, Matrix, Matrix4, Point3, SquareMatrix, Vector2, Vector3};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::{uniform, Depth, DepthTest, DrawParameters, Frame, Program, Surface};
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::app::{AppState, SoundSettings};
use crate::config::STAR_LAYERS;
use crate::fps_game::{FpsGame, MapType, FPS_MAP_SIZE};
use crate::starship_game::{Game, PickupKind};
use crate::graphics::{identity_transform, mat4, Mesh, StarVertex, Transform};
use crate::procedural::{
    arena_floor_mesh, bullet_mesh, cube_mesh, generate_stars, procedural_asteroid, procedural_ship,
    quad_mesh, ship_indices,
};
use crate::shaders::{
    SOLID_FRAGMENT_SHADER, SOLID_VERTEX_SHADER, STAR_FRAGMENT_SHADER, STAR_VERTEX_SHADER,
};
use crate::snake_game::{SnakeGame, SNAKE_ARENA_HALF_SIZE};
use crate::tetris_game::{
    TetrisGame, TETRIS_BOARD_H, TETRIS_BOARD_TOP, TETRIS_BOARD_W, TETRIS_BOARD_X,
    TETRIS_CELL_SIZE,
};
use crate::ui::{
    menu_button, mouse_to_ui, slider_fill_and_handle, slider_track_rect, text_height, text_mesh,
    text_width, ui_projection, MenuButton, Rect,
};

pub struct Renderer {
    solid_program: Program,
    star_program: Program,
    player_mesh: Mesh,
    bullet_mesh: Mesh,
    arena_mesh: Mesh,
    snake_cube_mesh: Mesh,
    food_cube_mesh: Mesh,
    bad_food_cube_mesh: Mesh,
    repair_cube_mesh: Mesh,
    rapid_cube_mesh: Mesh,
    shield_cube_mesh: Mesh,
    fps_wall_mesh: Mesh,
    fps_outdoor_wall_mesh: Mesh,
    fps_window_mesh: Mesh,
    fps_enemy_mesh: Mesh,
    fps_viewmodel_mesh: Mesh,
    fps_projectile_mesh: Mesh,
    platformer_goal_light_mesh: Mesh,
    platformer_goal_dark_mesh: Mesh,
    asteroid_meshes: Vec<Mesh>,
    star_buffers: Vec<glium::VertexBuffer<StarVertex>>,
    no_indices: NoIndices,
    fps_light_time: f32,
}

impl Renderer {
    pub fn new(display: &impl Facade) -> Self {
        let solid_program =
            Program::from_source(display, SOLID_VERTEX_SHADER, SOLID_FRAGMENT_SHADER, None)
                .expect("solid shader program");
        let star_program =
            Program::from_source(display, STAR_VERTEX_SHADER, STAR_FRAGMENT_SHADER, None)
                .expect("star shader program");

        let mut rng = StdRng::seed_from_u64(42);
        let player_mesh = Mesh::new(display, procedural_ship(&mut rng), ship_indices());
        let bullet_mesh = Mesh::new(
            display,
            bullet_mesh(),
            vec![0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6],
        );
        let (arena_vertices, arena_indices) =
            arena_floor_mesh(SNAKE_ARENA_HALF_SIZE, [0.035, 0.12, 0.105]);
        let arena_mesh = Mesh::new(display, arena_vertices, arena_indices);
        let (snake_vertices, snake_indices) = cube_mesh([0.22, 0.95, 0.48]);
        let snake_cube_mesh = Mesh::new(display, snake_vertices, snake_indices);
        let (food_vertices, food_indices) = cube_mesh([1.0, 0.22, 0.18]);
        let food_cube_mesh = Mesh::new(display, food_vertices, food_indices);
        let (bad_food_vertices, bad_food_indices) = cube_mesh([0.65, 0.18, 1.0]);
        let bad_food_cube_mesh = Mesh::new(display, bad_food_vertices, bad_food_indices);
        let (repair_vertices, repair_indices) = cube_mesh([0.28, 1.0, 0.42]);
        let repair_cube_mesh = Mesh::new(display, repair_vertices, repair_indices);
        let (rapid_vertices, rapid_indices) = cube_mesh([1.0, 0.84, 0.22]);
        let rapid_cube_mesh = Mesh::new(display, rapid_vertices, rapid_indices);
        let (shield_vertices, shield_indices) = cube_mesh([0.24, 0.62, 1.0]);
        let shield_cube_mesh = Mesh::new(display, shield_vertices, shield_indices);
        let (wall_vertices, wall_indices) = cube_mesh([0.18, 0.22, 0.28]);
        let fps_wall_mesh = Mesh::new(display, wall_vertices, wall_indices);
        let (outdoor_wall_vertices, outdoor_wall_indices) = cube_mesh([0.44, 0.34, 0.22]);
        let fps_outdoor_wall_mesh = Mesh::new(display, outdoor_wall_vertices, outdoor_wall_indices);
        let (window_vertices, window_indices) = cube_mesh([0.52, 0.78, 0.98]);
        let fps_window_mesh = Mesh::new(display, window_vertices, window_indices);
        let (enemy_vertices, enemy_indices) = cube_mesh([0.95, 0.18, 0.16]);
        let fps_enemy_mesh = Mesh::new(display, enemy_vertices, enemy_indices);
        let (viewmodel_vertices, viewmodel_indices) = cube_mesh([0.18, 0.18, 0.22]);
        let fps_viewmodel_mesh = Mesh::new(display, viewmodel_vertices, viewmodel_indices);
        let (projectile_vertices, projectile_indices) = cube_mesh([1.0, 0.82, 0.34]);
        let fps_projectile_mesh = Mesh::new(display, projectile_vertices, projectile_indices);
        let (goal_light_vertices, goal_light_indices) = cube_mesh([0.92, 0.86, 0.22]);
        let platformer_goal_light_mesh = Mesh::new(display, goal_light_vertices, goal_light_indices);
        let (goal_dark_vertices, goal_dark_indices) = cube_mesh([0.12, 0.12, 0.12]);
        let platformer_goal_dark_mesh = Mesh::new(display, goal_dark_vertices, goal_dark_indices);
        let asteroid_meshes = (0..8)
            .map(|i| {
                let (vertices, indices) = procedural_asteroid(18, 11, i as u64 * 113 + 7);
                Mesh::new(display, vertices, indices)
            })
            .collect();

        let star_buffers = (0..STAR_LAYERS)
            .map(|layer| {
                let stars = generate_stars(layer, &mut rng);
                glium::VertexBuffer::dynamic(display, &stars).expect("star buffer")
            })
            .collect();

        Self {
            solid_program,
            star_program,
            player_mesh,
            bullet_mesh,
            arena_mesh,
            snake_cube_mesh,
            food_cube_mesh,
            bad_food_cube_mesh,
            repair_cube_mesh,
            rapid_cube_mesh,
            shield_cube_mesh,
            fps_wall_mesh,
            fps_outdoor_wall_mesh,
            fps_window_mesh,
            fps_enemy_mesh,
            fps_viewmodel_mesh,
            fps_projectile_mesh,
            platformer_goal_light_mesh,
            platformer_goal_dark_mesh,
            asteroid_meshes,
            star_buffers,
            no_indices: NoIndices(PrimitiveType::Points),
            fps_light_time: 0.0,
        }
    }

    pub fn asteroid_mesh_count(&self) -> usize {
        self.asteroid_meshes.len()
    }

    pub fn render(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        game: &Game,
        elapsed: f32,
        dimensions: (u32, u32),
    ) {
        frame.clear_color_and_depth((0.006, 0.008, 0.018, 1.0), 1.0);

        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(58.0), aspect, 0.1, 160.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, -0.15, 10.6),
            Point3::new(0.0, -0.35, -15.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;

        self.draw_stars(frame, vp, elapsed);

        let player_tilt = vec3(
            -(game.player.target.y - game.player.position.y) * 0.09,
            (game.player.target.x - game.player.position.x) * 0.08,
            -(game.player.target.x - game.player.position.x) * 0.14,
        );
        self.draw_mesh(
            frame,
            &self.player_mesh,
            vp,
            Transform {
                position: game.player.position,
                rotation: player_tilt,
                scale: 0.82,
            },
            1.0,
        );

        for bullet in &game.bullets {
            self.draw_mesh(
                frame,
                &self.bullet_mesh,
                vp,
                Transform {
                    position: bullet.position,
                    rotation: vec3(0.0, 0.0, 0.0),
                    scale: 1.0,
                },
                1.0,
            );
        }

        for asteroid in &game.asteroids {
            let mesh = &self.asteroid_meshes[asteroid.mesh_id % self.asteroid_meshes.len()];
            self.draw_mesh(
                frame,
                mesh,
                vp,
                Transform {
                    position: asteroid.position,
                    rotation: asteroid.rotation,
                    scale: asteroid.radius,
                },
                1.0,
            );
        }
        for pickup in &game.pickups {
            let mesh = match pickup.kind {
                PickupKind::Repair => &self.repair_cube_mesh,
                PickupKind::RapidFire => &self.rapid_cube_mesh,
                PickupKind::Shield => &self.shield_cube_mesh,
            };
            self.draw_mesh(
                frame,
                mesh,
                vp,
                Transform {
                    position: pickup.position,
                    rotation: pickup.rotation,
                    scale: 0.42,
                },
                1.0,
            );
        }

        self.draw_hud(display, frame, game, aspect);
    }

    pub fn render_menu(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        elapsed: f32,
        dimensions: (u32, u32),
        mouse_ndc: Vector2<f32>,
    ) {
        frame.clear_color_and_depth((0.004, 0.006, 0.015, 1.0), 1.0);

        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(58.0), aspect, 0.1, 160.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, -0.4, 12.5),
            Point3::new(0.0, -0.2, -15.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;

        self.draw_stars(frame, vp, elapsed * 0.72);
        self.draw_mesh(
            frame,
            &self.player_mesh,
            vp,
            Transform {
                position: vec3(0.0, 0.15 + elapsed.sin() * 0.14, -9.2),
                rotation: vec3(0.18, elapsed * 0.45, elapsed.sin() * 0.08),
                scale: 0.92,
            },
            1.0,
        );
        let asteroid_mesh = &self.asteroid_meshes[0];
        self.draw_mesh(
            frame,
            asteroid_mesh,
            vp,
            Transform {
                position: vec3(-1.65, -0.15, -8.8),
                rotation: vec3(elapsed * 0.4, elapsed * 0.7, 0.0),
                scale: 0.55,
            },
            1.0,
        );
        self.draw_mesh(
            frame,
            &self.bullet_mesh,
            vp,
            Transform {
                position: vec3(1.45, -0.2, -8.6),
                rotation: vec3(0.0, elapsed * 0.8, 0.0),
                scale: 1.6,
            },
            1.0,
        );

        let ui = ui_projection(aspect);
        let start_hover = menu_button(MenuButton::Start).contains(mouse_to_ui(mouse_ndc, aspect));
        let quit_hover = menu_button(MenuButton::Quit).contains(mouse_to_ui(mouse_ndc, aspect));

        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-4.35, -3.05, 8.7, 6.1),
            [0.006, 0.018, 0.045],
            0.88,
        );

        self.draw_text_centered(
            display,
            frame,
            ui,
            "SPACE SHOOTER MENU",
            0.0,
            1.85,
            0.095,
            7.2,
            [0.45, 0.95, 1.0],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "DODGE ROCKS AND FIRE",
            0.0,
            1.1,
            0.055,
            6.6,
            [0.86, 0.94, 1.0],
        );

        self.draw_menu_button(
            display,
            frame,
            ui,
            MenuButton::Start,
            start_hover,
            "START GAME",
        );
        self.draw_menu_button(display, frame, ui, MenuButton::Quit, quit_hover, "BACK");

        self.draw_text_centered(
            display,
            frame,
            ui,
            "MOUSE STEERS   CLICK OR SPACE FIRES",
            0.0,
            -2.32,
            0.052,
            7.6,
            [0.52, 0.68, 0.82],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "ENTER OR SPACE STARTS   F1 HELP",
            0.0,
            -2.72,
            0.048,
            6.7,
            [0.42, 0.58, 0.72],
        );
    }

    pub fn render_game_select(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        elapsed: f32,
        dimensions: (u32, u32),
        mouse_ndc: Vector2<f32>,
    ) {
        frame.clear_color_and_depth((0.004, 0.006, 0.015, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(58.0), aspect, 0.1, 160.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, -0.4, 12.5),
            Point3::new(0.0, -0.2, -15.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;
        self.draw_stars(frame, vp, elapsed * 0.55);

        let ui = ui_projection(aspect);
        let mouse = mouse_to_ui(mouse_ndc, aspect);
        let space_hover = menu_button(MenuButton::SpaceShooter).contains(mouse);
        let snake_hover = menu_button(MenuButton::Snake).contains(mouse);
        let fps_hover = menu_button(MenuButton::Fps).contains(mouse);
        let platformer_hover = menu_button(MenuButton::Platformer).contains(mouse);
        let tetris_hover = menu_button(MenuButton::Tetris).contains(mouse);
        let quit_hover = menu_button(MenuButton::Quit).contains(mouse);
        let settings_hover = menu_button(MenuButton::Settings).contains(mouse);

        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-4.35, -3.68, 8.7, 6.73),
            [0.01, 0.025, 0.055],
            0.9,
        );
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-4.12, -3.45, 8.24, 6.27),
            [0.05, 0.13, 0.24],
            0.55,
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "VIBE ENGINE",
            0.0,
            4.0,
            0.11,
            7.0,
            [0.72, 0.96, 1.0],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "BY ELIA1995",
            0.0,
            3.30,
            0.065,
            6.8,
            [0.62, 0.86, 1.0],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "SELECT GAME",
            0.0,
            1.95,
            0.105,
            7.0,
            [0.62, 0.96, 1.0],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "CHOOSE YOUR DEMO",
            0.0,
            1.28,
            0.062,
            6.0,
            [0.72, 0.86, 1.0],
        );
        self.draw_menu_button(
            display,
            frame,
            ui,
            MenuButton::SpaceShooter,
            space_hover,
            "SPACE SHOOTER",
        );
        self.draw_menu_button(
            display,
            frame,
            ui,
            MenuButton::Snake,
            snake_hover,
            "3D SNAKE",
        );
        self.draw_menu_button(display, frame, ui, MenuButton::Fps, fps_hover, "FPS ARENA");
        self.draw_menu_button(
            display,
            frame,
            ui,
            MenuButton::Platformer,
            platformer_hover,
            "PLATFORMER",
        );
        self.draw_menu_button(display, frame, ui, MenuButton::Tetris, tetris_hover, "TETRIS");
        self.draw_menu_button(display, frame, ui, MenuButton::Quit, quit_hover, "QUIT");
        self.draw_menu_button(display, frame, ui, MenuButton::Settings, settings_hover, "SETTINGS");
        self.draw_text_centered(
            display,
            frame,
            ui,
            "CLICK A GAME   F1 HELP",
            0.0,
            -2.72,
            0.048,
            6.2,
            [0.42, 0.58, 0.72],
        );
    }

    pub fn render_snake_menu(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        elapsed: f32,
        dimensions: (u32, u32),
        mouse_ndc: Vector2<f32>,
    ) {
        frame.clear_color_and_depth((0.004, 0.012, 0.01, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(52.0), aspect, 0.1, 120.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 9.2, 12.5),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;
        self.draw_stars(frame, vp, elapsed * 0.35);
        self.draw_mesh(
            frame,
            &self.snake_cube_mesh,
            vp,
            Transform {
                position: vec3(-1.2, 0.0, 0.0),
                rotation: vec3(elapsed * 0.3, elapsed * 0.6, 0.0),
                scale: 1.25,
            },
            1.0,
        );
        self.draw_mesh(
            frame,
            &self.food_cube_mesh,
            vp,
            Transform {
                position: vec3(1.2, 0.0, 0.0),
                rotation: vec3(0.0, elapsed * 0.8, 0.0),
                scale: 0.85,
            },
            1.0,
        );

        let ui = ui_projection(aspect);
        let start_hover = menu_button(MenuButton::Start).contains(mouse_to_ui(mouse_ndc, aspect));
        let quit_hover = menu_button(MenuButton::Quit).contains(mouse_to_ui(mouse_ndc, aspect));
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-4.35, -3.05, 8.7, 6.1),
            [0.005, 0.035, 0.026],
            0.88,
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "3D SNAKE MENU",
            0.0,
            1.85,
            0.095,
            7.2,
            [0.45, 1.0, 0.65],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "GROW ON A CUBE GRID",
            0.0,
            1.1,
            0.055,
            6.6,
            [0.72, 0.95, 0.78],
        );
        self.draw_menu_button(
            display,
            frame,
            ui,
            MenuButton::Start,
            start_hover,
            "START SNAKE",
        );
        self.draw_menu_button(display, frame, ui, MenuButton::Quit, quit_hover, "BACK");
        self.draw_text_centered(
            display,
            frame,
            ui,
            "ARROWS OR WASD TURN   R RESTART",
            0.0,
            -2.35,
            0.048,
            7.0,
            [0.52, 0.78, 0.62],
        );
    }

    pub fn render_snake(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        snake: &SnakeGame,
        elapsed: f32,
        dimensions: (u32, u32),
    ) {
        frame.clear_color_and_depth((0.004, 0.012, 0.01, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(50.0), aspect, 0.1, 140.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 12.0, 13.5),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;
        self.draw_stars(frame, vp, elapsed * 0.22);

        self.draw_mesh(frame, &self.arena_mesh, vp, identity_transform(), 0.42);

        for i in 0..28 {
            let t = -SNAKE_ARENA_HALF_SIZE + i as f32 * (SNAKE_ARENA_HALF_SIZE * 2.0 / 27.0);
            for &(x, z) in &[
                (t, -SNAKE_ARENA_HALF_SIZE),
                (t, SNAKE_ARENA_HALF_SIZE),
                (-SNAKE_ARENA_HALF_SIZE, t),
                (SNAKE_ARENA_HALF_SIZE, t),
            ] {
                self.draw_mesh(
                    frame,
                    &self.food_cube_mesh,
                    vp,
                    Transform {
                        position: vec3(x, 0.16, z),
                        rotation: vec3(0.0, elapsed * 0.25, 0.0),
                        scale: 0.18,
                    },
                    0.9,
                );
            }
        }

        for (i, _) in snake.snake.iter().enumerate() {
            let (draw_x, draw_y) = snake.visual_segment_position(i);
            let growth = smoothstep(snake.visual_segment_scale(i));
            self.draw_mesh(
                frame,
                &self.snake_cube_mesh,
                vp,
                Transform {
                    position: vec3(draw_x, 0.55, draw_y),
                    rotation: vec3(0.0, -snake.visual_segment_yaw(i), 0.0),
                    scale: if i == 0 { 0.92 } else { 0.78 * growth },
                },
                1.0,
            );
        }

        self.draw_mesh(
            frame,
            &self.food_cube_mesh,
            vp,
            Transform {
                position: vec3(snake.food.0, 0.56 + elapsed.sin() * 0.08, snake.food.1),
                rotation: vec3(elapsed * 0.7, elapsed * 1.2, 0.0),
                scale: 0.74,
            },
            1.0,
        );
        self.draw_mesh(
            frame,
            &self.bad_food_cube_mesh,
            vp,
            Transform {
                position: vec3(
                    snake.bad_food.0,
                    0.5 + (elapsed * 1.7).sin() * 0.06,
                    snake.bad_food.1,
                ),
                rotation: vec3(elapsed * 1.1, -elapsed * 1.5, elapsed * 0.4),
                scale: 0.64 + (elapsed * 4.0).sin().abs() * 0.08,
            },
            1.0,
        );

        self.draw_snake_hud(display, frame, snake, aspect);
    }

    pub fn render_fps_menu(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        elapsed: f32,
        dimensions: (u32, u32),
        mouse_ndc: Vector2<f32>,
    ) {
        frame.clear_color_and_depth((0.006, 0.008, 0.014, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(56.0), aspect, 0.1, 120.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 5.2, 10.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;
        self.draw_stars(frame, vp, elapsed * 0.35);
        for i in -3..=3 {
            self.draw_mesh(
                frame,
                &self.fps_wall_mesh,
                vp,
                Transform {
                    position: vec3(i as f32, 0.0, -2.0),
                    rotation: vec3(0.0, 0.0, 0.0),
                    scale: 0.95,
                },
                1.0,
            );
        }
        self.draw_mesh(
            frame,
            &self.fps_enemy_mesh,
            vp,
            Transform {
                position: vec3(0.0, 0.15 + elapsed.sin() * 0.1, 0.8),
                rotation: vec3(0.0, elapsed, 0.0),
                scale: 0.7,
            },
            1.0,
        );

        let ui = ui_projection(aspect);
        let start_hover = menu_button(MenuButton::Start).contains(mouse_to_ui(mouse_ndc, aspect));
        let quit_hover = menu_button(MenuButton::Quit).contains(mouse_to_ui(mouse_ndc, aspect));
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-4.35, -3.05, 8.7, 6.1),
            [0.012, 0.014, 0.025],
            0.88,
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "FPS ARENA MENU",
            0.0,
            1.85,
            0.095,
            7.2,
            [0.98, 0.74, 0.52],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "PROCEDURAL MAPS AND ENEMIES",
            0.0,
            1.1,
            0.052,
            7.0,
            [0.9, 0.82, 0.72],
        );
        self.draw_menu_button(
            display,
            frame,
            ui,
            MenuButton::Start,
            start_hover,
            "START FPS",
        );
        self.draw_menu_button(display, frame, ui, MenuButton::Quit, quit_hover, "BACK");
        self.draw_text_centered(
            display,
            frame,
            ui,
            "WASD MOVE   MOUSE LOOK   CLICK SHOOT",
            0.0,
            -2.35,
            0.048,
            7.2,
            [0.78, 0.76, 0.72],
        );
    }

    pub fn render_fps(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        fps: &FpsGame,
        dimensions: (u32, u32),
    ) {
        self.fps_light_time += 0.018;
        let sky_phase = (self.fps_light_time * 0.22).sin() * 0.5 + 0.5;
        let sky_color = [
            0.16 + 0.42 * sky_phase,
            0.25 + 0.50 * sky_phase,
            0.42 + 0.48 * sky_phase,
        ];
        let (sky_r, sky_g, sky_b) = match fps.map_type {
            MapType::Indoor => (
                sky_color[0] * 0.08,
                sky_color[1] * 0.09,
                sky_color[2] * 0.12,
            ),
            MapType::Outdoor => (sky_color[0], sky_color[1], sky_color[2]),
        };
        frame.clear_color_and_depth((sky_r, sky_g, sky_b, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(72.0), aspect, 0.05, 80.0);
        let eye = Point3::new(
            fps.player_x - FPS_MAP_SIZE as f32 * 0.5,
            0.62,
            fps.player_z - FPS_MAP_SIZE as f32 * 0.5,
        );
        let dir = Vector3::new(
            fps.yaw.cos() * fps.pitch.cos(),
            fps.pitch.sin(),
            fps.yaw.sin() * fps.pitch.cos(),
        );
        let view = Matrix4::look_at_rh(eye, eye + dir, Vector3::unit_y());
        let vp = projection * view;

        let offset = FPS_MAP_SIZE as f32 * 0.5;
        let mut indoor_light_pos = [eye.x, 1.25, eye.z];
        let sky_luma = sky_color[0] * 0.2126 + sky_color[1] * 0.7152 + sky_color[2] * 0.0722;
        let mut indoor_light_color = [
            (sky_color[0] * 1.12).clamp(0.0, 1.0),
            (sky_color[1] * 1.08).clamp(0.0, 1.0),
            (sky_color[2] * 1.02).clamp(0.0, 1.0),
        ];
        let mut indoor_light_strength = 0.0;
        if fps.map_type == MapType::Indoor {
            let mut best_dist2 = f32::MAX;
            for z in 0..FPS_MAP_SIZE {
                for x in 0..FPS_MAP_SIZE {
                    if fps.is_indoor_window(x, z) {
                        let wx = x as f32 + 0.5 - offset;
                        let wz = z as f32 + 0.5 - offset;
                        let dx = wx - eye.x;
                        let dz = wz - eye.z;
                        let d2 = dx * dx + dz * dz;
                        if d2 < best_dist2 {
                            best_dist2 = d2;
                            indoor_light_pos = [wx, 1.15, wz];
                        }
                    }
                }
            }
            indoor_light_strength = 1.2 + sky_luma * 2.4;
            // Small sky flicker keeps window light dynamic without looking artificial.
            indoor_light_strength *= 0.92 + 0.08 * (self.fps_light_time * 1.3).sin();
            indoor_light_color = [
                indoor_light_color[0],
                indoor_light_color[1],
                indoor_light_color[2],
            ];
        }

        self.draw_mesh_with_light(
            frame,
            &self.arena_mesh,
            vp,
            Transform {
                position: vec3(0.0, -0.52, 0.0),
                rotation: vec3(0.0, 0.0, 0.0),
                scale: 1.18,
            },
            0.65,
            indoor_light_pos,
            indoor_light_color,
            indoor_light_strength,
            self.fps_light_time,
        );

        if fps.map_type == MapType::Indoor {
            for z in 0..FPS_MAP_SIZE {
                for x in 0..FPS_MAP_SIZE {
                    let world_x = x as f32 + 0.5 - offset;
                    let world_z = z as f32 + 0.5 - offset;
                    if fps.is_wall(x, z) {
                        let is_window = fps.is_indoor_window(x, z);
                        for layer in 0..3 {
                            let draw_layer = if is_window {
                                layer != 1
                            } else {
                                true
                            };
                            if draw_layer {
                                self.draw_mesh_with_light(
                                    frame,
                                    &self.fps_wall_mesh,
                                    vp,
                                    Transform {
                                        position: vec3(world_x, layer as f32, world_z),
                                        rotation: vec3(0.0, 0.0, 0.0),
                                        scale: 1.0,
                                    },
                                    1.0,
                                    indoor_light_pos,
                                    indoor_light_color,
                                    indoor_light_strength,
                                    self.fps_light_time,
                                );
                            }
                        }
                        if is_window {
                            self.draw_mesh_with_light(
                                frame,
                                &self.fps_window_mesh,
                                vp,
                                Transform {
                                    position: vec3(world_x, 1.0, world_z),
                                    rotation: vec3(0.0, 0.0, 0.0),
                                    scale: 0.92,
                                },
                                0.32 + sky_luma * 0.22,
                                indoor_light_pos,
                                indoor_light_color,
                                indoor_light_strength,
                                self.fps_light_time,
                            );
                        }
                    } else {
                        // Solid ceiling over playable indoor space.
                        self.draw_mesh_with_light(
                            frame,
                            &self.fps_wall_mesh,
                            vp,
                            Transform {
                                position: vec3(world_x, 3.0, world_z),
                                rotation: vec3(0.0, 0.0, 0.0),
                                scale: 1.0,
                            },
                            0.98,
                            indoor_light_pos,
                            indoor_light_color,
                            indoor_light_strength,
                            self.fps_light_time,
                        );
                    }
                }
            }
            // Doorways are passable cells with an overhead lintel cube.
            for z in 0..FPS_MAP_SIZE {
                for x in 0..FPS_MAP_SIZE {
                    if fps.is_indoor_door(x, z) {
                        self.draw_mesh_with_light(
                            frame,
                            &self.fps_wall_mesh,
                            vp,
                            Transform {
                                position: vec3(x as f32 + 0.5 - offset, 2.0, z as f32 + 0.5 - offset),
                                rotation: vec3(0.0, 0.0, 0.0),
                                scale: 1.0,
                            },
                            1.0,
                            indoor_light_pos,
                            indoor_light_color,
                            indoor_light_strength,
                            self.fps_light_time,
                        );
                    }
                }
            }
        } else {
            for z in 0..FPS_MAP_SIZE {
                for x in 0..FPS_MAP_SIZE {
                    if fps.is_wall(x, z) {
                        self.draw_mesh_with_light(
                            frame,
                            &self.fps_outdoor_wall_mesh,
                            vp,
                            Transform {
                                position: vec3(x as f32 + 0.5 - offset, 0.0, z as f32 + 0.5 - offset),
                                rotation: vec3(0.0, 0.0, 0.0),
                                scale: 1.0,
                            },
                            1.0,
                            indoor_light_pos,
                            indoor_light_color,
                            0.0,
                            self.fps_light_time,
                        );
                    }
                }
            }
        }
        for enemy in &fps.enemies {
            self.draw_mesh_with_light(
                frame,
                &self.fps_enemy_mesh,
                vp,
                Transform {
                    position: vec3(enemy.x - offset, -0.02, enemy.z - offset),
                    rotation: vec3(0.0, enemy.heading, 0.0),
                    scale: 0.62,
                },
                1.0,
                indoor_light_pos,
                indoor_light_color,
                indoor_light_strength,
                self.fps_light_time,
            );
        }

        for projectile in &fps.projectiles {
            self.draw_mesh_with_light(
                frame,
                &self.fps_projectile_mesh,
                vp,
                Transform {
                    position: vec3(projectile.x - offset, 0.1, projectile.z - offset),
                    rotation: vec3(0.0, 0.0, 0.0),
                    scale: 0.08,
                },
                1.0,
                indoor_light_pos,
                indoor_light_color,
                indoor_light_strength,
                self.fps_light_time,
            );
        }

        // Render the FPS viewmodel directly in camera space so it never spins with world movement.
        let viewmodel_vp = projection;
        let gun_base = vec3(0.34, -0.24, -0.62);
        self.draw_mesh_with_light(
            frame,
            &self.fps_viewmodel_mesh,
            viewmodel_vp,
            Transform {
                position: gun_base,
                rotation: vec3(0.1, -0.46, 0.2),
                scale: 0.22,
            },
            1.0,
            indoor_light_pos,
            indoor_light_color,
            indoor_light_strength,
            self.fps_light_time,
        );
        self.draw_mesh_with_light(
            frame,
            &self.fps_viewmodel_mesh,
            viewmodel_vp,
            Transform {
                position: gun_base + vec3(-0.03, 0.03, -0.22),
                rotation: vec3(0.12, -0.46, 0.18),
                scale: 0.12,
            },
            1.0,
            indoor_light_pos,
            indoor_light_color,
            indoor_light_strength,
            self.fps_light_time,
        );

        if fps.shot_feedback > 0.0 {
            let flash_alpha = (fps.shot_feedback * 0.95).clamp(0.0, 1.0);
            self.draw_mesh_with_light(
                frame,
                &self.fps_projectile_mesh,
                viewmodel_vp,
                Transform {
                    position: gun_base + vec3(-0.035, 0.032, -0.305),
                    rotation: vec3(0.0, 0.0, 0.0),
                    scale: 0.06 + fps.shot_feedback * 0.03,
                },
                flash_alpha,
                indoor_light_pos,
                indoor_light_color,
                indoor_light_strength,
                self.fps_light_time,
            );
        }

        self.draw_fps_hud(display, frame, fps, aspect);
    }

    pub fn render_platformer_menu(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        elapsed: f32,
        dimensions: (u32, u32),
        mouse_ndc: Vector2<f32>,
    ) {
        frame.clear_color_and_depth((0.006, 0.008, 0.014, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(56.0), aspect, 0.1, 120.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 5.2, 10.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;
        self.draw_stars(frame, vp, elapsed * 0.35);
        for i in 0..5 {
            self.draw_mesh(
                frame,
                &self.snake_cube_mesh,
                vp,
                Transform {
                    position: vec3(i as f32 * 1.2 - 2.4, i as f32 * 0.35 - 1.0, -2.0),
                    rotation: vec3(0.0, elapsed + i as f32 * 0.4, 0.0),
                    scale: 0.55 + i as f32 * 0.08,
                },
                0.8,
            );
        }

        let ui = ui_projection(aspect);
        let start_hover = menu_button(MenuButton::Start).contains(mouse_to_ui(mouse_ndc, aspect));
        let quit_hover = menu_button(MenuButton::Quit).contains(mouse_to_ui(mouse_ndc, aspect));
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-4.35, -3.05, 8.7, 6.1),
            [0.012, 0.025, 0.014],
            0.88,
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "3D PLATFORMER MENU",
            0.0,
            1.85,
            0.095,
            7.2,
            [0.52, 0.98, 0.74],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "JUMP BETWEEN FLOATING PLATFORMS",
            0.0,
            1.1,
            0.052,
            7.0,
            [0.72, 0.9, 0.82],
        );
        self.draw_menu_button(
            display,
            frame,
            ui,
            MenuButton::Start,
            start_hover,
            "START PLATFORMER",
        );
        self.draw_menu_button(display, frame, ui, MenuButton::Quit, quit_hover, "BACK");
        self.draw_text_centered(
            display,
            frame,
            ui,
            "WASD MOVE   SPACE JUMP   MOUSE LOOK",
            0.0,
            -2.35,
            0.048,
            7.2,
            [0.78, 0.76, 0.72],
        );
    }

    pub fn render_settings(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        elapsed: f32,
        dimensions: (u32, u32),
        mouse_ndc: Vector2<f32>,
        sound: &SoundSettings,
    ) {
        frame.clear_color_and_depth((0.004, 0.006, 0.015, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(58.0), aspect, 0.1, 160.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, -0.4, 12.5),
            Point3::new(0.0, -0.2, -15.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;
        self.draw_stars(frame, vp, elapsed * 0.55);

        let ui = ui_projection(aspect);
        let mouse = mouse_to_ui(mouse_ndc, aspect);
        let back_hover = menu_button(MenuButton::BackFromSettings).contains(mouse);

        // Background panel
        self.draw_ui_rect(
            display, frame, ui,
            Rect::new(-4.35, -4.0, 8.7, 7.65),
            [0.01, 0.025, 0.055],
            0.9,
        );
        self.draw_ui_rect(
            display, frame, ui,
            Rect::new(-4.12, -3.77, 8.24, 7.21),
            [0.05, 0.13, 0.24],
            0.55,
        );

        self.draw_text_centered(display, frame, ui, "SETTINGS", 0.0, 2.82, 0.105, 7.0, [0.62, 0.96, 1.0]);
        self.draw_text_centered(display, frame, ui, "AUDIO", 0.0, 2.18, 0.068, 6.5, [0.72, 0.86, 1.0]);

        let slider_labels = ["MASTER VOLUME", "MUSIC VOLUME", "SFX VOLUME"];
        let slider_values = [sound.master_volume, sound.music_volume, sound.sfx_volume];

        for (i, (&label, &value)) in slider_labels.iter().zip(slider_values.iter()).enumerate() {
            let track = slider_track_rect(i);
            let label_y = track.y + track.h + 0.28;

            // Label and percentage
            self.draw_text_centered(display, frame, ui, label, -0.8, label_y, 0.058, 5.0, [0.72, 0.88, 1.0]);
            let pct = format!("{:3.0}%", value * 100.0);
            self.draw_text(display, frame, ui, &pct, track.x + track.w + 0.18, track.y + 0.02, 0.062, [0.92, 0.96, 1.0]);

            // Track background
            self.draw_ui_rect(display, frame, ui, track, [0.055, 0.12, 0.2], 0.95);

            // Filled portion
            let (fill, handle) = slider_fill_and_handle(track, value);
            self.draw_ui_rect(display, frame, ui, fill, [0.16, 0.54, 0.68], 1.0);

            // Handle knob
            let knob_hovered = handle.contains(mouse);
            let knob_color = if knob_hovered { [0.5, 0.96, 1.0] } else { [0.78, 0.94, 1.0] };
            self.draw_ui_rect(display, frame, ui, handle, knob_color, 1.0);
        }

        // Back button
        self.draw_menu_button(display, frame, ui, MenuButton::BackFromSettings, back_hover, "BACK");
    }

    pub fn render_platformer(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        platformer: &crate::platformer_game::PlatformerGame,
        _elapsed: f32,
        dimensions: (u32, u32),
    ) {
        frame.clear_color_and_depth((0.15, 0.18, 0.22, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(68.0), aspect, 0.05, 100.0);

        // Third-person camera positioned behind and above the player
        let forward = Vector3::new(platformer.camera_yaw.cos(), 0.0, platformer.camera_yaw.sin());
        let right = Vector3::new(-forward.z, 0.0, forward.x);
        let camera_dist = 3.5;
        let camera_height = 2.2;
        let eye = Point3::new(
            platformer.player_x - forward.x * camera_dist + right.x * 0.8,
            platformer.player_y + camera_height,
            platformer.player_z - forward.z * camera_dist + right.z * 0.8,
        );
        let target = Point3::new(
            platformer.player_x,
            platformer.player_y + 0.6,
            platformer.player_z,
        );
        let view = Matrix4::look_at_rh(eye, target, Vector3::unit_y());
        let vp = projection * view;

        // Render floor
        self.draw_mesh(
            frame,
            &self.arena_mesh,
            vp,
            Transform {
                position: vec3(0.0, -0.6, 0.0),
                rotation: vec3(0.0, 0.0, 0.0),
                scale: 1.8,
            },
            0.32,
        );

        // Render platforms
        let goal_index = platformer.platforms.len().saturating_sub(1);
        for (index, platform) in platformer.platforms.iter().enumerate() {
            self.draw_mesh(
                frame,
                &self.snake_cube_mesh,
                vp,
                Transform {
                    position: vec3(platform.x, platform.y, platform.z),
                    rotation: vec3(0.0, 0.0, 0.0),
                    scale: 1.0,
                },
                0.92,
            );

            if index == goal_index {
                let grid = 4;
                let tile_scale = 0.12;
                let step_x = platform.width / grid as f32;
                let step_z = platform.depth / grid as f32;
                let base_y = platform.y + platform.height + 0.11;
                for gz in 0..grid {
                    for gx in 0..grid {
                        let checker = (gx + gz) % 2 == 0;
                        let mesh = if checker {
                            &self.platformer_goal_light_mesh
                        } else {
                            &self.platformer_goal_dark_mesh
                        };
                        self.draw_mesh(
                            frame,
                            mesh,
                            vp,
                            Transform {
                                position: vec3(
                                    platform.x - platform.width * 0.5
                                        + step_x * (gx as f32 + 0.5),
                                    base_y,
                                    platform.z - platform.depth * 0.5
                                        + step_z * (gz as f32 + 0.5),
                                ),
                                rotation: vec3(0.0, 0.0, 0.0),
                                scale: tile_scale,
                            },
                            1.0,
                        );
                    }
                }
            }
        }

        // Render player
        self.draw_mesh(
            frame,
            &self.snake_cube_mesh,
            vp,
            Transform {
                position: vec3(platformer.player_x, platformer.player_y, platformer.player_z),
                rotation: vec3(0.0, platformer.camera_yaw, 0.0),
                scale: 0.65,
            },
            0.78,
        );

        // Draw platformer HUD
        let ui = ui_projection(aspect);
        self.draw_text_centered(
            display,
            frame,
            ui,
            &format!("LEVEL {} | SCORE {}", platformer.level, platformer.score),
            0.0,
            4.5,
            0.062,
            7.0,
            [0.52, 0.98, 0.74],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "REACH THE GREEN PLATFORM TO ADVANCE",
            0.0,
            3.9,
            0.038,
            7.0,
            [0.78, 0.76, 0.72],
        );

        if platformer.game_over {
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(-3.0, -1.5, 6.0, 3.0),
                [0.0, 0.0, 0.0],
                0.75,
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "FELL OFF!",
                0.0,
                0.45,
                0.12,
                7.2,
                [0.98, 0.52, 0.52],
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "Press R to restart",
                0.0,
                -0.45,
                0.048,
                7.0,
                [0.78, 0.76, 0.72],
            );
        }
    }

    pub fn render_tetris_menu(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        elapsed: f32,
        dimensions: (u32, u32),
        mouse_ndc: Vector2<f32>,
    ) {
        frame.clear_color_and_depth((0.008, 0.01, 0.02, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let projection = perspective(Deg(54.0), aspect, 0.1, 120.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 4.9, 10.2),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        let vp = projection * view;
        self.draw_stars(frame, vp, elapsed * 0.42);

        for i in 0..6 {
            self.draw_mesh(
                frame,
                &self.fps_projectile_mesh,
                vp,
                Transform {
                    position: vec3(i as f32 * 0.7 - 1.75, -0.7 + i as f32 * 0.22, -2.3),
                    rotation: vec3(elapsed * 0.25, elapsed * 0.48 + i as f32, 0.0),
                    scale: 0.28,
                },
                0.95,
            );
        }

        let ui = ui_projection(aspect);
        let start_hover = menu_button(MenuButton::Start).contains(mouse_to_ui(mouse_ndc, aspect));
        let quit_hover = menu_button(MenuButton::Quit).contains(mouse_to_ui(mouse_ndc, aspect));
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-4.35, -3.05, 8.7, 6.1),
            [0.018, 0.02, 0.045],
            0.88,
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "TETRIS MENU",
            0.0,
            1.85,
            0.095,
            7.2,
            [0.72, 0.8, 1.0],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "STACK CLEAN LINES IN 3D VIBE STYLE",
            0.0,
            1.1,
            0.052,
            7.2,
            [0.84, 0.9, 1.0],
        );
        self.draw_menu_button(
            display,
            frame,
            ui,
            MenuButton::Start,
            start_hover,
            "START TETRIS",
        );
        self.draw_menu_button(display, frame, ui, MenuButton::Quit, quit_hover, "BACK");
        self.draw_text_centered(
            display,
            frame,
            ui,
            "MOUSE MOVE AIMS COLUMN   LEFT ROTATE   RIGHT HARD DROP",
            0.0,
            -2.28,
            0.043,
            8.0,
            [0.72, 0.78, 0.9],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "MIDDLE HOLD   KEYBOARD ALSO WORKS   G TOGGLE GHOST",
            0.0,
            -2.66,
            0.043,
            7.8,
            [0.64, 0.72, 0.84],
        );
    }

    pub fn render_tetris(
        &mut self,
        display: &impl Facade,
        frame: &mut Frame,
        tetris: &TetrisGame,
        _elapsed: f32,
        dimensions: (u32, u32),
    ) {
        frame.clear_color_and_depth((0.008, 0.012, 0.022, 1.0), 1.0);
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let ui = ui_projection(aspect);

        let cell = TETRIS_CELL_SIZE;
        let board_w = TETRIS_BOARD_W as f32 * cell;
        let board_h = TETRIS_BOARD_H as f32 * cell;
        let board_x = TETRIS_BOARD_X;
        let board_top = TETRIS_BOARD_TOP;

        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(board_x - 0.34, board_top - board_h - 0.34, board_w + 0.68, board_h + 0.68),
            [0.03, 0.04, 0.08],
            0.96,
        );

        for y in 0..TETRIS_BOARD_H {
            for x in 0..TETRIS_BOARD_W {
                let px = board_x + x as f32 * cell + 0.01;
                let py = board_top - ((TETRIS_BOARD_H - y) as f32) * cell + 0.01;
                let rect = Rect::new(px, py, cell - 0.02, cell - 0.02);
                let id = tetris.board_cell(x as i32, y as i32);
                if id == 0 {
                    let empty = if (x + y) % 2 == 0 {
                        [0.065, 0.08, 0.115]
                    } else {
                        [0.05, 0.064, 0.096]
                    };
                    self.draw_ui_rect(display, frame, ui, rect, empty, 0.9);
                } else {
                    self.draw_ui_rect(display, frame, ui, rect, tetris_color(id), 1.0);
                }
            }
        }

        if let Some(ghost_cells) = tetris.ghost_cells() {
            for (x, y) in ghost_cells {
                if y < 0 {
                    continue;
                }
                let px = board_x + x as f32 * cell + 0.01;
                let py = board_top - ((TETRIS_BOARD_H as i32 - y) as f32) * cell + 0.01;
                self.draw_ui_rect(
                    display,
                    frame,
                    ui,
                    Rect::new(px, py, cell - 0.02, cell - 0.02),
                    [0.78, 0.82, 0.9],
                    0.28,
                );
            }
        }

        for (x, y) in tetris.active_cells() {
            if y < 0 {
                continue;
            }
            let px = board_x + x as f32 * cell + 0.01;
            let py = board_top - ((TETRIS_BOARD_H as i32 - y) as f32) * cell + 0.01;
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(px, py, cell - 0.02, cell - 0.02),
                tetris_color(tetris.active_color()),
                1.0,
            );
        }

        let side_x = board_x + board_w + 0.65;
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(side_x, -2.75, 3.1, 6.4),
            [0.02, 0.03, 0.065],
            0.95,
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "NEXT",
            side_x + 1.55,
            2.86,
            0.062,
            2.2,
            [0.82, 0.92, 1.0],
        );

        let preview_cell = 0.26;
        let preview_x = side_x + 0.86;
        let preview_y = 1.55;
        for (x, y) in tetris.next_cells() {
            let rx = preview_x + (x as f32 + 1.5) * preview_cell;
            let ry = preview_y + (y as f32 + 1.5) * preview_cell;
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(rx, ry, preview_cell - 0.02, preview_cell - 0.02),
                tetris_color(tetris.next_color()),
                1.0,
            );
        }

        self.draw_text_centered(
            display,
            frame,
            ui,
            "HOLD",
            side_x + 1.55,
            0.62,
            0.062,
            2.2,
            [0.82, 0.92, 1.0],
        );
        if let (Some(cells), Some(color)) = (tetris.hold_cells(), tetris.hold_color()) {
            let hold_x = side_x + 0.86;
            let hold_y = -0.68;
            for (x, y) in cells {
                let rx = hold_x + (x as f32 + 1.5) * preview_cell;
                let ry = hold_y + (y as f32 + 1.5) * preview_cell;
                self.draw_ui_rect(
                    display,
                    frame,
                    ui,
                    Rect::new(rx, ry, preview_cell - 0.02, preview_cell - 0.02),
                    tetris_color(color),
                    1.0,
                );
            }
        }

        self.draw_text(
            display,
            frame,
            ui,
            &format!("SCORE {}", tetris.score),
            side_x + 0.42,
            1.0,
            0.054,
            [0.9, 0.98, 1.0],
        );
        self.draw_text(
            display,
            frame,
            ui,
            &format!("LINES {}", tetris.lines),
            side_x + 0.42,
            0.55,
            0.054,
            [0.78, 0.9, 1.0],
        );
        self.draw_text(
            display,
            frame,
            ui,
            &format!("LEVEL {}", tetris.level),
            side_x + 0.42,
            0.02,
            0.054,
            [0.72, 0.86, 1.0],
        );
        self.draw_text(
            display,
            frame,
            ui,
            if tetris.ghost_enabled() {
                "GHOST ON"
            } else {
                "GHOST OFF"
            },
            side_x + 0.42,
            -0.44,
            0.048,
            [0.64, 0.8, 0.94],
        );

        self.draw_text_centered(
            display,
            frame,
            ui,
            "MOUSE MOVE AIMS COLUMN   LEFT ROTATE   MIDDLE HOLD",
            0.0,
            -4.55,
            0.041,
            8.6,
            [0.66, 0.78, 0.92],
        );
        self.draw_text_centered(
            display,
            frame,
            ui,
            "RIGHT CLICK HARD DROP   OR USE KEYBOARD CONTROLS",
            0.0,
            -4.88,
            0.041,
            8.6,
            [0.58, 0.7, 0.86],
        );

        if tetris.game_over {
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(-3.0, -1.35, 6.0, 2.7),
                [0.0, 0.0, 0.0],
                0.78,
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "STACK LOCKED",
                0.0,
                0.4,
                0.1,
                6.0,
                [1.0, 0.58, 0.58],
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "PRESS R TO RESTART",
                0.0,
                -0.38,
                0.048,
                5.6,
                [0.84, 0.92, 1.0],
            );
        }
    }

    pub fn render_help_dialog(
        &self,
        display: &impl Facade,
        frame: &mut Frame,
        dimensions: (u32, u32),
        app_state: AppState,
    ) {
        let aspect = dimensions.0 as f32 / dimensions.1.max(1) as f32;
        let ui = ui_projection(aspect);
        let panel = Rect::new(-3.65, -2.55, 7.3, 5.1);

        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-20.0, -20.0, 40.0, 40.0),
            [0.0, 0.0, 0.0],
            0.45,
        );
        self.draw_ui_rect(display, frame, ui, panel, [0.01, 0.022, 0.045], 0.96);
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(
                panel.x + 0.18,
                panel.y + 0.18,
                panel.w - 0.36,
                panel.h - 0.36,
            ),
            [0.045, 0.115, 0.18],
            0.82,
        );
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(
                panel.x + 0.18,
                panel.y + panel.h - 0.28,
                panel.w - 0.36,
                0.08,
            ),
            [0.36, 0.88, 1.0],
            1.0,
        );

        self.draw_text_centered(
            display,
            frame,
            ui,
            "HELP",
            0.0,
            1.55,
            0.09,
            3.5,
            [0.62, 0.96, 1.0],
        );

        let rows = match app_state {
            AppState::GameSelect => [
                "CLICK           SELECT GAME",
                "ENTER           SPACE SHOOTER",
                "S               3D SNAKE",
                "F               FPS ARENA",
                "T               TETRIS",
            ],
            AppState::SpaceMenu => [
                "ENTER           START GAME",
                "CLICK           SELECT BUTTONS",
                "ESC             GAME SELECT",
                "",
                "",
            ],
            AppState::SpacePlaying => [
                "MOUSE           STEER SHIP",
                "LEFT CLICK      FIRE",
                "SPACE           FIRE",
                "R               RESTART GAME OVER",
                "ESC             RETURN TO MENU",
            ],
            AppState::SnakeMenu => [
                "ENTER           START SNAKE",
                "CLICK           SELECT BUTTONS",
                "ESC             GAME SELECT",
                "",
                "",
            ],
            AppState::SnakePlaying => [
                "ARROWS WASD     TURN",
                "R               RESTART GAME OVER",
                "ESC             RETURN TO MENU",
                "",
                "",
            ],
            AppState::FpsMenu => [
                "ENTER           START FPS",
                "CLICK           SELECT BUTTONS",
                "ESC             GAME SELECT",
                "",
                "",
            ],
            AppState::FpsPlaying => [
                "WASD            MOVE",
                "MOUSE           LOOK",
                "LEFT CLICK      SHOOT",
                "R               RESTART GAME OVER",
                "ESC             RETURN TO MENU",
            ],
            AppState::PlatformerMenu => [
                "ENTER           START PLATFORMER",
                "CLICK           SELECT BUTTONS",
                "ESC             GAME SELECT",
                "",
                "",
            ],
            AppState::PlatformerPlaying => [
                "WASD            MOVE",
                "SPACE           JUMP",
                "MOUSE           LOOK AROUND",
                "R               RESTART ON FALL",
                "ESC             RETURN TO MENU",
            ],
            AppState::TetrisMenu => [
                "ENTER           START TETRIS",
                "MOUSE           AIM COLUMN IN GAME",
                "CLICK           SELECT BUTTONS",
                "ESC             GAME SELECT",
                "",
            ],
            AppState::TetrisPlaying => [
                "MOUSE MOVE      AIM COLUMN",
                "LEFT CLICK      ROTATE",
                "RIGHT CLICK     HARD DROP",
                "MIDDLE CLICK    HOLD PIECE",
                "ESC             RETURN TO MENU",
            ],
            AppState::Settings => [
                "DRAG SLIDERS    ADJUST VOLUME",
                "BACK            RETURN TO MENU",
                "",
                "",
                "",
            ],
        };

        for (i, row) in rows.iter().enumerate() {
            self.draw_text_centered(
                display,
                frame,
                ui,
                row,
                0.0,
                0.95 - i as f32 * 0.52,
                0.048,
                6.4,
                [0.88, 0.96, 1.0],
            );
        }

        self.draw_text_centered(
            display,
            frame,
            ui,
            "PRESS ESC TO CLOSE",
            0.0,
            -2.0,
            0.045,
            5.8,
            [0.46, 0.7, 0.86],
        );
    }

    fn draw_stars(&self, frame: &mut Frame, vp: Matrix4<f32>, elapsed: f32) {
        let params = DrawParameters {
            blend: glium::Blend::alpha_blending(),
            depth: Depth {
                test: DepthTest::IfLess,
                write: false,
                ..Default::default()
            },
            ..Default::default()
        };

        for (layer, buffer) in self.star_buffers.iter().enumerate() {
            let speed = [3.5f32, 6.8, 11.5][layer];
            let uniforms = uniform! {
                vp: mat4(vp),
                time: elapsed,
                layer_speed: speed,
                wrap_depth: 120.0f32,
            };
            frame
                .draw(
                    buffer,
                    self.no_indices,
                    &self.star_program,
                    &uniforms,
                    &params,
                )
                .expect("draw stars");
        }
    }

    fn draw_mesh(
        &self,
        frame: &mut Frame,
        mesh: &Mesh,
        vp: Matrix4<f32>,
        transform: Transform,
        alpha: f32,
    ) {
        self.draw_mesh_with_light(
            frame,
            mesh,
            vp,
            transform,
            alpha,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.0,
            0.0,
        );
    }

    fn draw_mesh_with_light(
        &self,
        frame: &mut Frame,
        mesh: &Mesh,
        vp: Matrix4<f32>,
        transform: Transform,
        alpha: f32,
        point_light_pos: [f32; 3],
        point_light_color: [f32; 3],
        point_light_strength: f32,
        time: f32,
    ) {
        let model = transform.matrix();
        let uniforms = uniform! {
            model: mat4(model),
            vp: mat4(vp),
            normal_matrix: mat4(model.invert().unwrap_or_else(Matrix4::identity).transpose()),
            light_dir: [0.35f32, 0.8, 0.5],
            point_light_pos: point_light_pos,
            point_light_color: point_light_color,
            point_light_strength: point_light_strength,
            time: time,
            alpha: alpha,
        };
        let params = DrawParameters {
            blend: glium::Blend::alpha_blending(),
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };
        frame
            .draw(
                &mesh.vertices,
                &mesh.indices,
                &self.solid_program,
                &uniforms,
                &params,
            )
            .expect("draw mesh");
    }

    fn draw_hud(&self, display: &impl Facade, frame: &mut Frame, game: &Game, aspect: f32) {
        let ui = ui_projection(aspect);
        let health_width = game.player.health.max(0) as f32 * 0.42;
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-8.35, 4.34, 2.1, 0.13),
            [0.05, 0.12, 0.16],
            0.88,
        );
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-8.35, 4.34, health_width, 0.13),
            [0.15, 0.95, 0.78],
            1.0,
        );

        self.draw_text(
            display,
            frame,
            ui,
            &format!("SCORE {}", game.score),
            -8.35,
            4.02,
            0.048,
            [0.9, 0.98, 1.0],
        );
        self.draw_text(
            display,
            frame,
            ui,
            &format!("WAVE {}", game.wave()),
            -1.0,
            4.34,
            0.045,
            [0.78, 0.9, 1.0],
        );
        self.draw_text(
            display,
            frame,
            ui,
            &format!("KILLS {}", game.kills),
            -1.0,
            4.02,
            0.04,
            [0.56, 0.72, 0.88],
        );
        if game.combo > 1 {
            self.draw_text(
                display,
                frame,
                ui,
                &format!("COMBO X{}", game.combo),
                5.3,
                4.34,
                0.045,
                [1.0, 0.82, 0.28],
            );
        }

        if game.rapid_fire_timer > 0.0 {
            self.draw_text(
                display,
                frame,
                ui,
                &format!("RAPID {:.0}", game.rapid_fire_timer.ceil()),
                5.3,
                4.02,
                0.04,
                [1.0, 0.9, 0.35],
            );
        }
        if game.shield_timer > 0.0 {
            self.draw_text(
                display,
                frame,
                ui,
                &format!("SHIELD {:.0}", game.shield_timer.ceil()),
                5.3,
                3.72,
                0.04,
                [0.45, 0.76, 1.0],
            );
        }

        if game.game_over {
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(-3.4, -1.45, 6.8, 2.9),
                [0.015, 0.02, 0.04],
                0.95,
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "SHIP DESTROYED",
                0.0,
                0.55,
                0.07,
                5.8,
                [1.0, 0.64, 0.58],
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                &format!("SCORE {}   BEST COMBO {}", game.score, game.best_combo),
                0.0,
                -0.08,
                0.043,
                6.1,
                [0.9, 0.96, 1.0],
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "PRESS R TO RESTART",
                0.0,
                -0.75,
                0.045,
                5.0,
                [0.58, 0.78, 0.92],
            );
        }
    }

    fn draw_snake_hud(
        &self,
        display: &impl Facade,
        frame: &mut Frame,
        snake: &SnakeGame,
        aspect: f32,
    ) {
        let ui = ui_projection(aspect);
        let score_width = (snake.score as f32 * 0.18 + 0.18).min(2.8);
        let score_bar = quad_mesh(-7.9, 4.55, score_width, 0.09, [0.35, 1.0, 0.48]);
        let score_mesh = Mesh::new(display, score_bar.0, score_bar.1);
        self.draw_ui_mesh(frame, &score_mesh, ui, 1.0);

        let score_label = format!("SCORE {}", snake.score);
        self.draw_text(
            display,
            frame,
            ui,
            &score_label,
            -7.9,
            4.18,
            0.055,
            [0.72, 1.0, 0.78],
        );

        if snake.game_over {
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(-20.0, -20.0, 40.0, 40.0),
                [0.0, 0.0, 0.0],
                0.46,
            );
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(-3.4, -1.45, 6.8, 2.9),
                [0.015, 0.028, 0.035],
                0.96,
            );
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(-3.15, -1.2, 6.3, 2.4),
                [0.08, 0.16, 0.15],
                0.88,
            );
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(-3.15, 1.08, 6.3, 0.08),
                [0.42, 1.0, 0.62],
                1.0,
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "GAME OVER",
                0.0,
                0.56,
                0.08,
                4.8,
                [0.9, 1.0, 0.82],
            );
            let final_score = format!("SCORE {}", snake.score);
            self.draw_text_centered(
                display,
                frame,
                ui,
                &final_score,
                0.0,
                -0.12,
                0.055,
                4.2,
                [0.64, 1.0, 0.72],
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "PRESS R TO RESTART",
                0.0,
                -0.72,
                0.045,
                5.4,
                [0.72, 0.9, 0.86],
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "ESC BACK TO MENU",
                0.0,
                -1.05,
                0.04,
                4.8,
                [0.42, 0.7, 0.66],
            );
        }
    }

    fn draw_fps_hud(&self, display: &impl Facade, frame: &mut Frame, fps: &FpsGame, aspect: f32) {
        let ui = ui_projection(aspect);
        self.draw_text(
            display,
            frame,
            ui,
            &format!("HEALTH {}", fps.health.max(0)),
            -8.35,
            4.34,
            0.045,
            [1.0, 0.72, 0.62],
        );
        self.draw_text(
            display,
            frame,
            ui,
            &format!("AMMO {}", fps.ammo),
            -8.35,
            4.0,
            0.045,
            [0.9, 0.86, 0.64],
        );
        self.draw_text(
            display,
            frame,
            ui,
            &format!("SCORE {}", fps.score),
            4.8,
            4.34,
            0.045,
            [0.92, 0.96, 1.0],
        );
        self.draw_text(
            display,
            frame,
            ui,
            &format!("LEVEL {}  ENEMIES {}", fps.level, fps.enemies.len()),
            3.3,
            4.0,
            0.04,
            [0.74, 0.82, 0.94],
        );
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-0.06, -0.01, 0.12, 0.02),
            [1.0, 0.95, 0.75],
            1.0,
        );
        self.draw_ui_rect(
            display,
            frame,
            ui,
            Rect::new(-0.01, -0.06, 0.02, 0.12),
            [1.0, 0.95, 0.75],
            1.0,
        );

        if fps.game_over {
            self.draw_ui_rect(
                display,
                frame,
                ui,
                Rect::new(-3.4, -1.45, 6.8, 2.9),
                [0.02, 0.015, 0.015],
                0.96,
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "MISSION FAILED",
                0.0,
                0.5,
                0.07,
                5.8,
                [1.0, 0.55, 0.45],
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                &format!("SCORE {}   LEVEL {}", fps.score, fps.level),
                0.0,
                -0.12,
                0.045,
                5.8,
                [0.9, 0.96, 1.0],
            );
            self.draw_text_centered(
                display,
                frame,
                ui,
                "PRESS R TO RESTART",
                0.0,
                -0.78,
                0.045,
                5.0,
                [0.7, 0.82, 0.94],
            );
        }
    }

    fn draw_ui_rect(
        &self,
        display: &impl Facade,
        frame: &mut Frame,
        vp: Matrix4<f32>,
        rect: Rect,
        color: [f32; 3],
        alpha: f32,
    ) {
        let (vertices, indices) = quad_mesh(rect.x, rect.y, rect.w, rect.h, color);
        let mesh = Mesh::new(display, vertices, indices);
        self.draw_ui_mesh(frame, &mesh, vp, alpha);
    }

    fn draw_menu_button(
        &self,
        display: &impl Facade,
        frame: &mut Frame,
        vp: Matrix4<f32>,
        button: MenuButton,
        hovered: bool,
        label: &str,
    ) {
        let rect = menu_button(button);
        let fill = if hovered {
            [0.16, 0.54, 0.68]
        } else {
            [0.055, 0.16, 0.25]
        };
        let edge = if hovered {
            [0.5, 0.96, 1.0]
        } else {
            [0.18, 0.45, 0.62]
        };
        self.draw_ui_rect(display, frame, vp, rect, fill, 0.95);
        self.draw_ui_rect(
            display,
            frame,
            vp,
            Rect::new(rect.x, rect.y + rect.h - 0.06, rect.w, 0.06),
            edge,
            1.0,
        );
        self.draw_text_in_rect(
            display,
            frame,
            vp,
            label,
            rect,
            0.064,
            rect.w - 0.38,
            rect.h - 0.18,
            [0.92, 0.98, 1.0],
        );
    }

    fn draw_text(
        &self,
        display: &impl Facade,
        frame: &mut Frame,
        vp: Matrix4<f32>,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: [f32; 3],
    ) {
        let (vertices, indices) = text_mesh(text, x, y, scale, color);
        if vertices.is_empty() {
            return;
        }
        let mesh = Mesh::new(display, vertices, indices);
        self.draw_ui_mesh(frame, &mesh, vp, 1.0);
    }

    fn draw_text_centered(
        &self,
        display: &impl Facade,
        frame: &mut Frame,
        vp: Matrix4<f32>,
        text: &str,
        center_x: f32,
        y: f32,
        scale: f32,
        max_width: f32,
        color: [f32; 3],
    ) {
        let width = text_width(text, scale);
        let fitted_scale = if width > max_width {
            scale * max_width / width
        } else {
            scale
        };
        let fitted_width = text_width(text, fitted_scale);
        self.draw_text(
            display,
            frame,
            vp,
            text,
            center_x - fitted_width * 0.5,
            y,
            fitted_scale,
            color,
        );
    }

    fn draw_text_in_rect(
        &self,
        display: &impl Facade,
        frame: &mut Frame,
        vp: Matrix4<f32>,
        text: &str,
        rect: Rect,
        scale: f32,
        max_width: f32,
        max_height: f32,
        color: [f32; 3],
    ) {
        let width_scale = if text_width(text, scale) > max_width {
            scale * max_width / text_width(text, scale)
        } else {
            scale
        };
        let fitted_scale = if text_height(width_scale) > max_height {
            width_scale * max_height / text_height(width_scale)
        } else {
            width_scale
        };
        let fitted_width = text_width(text, fitted_scale);
        let fitted_height = text_height(fitted_scale);
        self.draw_text(
            display,
            frame,
            vp,
            text,
            rect.x + (rect.w - fitted_width) * 0.5,
            rect.y + (rect.h - fitted_height) * 0.5,
            fitted_scale,
            color,
        );
    }

    fn draw_ui_mesh(&self, frame: &mut Frame, mesh: &Mesh, vp: Matrix4<f32>, alpha: f32) {
        let model = Matrix4::identity();
        let uniforms = uniform! {
            model: mat4(model),
            vp: mat4(vp),
            normal_matrix: mat4(Matrix4::identity()),
            light_dir: [0.0f32, 0.0, 1.0],
            alpha: alpha,
        };
        let params = DrawParameters {
            blend: glium::Blend::alpha_blending(),
            depth: Depth {
                test: DepthTest::Overwrite,
                write: false,
                ..Default::default()
            },
            ..Default::default()
        };
        frame
            .draw(
                &mesh.vertices,
                &mesh.indices,
                &self.solid_program,
                &uniforms,
                &params,
            )
            .expect("draw ui mesh");
    }
}

fn smoothstep(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn tetris_color(id: u8) -> [f32; 3] {
    match id {
        1 => [0.25, 0.92, 0.95],
        2 => [0.98, 0.86, 0.28],
        3 => [0.78, 0.48, 0.95],
        4 => [0.98, 0.64, 0.3],
        5 => [0.35, 0.55, 0.95],
        6 => [0.35, 0.92, 0.48],
        7 => [0.95, 0.35, 0.35],
        _ => [0.18, 0.2, 0.28],
    }
}
