# Vibe Editor Documentation

This document is the in-depth guide for the standalone Vibe Editor application.

It explains:

- How the editor is structured internally.
- What each panel does and how interactions work.
- How project data is modeled and serialized.
- How runtime export is produced.
- How to extend the editor safely.

The implementation described here corresponds to the current code in src/bin/vibe_editor.rs.

## 1) Purpose and Scope

Vibe Editor is a native GUI tool (eframe/egui) for authoring Vibe Engine content through a scene/entity/component workflow.

Primary responsibilities:

- Project authoring (metadata and physics/settings).
- Scene authoring (scene list, clear color, ambient, grid).
- Entity authoring (transform and optional components).
- Asset catalog management.
- Runtime pack export for engine-side consumption.
- Launching sibling binaries from the editor (game runtime and launcher app).

Non-goals in current version:

- No timeline/animation editor.
- No prefab system.
- No built-in script compiler.
- No undo/redo stack.
- No deep in-editor game simulation beyond a UI-level preview toggle.

## 2) High-Level Architecture

### 2.1 Main app state

The editor is modeled as a single stateful app struct:

- EditorApp holds project data, UI state, viewport state, selection state, status/logging, and ID allocation.

Core fields include:

- project: full editable project document.
- project_path: save/open target path.
- selected_scene, selected_entity, selected_asset: active selections.
- hierarchy_filter, asset_filter: list filtering state.
- tool_mode: Select / Move / Rotate / Scale (only Move is currently behaviorally active in viewport).
- viewport_zoom, viewport_pan: camera controls for 2D top projection.
- snap_enabled, snap_size, show_grid: placement aids.
- play_mode: UI toggle for preview state.
- dirty: unsaved-changes flag.
- status_line + console: user-visible diagnostics/event trace.
- next_id: monotonic ID allocator.

### 2.2 Frame/update lifecycle

Each frame, update executes this order:

1. ensure_valid_selection
2. show_top_bar
3. show_bottom_panel
4. show_hierarchy_panel
5. show_inspector_panel
6. show_viewport

This order is important:

- Selection normalization happens before panel reads/writes.
- Panels mutate shared app state directly.
- Viewport interactions can change selection and transform values last in frame.

## 3) Data Model

All project-side data is serde-serializable JSON.

### 3.1 ProjectDocument

Contains:

- name, author, engine_version
- created_unix, modified_unix
- notes
- settings: ProjectSettings
- scenes: Vec<SceneDocument>
- assets: Vec<AssetDocument>

Default project creates one scene and no assets.

### 3.2 ProjectSettings

Current editable settings:

- startup_scene (string)
- gravity (f32)
- fixed_timestep (f32)
- lighting_quality (string)

### 3.3 SceneDocument

Contains:

- id
- name
- ambient_light [r,g,b]
- clear_color [r,g,b]
- grid_size
- entities

Default scene includes a default camera entity.

### 3.4 EntityDocument and components

Entity fields:

- id
- name
- enabled
- tags (comma-separated string in project format)
- transform (always present)
- render (optional)
- collider (optional)
- script (optional)

Components:

- TransformComponent: position/rotation/scale, each as [f32; 3].
- RenderComponent:
  - kind: Mesh | Sprite | Light
  - mesh path
  - material path
  - color [f32; 4]
  - layer i32
- ColliderComponent:
  - shape: Box | Sphere | Capsule
  - size [f32; 3]
  - is_trigger
- ScriptComponent:
  - script_path
  - entry

### 3.5 AssetDocument

Asset catalog items include:

- id
- name
- path
- kind: Texture | Mesh | Material | Audio | Script | Other

Kind can be inferred by extension on import and manually changed in Inspector.

## 4) UI Layout and Panel Behavior

The editor uses a multi-panel desktop layout:

- Top bar: menus + toolbar controls.
- Left panel: Hierarchy.
- Right panel: Inspector.
- Bottom panel: Asset Browser + Console split.
- Center: Viewport.

### 4.1 Top bar

File menu:

- New Project
- Open Project...
- Save
- Save As...
- Export Runtime Pack...
- Quit

Project menu:

- Launch Game Collection (vibe-engine)
- Launch Launcher App (vibe-launcher)
- Create Scene
- Create Entity

Toolbar controls:

