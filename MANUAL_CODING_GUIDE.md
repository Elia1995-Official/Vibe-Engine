# Manual Coding Guide

This guide explains how to edit the Vibe Engine files by hand without getting lost in the project structure. The project is intentionally small, but it is split into modules so gameplay, rendering, UI, shaders, and procedural generation can evolve independently.

## First Rules

- Run `cargo fmt` after editing Rust files.
- Run `cargo check` for fast compile validation.
- Run `cargo run` when you want to test the game collection runtime.
- Run `cargo run --bin vibe-editor` for the standalone editor GUI.
- Run `cargo run --bin vibe-launcher` for the standalone launcher GUI.
- Run `cargo run --bin vibe-web` for native preview of the web/WASM frontend.
- Run `trunk serve --config Trunk.toml` to test the actual browser WASM build.
- After every completed code task, update the relevant documentation (at minimum `README.md` and this guide when applicable).
- Prefer changing one system at a time: gameplay in `game.rs`, visuals in `renderer.rs`, generated meshes in `procedural.rs`, menu layout in `ui.rs`.
- Keep keyboard bindings in `src/keybindings.rs` so `main.rs` stays focused on event routing.
- If you add a new `.rs` file in `src`, declare it near the top of `src/main.rs` with `mod your_file_name;`.

## File Map

### `src/main.rs`

This is the application bootstrap and event loop.

Edit this file when you want to:

- Add a new global screen or mode, such as `Pause`, `Settings`, or `GameOverMenu`.
- Change keyboard or mouse handling.
- Change cursor behavior when entering or leaving gameplay.
- Change the window title or starting resolution.
- Decide which renderer function is called for each app state.

Important sections:

- `mod app; mod config; ...`: module declarations.
- `enter_game(...)`: transition from menu to gameplay.
- `enter_menu(...)`: transition from gameplay back to menu.
- `WindowEvent::KeyboardInput`: keyboard bindings.
- `WindowEvent::MouseInput`: mouse click handling.
- `WindowEvent::RedrawRequested`: update and draw loop.

Keyboard bindings are now centralized in `src/keybindings.rs`. `main.rs` should route keyboard events there instead of growing a second copy of the input switch.

### `src/keybindings.rs`

This file owns all keyboard shortcuts and game/menu transitions.

Edit this file when you want to:

- Change global keyboard shortcuts like Escape, Enter, Space, F1, or R.
- Change how a game starts from its menu.
- Change how keyboard input maps to gameplay flags in each mode.
- Add or remove key-driven menu navigation.

Important responsibilities:

- Handles `SnakeDirection` mapping from arrow keys and WASD.
- Routes Space Shooter, Snake, FPS, and Platformer menu starts.
- Handles Escape-to-menu and Escape-to-quit behavior.
- Updates `Input` flags for FPS and Platformer movement.

Keep mouse handling in `main.rs`; keep keyboard binding logic here.

### `src/bin/vibe_editor.rs`

This file is a full standalone editor application (separate process from the game runtime).

Edit this file when you want to:

- Change editor layout and panels (Hierarchy, Inspector, Viewport, Asset Browser, Console).
- Change project save/load/export behavior.
- Change editor tools (selection, move, rotate, scale).
- Change how entities and assets are created and edited.
- Change launcher hooks used by the editor to run `vibe-engine` or `vibe-launcher`.

### `src/bin/vibe_launcher.rs`

This file is a standalone launcher GUI.

Edit this file when you want to:

- Change launcher UI and startup flow.
- Add new quick actions (build, open repo folder, open docs).
- Change process-launch behavior for runtime/editor binaries.

### `src/bin/vibe_web.rs`

This file is the browser-targeted WASM frontend.

Edit this file when you want to:

- Change web UI layout for Home/Runtime Pack/Build tabs.
- Change runtime pack validation logic in the browser.
- Change web-specific startup behavior (`eframe::WebRunner`) and canvas bootstrapping.

Web build infrastructure:

- `web/index.html`: Trunk entry page and canvas host element.
- `Trunk.toml`: Trunk build/serve configuration.

Example: change the window title:

```rust
let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
    .with_title("My Space Game")
    .with_inner_size(1280, 720)
    .build(&event_loop);
```

Example: add a new key binding:

```rust
PhysicalKey::Code(KeyCode::KeyP) if pressed => {
    // Toggle pause here after adding a Pause app state.
}
```

### `src/app.rs`

This file stores app-level state.

Edit this file when you want to:

- Add a new app screen to `AppState`.
- Add new input flags to `Input`.
- Change the default mouse position or initial input state.

Current states:

