# Vibe Engine

A compact Rust/OpenGL 3D game engine prototype with a small game collection.

The workspace now also includes:

- `vibe-editor`: a standalone visual editor application for creating game projects, scenes, entities, and runtime export packs.
- `vibe-launcher`: a standalone launcher GUI that can open either the game collection or the editor.
- `vibe-web`: a browser-targeted WASM application (Trunk-based) with web runtime launcher UI and runtime-pack inspection.

## Features

- Cross-platform windowing and OpenGL context creation through `glium`/`glutin`/`winit`
- Engine-style separation for input, timing, rendering, mesh creation, and gameplay state
- Procedurally generated spaceship mesh
- Procedural low-poly asteroid meshes
- Multi-layer parallax starfield
- Mouse-controlled ship movement
- Game selection screen
- Separate menu for each game
- **Space Shooter**: Classic arcade space shooter with scoring, waves, combos, pickups, shield, rapid fire, health, collision, and enemy spawning
- **3D Snake**: Free-movement 3D snake with good food for growth, bad food that shrinks you and reduces score
- **FPS Arena**: First-person shooter with procedural indoor/outdoor maps, enemies with steering AI, projectiles, levels, ammo, health, and score; fresh random map layouts each run; dynamic point-light window-sourced indoor illumination; visible enemy meshes with OBB collision detection; first-person gun viewmodel with muzzle flash feedback
- **3D Platformer (NEW)**: Third-person platformer with FPS-style movement, space-to-jump controls, gravity-based jumping, procedurally generated jumping sequences, a checkered goal platform that completes the level, and a fresh random layout every level
- **Tetris**: Legacy-adapted Tetris implementation with next/hold pieces, ghost toggle, keyboard controls, and mouse-assisted column aiming and actions

## Binaries

- `vibe-engine`: game collection runtime
- `vibe-editor`: full editor GUI
- `vibe-launcher`: unified launcher GUI
- `vibe-web`: web/WASM frontend

## Run

```powershell
# default run target (game collection)
cargo run

# explicit runtime
cargo run --bin vibe-engine

# editor app
cargo run --bin vibe-editor

# launcher app
cargo run --bin vibe-launcher

# web app (native desktop preview)
cargo run --bin vibe-web
```

## Web (WASM) Build

Prerequisites:

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Run locally:

```powershell
trunk serve --config Trunk.toml
```

Build static deploy output:

```powershell
trunk build --release --config Trunk.toml
```

The web build output is written to `dist-web/` and can be deployed to any static hosting provider.

## Documentation

Full documentation is available at **[Elia1995-Official.github.io/Vibe-Engine](https://Elia1995-Official.github.io/Vibe-Engine/)** — built with mdBook and deployed via GitHub Pages.

Documentation includes:
- [Editor Documentation](https://Elia1995-Official.github.io/Vibe-Engine/editor.html) — full guide for the Vibe Editor application
- [Manual Coding Guide](https://Elia1995-Official.github.io/Vibe-Engine/coding-guide.html) — file-by-file walkthrough of the engine source
- [Guida di Codifica Manuale](https://Elia1995-Official.github.io/Vibe-Engine/coding-guide-it.html) — versione italiana

Documentation rule:
- After each code change, update the relevant documentation files in the same work pass so behavior, controls, and architecture notes stay aligned with the code.

Controls:

- Game selection: click a game, press Enter for Space Shooter, press S for 3D Snake, press F for FPS Arena, or press T for Tetris
- Each game menu: click Start, press Enter, or press Space
- F1: show or hide help dialog
- Space Shooter: move mouse to steer, left mouse or Space to fire
- 3D Snake: Arrow keys or WASD to turn; hold the current direction to boost speed
- FPS Arena: WASD to move, mouse to look, left click to shoot
- 3D Platformer: WASD to move, Space to jump, hold RMB and move mouse to rotate camera
- Tetris keyboard: A/D or Left/Right to move, W/Up to rotate, S/Down soft drop, Space hard drop, C hold, G ghost toggle
- Tetris mouse: move cursor over board to aim column, left click rotate, right click hard drop, middle click hold
- R: restart after game over
- Escape: return to the current game's menu, then to game selection, then quit