- Tool mode toggles.
- Play Preview / Stop Preview.
- Grid toggle.
- Snap toggle and snap step value.
- Quick actions: + Entity, Import Asset.

### 4.2 Hierarchy panel

Capabilities:

- Add/remove scenes.
- Add/duplicate/delete entities.
- Text filter for entity names.
- Scene collapsible groups with entity counts.
- Scene-level "Scene Settings" pseudo-item to switch inspector context.

Selection semantics:

- Selecting a scene settings row clears selected_entity.
- Selecting entity sets both selected_scene and selected_entity.

### 4.3 Inspector panel

Dual-context behavior:

- If entity selected: entity + components inspector.
- If no entity selected: scene + project inspector.

Entity inspector operations:

- Rename, enable toggle, tags edit.
- Transform edit via numeric DragValue fields.
- Per-component enable/disable toggles for Render, Collider, Script.
- Component-specific field editing.

Scene/project inspector operations:

- Scene name, grid size, ambient and clear color edits.
- Project metadata/settings/notes edits.

All edits mark project dirty and append a timestamped log entry.

### 4.4 Bottom panel

Left column: Asset Browser

- Import File...
- Remove Selected
- Add Path (manual path text input)
- Filter list by asset name
- Select asset to inspect/edit on right panel

Right column: Console

- Shows status line and timestamped action log.
- Clear button resets console history.

### 4.5 Viewport

The viewport draws a top-down projected scene:

- Background uses scene clear color.
- Optional grid overlay using scene grid_size.
- Entities drawn as circles with labels.
- Selected entity receives highlight ring.

Interactions:

- Mouse wheel while hovered: zoom in/out (clamped).
- Left click: select nearest entity within hit radius.
- Left drag in Move mode: move selected entity on X/Z plane.
- Double click: create entity at cursor world position.
- Middle mouse drag: pan viewport.

Coordinate transforms:

- world_to_screen and screen_to_world handle mapping.
- snap_value rounds X/Z positions when snap is enabled.

## 5) State Integrity and Safety Rules

### 5.1 Selection safety

ensure_valid_selection enforces invariants each frame:

- At least one scene exists.
- selected_scene index remains in range.
- selected_entity is cleared if entity no longer exists.

This avoids stale references after delete/open/new operations.

### 5.2 Stable IDs

ID behavior:

- allocate_id returns next_id then increments.
- recompute_next_id scans all scene/entity/asset IDs and sets max+1.

Recompute is called after default init, new project, and project open to avoid collisions.

### 5.3 Dirty tracking and logs

mark_dirty:

- Sets dirty true.
- Pushes timestamped log entry via push_log.

Write/save success clears dirty.

## 6) File I/O and Serialization

### 6.1 Project save/open

Project files are JSON and written pretty-formatted.

Open flow:

- Read file to string.
- Deserialize to ProjectDocument.
- Reset selections and dirty state.
- Recompute IDs.

Save flow:

- Save uses current path if known.
- Save As prompts for path (default filename project.vibe.json).
- modified_unix updated before serialization.

Errors are surfaced through status_line and console.

### 6.2 Runtime pack export

Export dialog writes JSON (default filename runtime_pack.json).

Export transformation:

- Project-level output keeps project_name and exported_unix.
- Scenes map to RuntimeScene with ambient/clear and entities.
- Entity tags string is split by comma, trimmed, and empty tags removed.
- Component payloads are passed through as optional structs.

This intentionally flattens project editing metadata into runtime-ready content.

## 7) Runtime Integration and Process Launching

The editor can launch other binaries.

launch_binary uses spawn_binary with multi-path probing:

- Adjacent to current executable.
- target/debug and target/release.
- Windows .exe variants.
- Final fallback: cargo run --bin <name>.

This improves resilience in dev and packaged contexts.

## 8) Asset Import Behavior

Import paths can come from file dialog or typed path input.

infer_asset_kind extension mapping:

- Texture: png/jpg/jpeg/webp/bmp/tga
- Mesh: obj/fbx/gltf/glb
- Audio: wav/ogg/mp3/flac
- Material: mat/material
- Script: rs/lua/js/ts
- Else: Other

Imported asset name defaults to file stem.

## 9) Practical Workflows

### 9.1 Create a new playable scene draft