- `GameSelect`: main menu showing all game choices.
- `SpaceMenu`, `SpacePlaying`: Space Shooter menu and gameplay.
- `SnakeMenu`, `SnakePlaying`: 3D Snake menu and gameplay.
- `FpsMenu`, `FpsPlaying`: FPS Arena menu and gameplay.
- `PlatformerMenu`, `PlatformerPlaying`: 3D Platformer menu and gameplay.

Example: add a pause state:

```rust
pub enum AppState {
    GameSelect,
    SpaceMenu,
    SpacePlaying,
    Paused,
}
```

After adding a state, update `main.rs` so the event loop knows how to render and transition to it.

### `src/config.rs`

This file contains shared gameplay and rendering constants.

Edit this file when you want quick balance changes:

- `WORLD_HALF_WIDTH`: horizontal playable area.
- `WORLD_HALF_HEIGHT`: vertical playable area.
- `PLAYER_Z`: player depth.
- `BULLET_SPEED`: shot speed.
- `ASTEROID_SPEED`: base asteroid speed.
- `STAR_LAYERS`: number of parallax star layers.
- `STAR_COUNT_PER_LAYER`: stars per layer.

Example: make the game faster:

```rust
pub const BULLET_SPEED: f32 = 34.0;
pub const ASTEROID_SPEED: f32 = 11.0;
```

Changing constants is usually the safest first edit because it rarely requires touching other files.

### `src/game.rs`

This file owns the shooter gameplay simulation.

Edit this file when you want to:

- Change player health.
- Change shooting cooldown.
- Change enemy spawning.
- Add new enemy behavior.
- Change collision rules.
- Change scoring.
- Add powerups or pickups.

Important structs:

- `Game`: the full simulation state.
- `Player`: ship position, target, cooldown, health.
- `Bullet`: bullet position.
- `Asteroid`: enemy position, velocity, radius, rotation, mesh selection.

Important functions:

- `Game::new()`: initial game state.
- `Game::update(...)`: all per-frame gameplay simulation.

Example: change starting health:

```rust
health: 8,
```

Example: change firing rate:

```rust
self.player.cooldown = 0.08;
```

Lower cooldown means faster shooting.

Example: make asteroids spawn less often:

```rust
self.spawn_timer = self.rng.gen_range(0.45..1.05) / difficulty.min(3.5);
```

Example: change score per asteroid:

```rust
self.score += 25;
```

When editing `Game::update`, keep this rough order:

1. Handle game-over restart.
2. Update player target and movement.
3. Spawn bullets.
4. Move bullets.
5. Spawn asteroids.
6. Move asteroids.
7. Resolve bullet collisions.
8. Resolve player collisions.
9. Remove destroyed or off-screen entities.

That order keeps the simulation predictable.

### `src/fps_game.rs`

This file owns the FPS Arena gameplay simulation.

Edit this file when you want to:

- Change indoor or outdoor map generation.
- Tune FPS movement, aiming, enemy chase speed, and damage.
- Adjust map progression and level scaling.
- Tune wall collision behavior for player or enemies.
- Change line-of-sight and hit registration.

Important fields and concepts:

- `map`: collision map (`true` means solid wall).
- `map_type`: alternates between `Indoor` and `Outdoor` by level.
- `indoor_doors`: passable doorway cells carved from indoor walls.
- `indoor_windows`: solid wall cells rendered with glass windows.
- Enemy steering state: velocity, wander angle, strafe sign, and heading for natural motion.
- `rng`: seeded randomly at game startup so procedural FPS maps differ each run.

Important functions:

- `generate_level()`: creates indoor mazes or outdoor arenas.
- `try_move(...)`: player movement with wall collision.
- `update_enemies(...)`: steering AI blend (pursuit, orbit/strafe, wander, separation) with radius-aware substep collision.
- `has_line_of_sight(...)`: ray-march check used by shooting.

Collision note:

- FPS shooting and enemy-player contact use oriented-box tests against enemy heading so collision follows the rendered enemy mesh footprint rather than center-point distance only.
- FPS bullets are now simulated as short-lived projectiles (with wall/enemy collision) instead of instant-hit traces.

If enemies appear to phase through corners, tune substep size and enemy radius in `update_enemies(...)`.

### `src/platformer_game.rs`

This file owns the 3D Platformer gameplay simulation.

Edit this file when you want to:

- Change platform generation, spacing, or progression.
- Tune jump mechanics (power, gravity, landing sensitivity).
- Adjust player movement speed and camera look responsiveness.
- Change camera distance/height for third-person view.
- Modify level progression and difficulty scaling.

Important fields and concepts:

- `player_x, player_y, player_z`: player position in 3D world space.
- `vel_y`: vertical velocity used for jump and gravity.
- `camera_yaw`: third-person camera yaw driven by mouse motion.
- `on_ground`: whether player is currently standing on a platform.
- `platforms`: vector of procedurally generated floating platforms.
- `level`: current level number (increments when reaching the goal).
- `score`: total score (increases by 100 per level completed).
- `game_over`: set to true if player falls below -8.0 on Y axis.

Important functions:

- `generate_level()`: creates platform sequence with increasing difficulty per level.
- `update()`: handles input, gravity, movement, jumping, collision, and goal detection.
- `try_move_horizontal()`: applies direct FPS-style horizontal movement with collision checks.
- `try_move_vertical()`: applies vertical motion, landing, and ceiling collision checks.
- `complete_level()`: increments score and level, then regenerates a fresh random platform layout.
- `player_collides_platform()`: sphere-to-box collision detection for landing and wall hits.

Platformer tuning:

- `GRAVITY: f32 = 18.5`: how fast the player falls (higher = faster).
- `JUMP_POWER: f32 = 8.2`: initial upward velocity when jumping.
- `MOVE_SPEED: f32 = 4.8`: horizontal movement speed.
- `LOOK_SPEED: f32 = 0.0022`: mouse-to-camera rotation sensitivity.
- `PLAYER_RADIUS: f32 = 0.28`: collision sphere radius.

Goal note:

- The last platform is drawn as a checkered finish pad.
- Touching and standing on that goal platform triggers level completion and generates the next random level.

Camera note:

- Platformer camera rotation now follows mouse movement directly, like the FPS controller.
- The player remains third-person visible while the camera orbits behind them.

Movement note:

- Platformer movement now follows the FPS controller model: WASD moves relative to camera direction, Space jumps, and gravity handles vertical motion.
- The horizontal move path is direct and reactive, not physics-spring-based.

To make jumping easier, increase `JUMP_POWER`. To make it harder, decrease it or increase `GRAVITY`.

### `src/graphics.rs`

This file contains shared rendering data types.

Edit this file when you want to:

- Add new vertex attributes.
- Change what every mesh vertex stores.
- Add transform behavior.
- Change mesh construction rules.

Important structs:

- `Vertex`: position, normal, color.
- `StarVertex`: position, color, size.
- `Mesh`: GPU vertex and index buffers.
- `Transform`: position, rotation, scale.

Be careful when editing `Vertex` or `StarVertex`. If you add fields, you must also:

- Update `implement_vertex!(...)`.
- Update all code that creates vertices.
- Update GLSL shader inputs in `src/shaders.rs`.

Example: adding a UV coordinate would require changes in `graphics.rs`, `procedural.rs`, `ui.rs`, and `shaders.rs`.

### `src/procedural.rs`

This file generates meshes and star data.

Edit this file when you want to:

- Change the player ship shape.
- Change asteroid roughness or polygon density.
- Change bullet mesh shape.
- Change star spread, brightness, size, or depth.
- Add new generated mesh helpers.

Important functions:

- `procedural_ship(...)`: creates the player ship vertices.
- `ship_indices()`: triangle list for the ship mesh.
- `bullet_mesh()`: bullet vertices.
- `procedural_asteroid(...)`: generated asteroid vertices and indices.
- `generate_stars(...)`: starfield points.
- `quad_mesh(...)`: simple rectangle mesh used by UI and HUD.

Example: change ship accent color range:

```rust
let accent = [
    rng.gen_range(0.8..1.0),
    rng.gen_range(0.2..0.4),
    rng.gen_range(0.2..0.4),
];
```

Example: make asteroids rougher:

```rust
let rough = rng.gen_range(0.55..1.45);
```

Example: make stars larger:

```rust
size: rng.gen_range(2.5..6.0) + layer as f32,
```

Important: mesh indices reference positions in the vertex array. If you add or remove ship vertices in `procedural_ship`, update `ship_indices()` so every triangle uses valid vertex numbers.

### `src/renderer.rs`

This file draws everything with OpenGL through `glium`.

Edit this file when you want to:

- Change camera position or field of view.
- Change object draw order.
- Change lighting direction.
- Change FPS indoor architecture layers (walls, ceilings, doors, windows).
- Change FPS sky-window lighting behavior (sky color cycle, nearest-window light source, and intensity).
- Change HUD visuals.
- Change main menu layout colors.
- Add rendering for new entity types.
- Add new shader programs.

Important functions:

- `Renderer::new(...)`: loads shaders and creates GPU meshes.
- `Renderer::render(...)`: draws gameplay.
- `Renderer::render_menu(...)`: draws the main menu.
- `draw_stars(...)`: draws parallax star points.
- `draw_mesh(...)`: draws 3D mesh objects.
- `draw_hud(...)`: draws health/score UI.
- `draw_ui_rect(...)`, `draw_menu_button(...)`, `draw_text(...)`: menu and UI helpers.

FPS rendering note:

- `render_fps(...)` now also draws a placeholder first-person gun viewmodel (body + barrel cubes) in camera space.
- Indoor dynamic light is derived from skybox color and projected from the nearest window toward the player.
- Viewmodel is rendered in camera space with fixed orientation so it does not spin/rotate when turning or moving.
- A muzzle-flash mesh is blended using `shot_feedback` for immediate visual fire feedback.

Platformer rendering note:

- `render_platformer(...)` uses third-person camera positioned behind and above the player.
- Camera follows player with fixed offset distance and height.
- Platforms, floor, and player all rendered from this third-person perspective.
- HUD displays current level, score, and game-over screen if player falls.
- Camera rotation responds to mouse X movement for orbital look control.

Example: change gameplay camera:

```rust
let view = Matrix4::look_at_rh(
    Point3::new(0.0, -0.15, 12.0),
    Point3::new(0.0, -0.35, -15.0),
    Vector3::unit_y(),
);
```

Higher camera `z` means the camera sits farther back.

Example: change field of view:

```rust
let projection = perspective(Deg(65.0), aspect, 0.1, 160.0);
```

Example: change the menu title:

```rust
self.draw_text(
    display,
    frame,
    ui,
    "MY GAME",
    -3.85,
    2.25,
    0.22,
    [0.45, 0.95, 1.0],
);
```

If you add a new gameplay entity in `game.rs`, draw it in `Renderer::render(...)` after choosing or creating a mesh for it.

### `src/ui.rs`

This file contains menu geometry and bitmap text helpers.

Edit this file when you want to:

- Move menu buttons.
- Change button hitboxes.
- Change UI coordinate conversion.
- Change bitmap text generation.
- Add supported characters to the built-in text renderer.

Important items:

- `MenuButton`: known menu button IDs (SpaceShooter, Snake, Fps, Platformer, Start, Quit).
- `Rect`: UI rectangle and hit testing.
- `ui_projection(...)`: orthographic UI camera.
- `mouse_to_ui(...)`: converts mouse normalized-device coordinates into UI coordinates.
- `menu_button(...)`: button rectangles.
- `text_width(...)`: simple text measurement.
- `text_mesh(...)`: converts text into quad meshes.
- `glyph(...)`: 5x7 bitmap glyph patterns.

Example: move the Start button upward:

```rust
MenuButton::Start => Rect::new(-1.95, -0.25, 3.9, 0.72),
```

Example: add a new menu button:

```rust
pub enum MenuButton {
    Start,
    Settings,
    Quit,
}
```

Then update `menu_button(...)`, `renderer.rs`, and `main.rs` click handling.

Example: add punctuation support:

```rust
'!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
```

### `src/shaders.rs`

This file contains GLSL shader source strings.

Edit this file when you want to:

- Change lighting.
- Change star rendering.
- Add new shader inputs or uniforms.
- Change fragment colors or alpha behavior.

Current shaders:

- `SOLID_VERTEX_SHADER`: transforms mesh vertices and normals.
- `SOLID_FRAGMENT_SHADER`: lights colored mesh surfaces.
- `STAR_VERTEX_SHADER`: animates star positions and point sizes.
- `STAR_FRAGMENT_SHADER`: gives stars soft circular alpha.

Example: make mesh lighting brighter:

```glsl
vec3 color = v_color * (0.38 + diffuse * 1.1) + rim;
```

Example: make stars sharper:

```glsl
float alpha = smoothstep(0.18, 0.01, dist);
```

If you add a shader uniform, also add it in the corresponding `uniform! { ... }` block in `renderer.rs`.

## Common Editing Recipes

### Make the player faster or smoother

Open `src/game.rs` and find:

```rust
self.player.position +=
    (self.player.target - self.player.position) * (1.0 - (-14.0 * dt).exp());
```

Increase `14.0` for snappier movement. Decrease it for heavier, smoother movement.

### Make the playable area wider

Open `src/config.rs`:

```rust
pub const WORLD_HALF_WIDTH: f32 = 10.5;
```

Then run `cargo run` and test if the ship and asteroids still feel balanced.

### Add a new enemy type

Recommended steps:

1. Add a new struct or enum field in `src/game.rs`.
2. Spawn it inside `Game::update`.
3. Update its movement in `Game::update`.
4. Add collision handling.
5. Add a procedural mesh in `src/procedural.rs`.
6. Create the GPU mesh in `Renderer::new`.
7. Draw it in `Renderer::render`.

Keep the first version simple: position, velocity, radius, mesh ID.

### Add a pause menu

Recommended steps:

1. Add `Paused` to `AppState` in `src/app.rs`.
2. In `src/main.rs`, map `P` or `Escape` to enter/leave pause.
3. In `WindowEvent::RedrawRequested`, do not call `game.update(...)` while paused.
4. Add `Renderer::render_pause(...)` in `src/renderer.rs`, or reuse `render_menu(...)` style helpers.

### Add a settings menu button

Recommended steps:

1. Add `Settings` to `MenuButton` in `src/ui.rs`.
2. Add a rectangle for it in `menu_button(...)`.
3. Draw it in `Renderer::render_menu(...)`.
4. Add click handling in `src/main.rs`.
5. Add `Settings` to `AppState` if it needs its own screen.

### Change menu text

Open `src/renderer.rs` and edit the text passed to `draw_text(...)`.

The current bitmap font supports uppercase letters and digits. Lowercase text is converted to uppercase. Unknown punctuation becomes a question mark style glyph unless you add it to `glyph(...)` in `src/ui.rs`.

### Change the starfield

Open `src/procedural.rs`:

- Star count is controlled in `src/config.rs`.
- Star spread and depth are in `generate_stars(...)`.
- Star animation speed is in `draw_stars(...)` inside `src/renderer.rs`.

Open `src/shaders.rs` if you want to change point shape or alpha.

## Adding New Files

To add a new module:

1. Create `src/audio.rs`, `src/settings.rs`, or another focused file.
2. Add this to the top of `src/main.rs`:

```rust
mod audio;
```

3. Import items where needed:

```rust
use audio::AudioSystem;
```

4. Mark shared items as `pub` inside the new file.

Rule of thumb: if another file needs to use a type or function, it must be `pub`.

## Rust Visibility Cheat Sheet

- `fn helper()` is private to the current module.
- `pub fn helper()` can be used by other modules.
- `struct Thing { field: i32 }` has private fields.
- `pub struct Thing { pub field: i32 }` can be constructed and read from other modules.
- `pub enum Mode { A, B }` can be matched from other modules.

Keep most helper functions private. Make only the actual module API public.

## Compile And Debug Workflow

Use this loop while editing:

```powershell
cargo fmt
cargo check
cargo run
```

Use `cargo check` often because it is faster than a full build.

For runtime panics with more detail:

```powershell
$env:RUST_BACKTRACE=1
cargo run
```

To clear the backtrace setting in the same terminal:

```powershell
Remove-Item Env:RUST_BACKTRACE
```

## Common Compile Errors

### `cannot find function/type in this scope`

You probably moved or added something but did not import it.

Fix by adding a `use` line:

```rust
use crate::ui::menu_button;
```

Inside `main.rs`, modules are imported without `crate::`:

```rust
use ui::menu_button;
```

### `field is private`

The struct field is not marked `pub`.

Example fix:

```rust
pub struct Player {
    pub health: i32,
}
```

Only make fields public when another module truly needs them.

### `UniformTypeMismatch`

The Rust uniform type does not match the GLSL shader type.

Examples:

- GLSL `float` should receive Rust `f32`.
- GLSL `vec3` should receive `[f32; 3]`.
- GLSL `mat4` should receive `[[f32; 4]; 4]`.

Good:

```rust
layer_speed: 3.5f32,
light_dir: [0.35f32, 0.8, 0.5],
vp: mat4(vp),
```

### Blank or invisible mesh

Check these points:

- Are indices valid for the vertex array?
- Is the object behind the camera?
- Is the object too small or too large?
- Is the color too dark?
- Is depth testing hiding it?
- Is the shader input name the same as the Rust vertex field?

### Text does not show a character

Add the character to `glyph(...)` in `src/ui.rs`, or use letters/digits that already exist.

## Style Guidelines For This Project

- Keep simulation logic in `game.rs`.
- Keep OpenGL drawing logic in `renderer.rs`.
- Keep generated geometry in `procedural.rs`.
- Keep hard-coded balancing values in `config.rs` when they are shared or likely to be tweaked.
- Keep shader code in `shaders.rs`.
- Keep UI hitboxes and text mesh logic in `ui.rs`.
- Avoid putting gameplay decisions in shaders.
- Avoid putting rendering buffer creation in `game.rs`.
- Keep `main.rs` focused on window events and app-state transitions.

This split keeps manual edits small and easier to test.