1. New Project.
2. Create Scene and set clear/ambient color.
3. Add Entity in hierarchy or double click viewport.
4. Select Move tool and place entities using snap.
5. Enable/configure Render and Collider in Inspector.
6. Add Script component where needed.
7. Import referenced assets in Asset Browser.
8. Save project.
9. Export Runtime Pack.

### 9.2 Fast blockout pass

1. Keep snap enabled and set larger step.
2. Use duplicate entity for repeated props.
3. Pan + zoom to place groups quickly.
4. Use filters in hierarchy to focus on subset.

### 9.3 Data sanity pass before export

1. Ensure startup_scene matches an existing scene name.
2. Ensure asset paths and script paths are valid for your runtime layout.
3. Verify tags use comma-separated semantics.
4. Export runtime pack and inspect in web runtime inspector.

## 10) Runtime Pack Compatibility Notes

The web frontend includes a runtime pack inspector that parses exported JSON and reports:

- Scene count
- Entity count
- Script/render/collider component counts
- Scene name list

If parse fails, error text is shown directly in the web UI.

This gives a lightweight validation pass without launching native game runtime.

## 11) Extension Guide for Developers

### 11.1 Add a new entity component type

Recommended change sequence:

1. Add component struct and optional field to EntityDocument.
2. Add inspector section for enable toggle + field editors.
3. Extend RuntimeEntity export struct and export mapping.
4. Ensure serialization derives are present.
5. Confirm open/save/export backward compatibility.

### 11.2 Add a new asset kind

1. Extend AssetKind enum and label().
2. Update inspector ComboBox list.
3. Update infer_asset_kind extension mapping.

### 11.3 Add true behavior for Rotate/Scale tool modes

Current viewport interaction branch only applies Move mode transforms.

To extend:

1. Add drag handling branches for Rotate and Scale.
2. Add visual gizmos/handles in draw_entities or dedicated draw pass.
3. Respect snap settings or add independent rotational snap settings.

### 11.4 Add undo/redo

Current model is direct mutation. For undo/redo:

1. Introduce command/event history stack.
2. Route all mutating operations through command dispatcher.
3. Encode inverse operations for each command.
4. Integrate keyboard shortcuts and menu entries.

## 12) Current Limitations and Design Tradeoffs

- play_mode is a UI flag and log signal, not a full simulation loop.
- No hard referential integrity checks between entity render/script paths and asset entries.
- No per-scene local asset scoping.
- Tags are stored as a single CSV string in project format, transformed to list on export.
- No validation warning panel yet (only status + console messages).

These tradeoffs keep the editor simple and fast to iterate while preserving a useful authoring baseline.

## 13) Troubleshooting

### Problem: Could not save/open project

Check:

- File permissions.
- Parent directory existence.
- JSON validity when opening manually edited files.

The status line and console include exact error strings.

### Problem: Entity appears unselectable in viewport

Check:

- Zoom level and pan offset.
- Whether entity exists in selected scene.
- Selection radius is proximity-based in screen space.

### Problem: Runtime pack parses in editor export but fails in downstream consumer

Check:

- Script/asset path expectations in runtime.
- Enum/string expectations in consumer parser.
- Tag formatting assumptions (CSV in project, list in runtime export).

## 14) Source Map (Implementation Anchors)

Primary file:

- src/bin/vibe_editor.rs

Notable anchors:

- ProjectDocument definition
- SceneDocument definition
- EntityDocument definition
- RuntimeExport definition
- EditorApp definition
- show_top_bar
- show_hierarchy_panel
- show_inspector_panel
- show_bottom_panel
- show_viewport
- draw_scene_and_project_inspector
- draw_entity_inspector
- draw_asset_browser
- draw_console
- draw_selected_asset
- draw_grid
- draw_entities
- select_entity_at_screen
- ensure_valid_selection
- save/open/export/import functions
- spawn_binary helper

Related runtime-pack consumer for cross-checking export compatibility:

- src/bin/vibe_web.rs
- draw_runtime_pack
- parse_runtime_pack

## 15) Recommended Next Improvements

- Add validation panel with structured warnings/errors before export.
- Add undo/redo command stack.
- Add true transform gizmos for rotate/scale.
- Add scene/entity search by tags and component presence.
- Add per-asset metadata (import settings, compression, bundles).
- Add runtime pack schema version field for forward compatibility.
