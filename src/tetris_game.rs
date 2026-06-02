use glium::winit::keyboard::KeyCode;

use crate::app::{Input, SoundEvent};

pub const TETRIS_BOARD_W: usize = 10;
pub const TETRIS_BOARD_H: usize = 20;
pub const TETRIS_CELL_SIZE: f32 = 0.32;
pub const TETRIS_BOARD_X: f32 = -2.7;
pub const TETRIS_BOARD_TOP: f32 = 3.55;

const STEP_TIME: f32 = 0.35;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    I,
    O,
    T,
    L,
    J,
    S,
    Z,
}

#[derive(Default, Clone, Copy)]
pub struct TetrisEvents {
    pub locked: bool,
    pub cleared: bool,
}

#[derive(Default)]
struct ControlState {
    left_pressed: bool,
    right_pressed: bool,
    down_held: bool,
    rotate_pressed: bool,
    drop_pressed: bool,
    hold_pressed: bool,
    ghost_toggle_pressed: bool,
}

#[derive(Clone, Copy)]
struct Piece {
    kind: PieceKind,
    blocks: [(i32, i32); 4],
    pos: (i32, i32),
}

pub struct TetrisGame {
    grid: Vec<u8>,
    piece: Piece,
    next: PieceKind,
    hold: Option<PieceKind>,
    hold_used: bool,
    timer: f32,
    step_time: f32,
    rng_state: u32,
    pub score: u32,
    pub level: u32,
    pub lines: u32,
    pub game_over: bool,
    events: TetrisEvents,
    controls: ControlState,
    show_ghost: bool,
}

impl TetrisGame {
    pub fn new() -> Self {
        let mut state = Self {
            grid: vec![0; TETRIS_BOARD_W * TETRIS_BOARD_H],
            piece: Piece::new(PieceKind::I),
            next: PieceKind::O,
            hold: None,
            hold_used: false,
            timer: STEP_TIME,
            step_time: STEP_TIME,
            rng_state: 0x1111_7788,
            score: 0,
            level: 1,
            lines: 0,
            game_over: false,
            events: TetrisEvents::default(),
            controls: ControlState::default(),
            show_ghost: true,
        };
        state.reset();
        state
    }

    pub fn reset(&mut self) {
        self.grid.fill(0);
        self.next = self.random_kind();
        self.piece = Piece::new(self.next);
        self.next = self.random_kind();
        self.hold = None;
        self.hold_used = false;
        self.timer = STEP_TIME;
        self.step_time = STEP_TIME;
        self.score = 0;
        self.level = 1;
        self.lines = 0;
        self.game_over = false;
        self.events = TetrisEvents::default();
        self.controls = ControlState::default();
        self.show_ghost = true;
    }

    pub fn handle_key(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyA | KeyCode::ArrowLeft if pressed => {
                self.controls.left_pressed = true;
            }
            KeyCode::KeyD | KeyCode::ArrowRight if pressed => {
                self.controls.right_pressed = true;
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.controls.down_held = pressed;
            }
            KeyCode::KeyW | KeyCode::ArrowUp if pressed => {
                self.controls.rotate_pressed = true;
            }
            KeyCode::Space if pressed => {
                self.controls.drop_pressed = true;
            }
            KeyCode::KeyC if pressed => {
                self.controls.hold_pressed = true;
            }
            KeyCode::KeyG if pressed => {
                self.controls.ghost_toggle_pressed = true;
            }
            _ => {}
        }
    }

    pub fn handle_mouse_move(&mut self, mouse_ui_x: f32, mouse_ui_y: f32) {
        if self.game_over || !is_inside_board(mouse_ui_x, mouse_ui_y) {
            return;
        }

        let target_col = ((mouse_ui_x - TETRIS_BOARD_X) / TETRIS_CELL_SIZE).floor() as i32;
        self.move_center_to_column(target_col.clamp(0, TETRIS_BOARD_W as i32 - 1));
    }

    pub fn mouse_rotate(&mut self) {
        self.controls.rotate_pressed = true;
    }

    pub fn mouse_hard_drop(&mut self) {
        self.controls.drop_pressed = true;
    }

    pub fn mouse_hold(&mut self) {
        self.controls.hold_pressed = true;
    }

    pub fn update(&mut self, input: &mut Input, dt: f32) {
        if input.restart {
            self.reset();
            input.restart = false;
            return;
        }

        if self.game_over {
            input.restart = false;
            self.clear_pressed_controls();
            return;
        }

        self.events = TetrisEvents::default();

        if self.controls.ghost_toggle_pressed {
            self.show_ghost = !self.show_ghost;
        }

        if self.controls.hold_pressed && !self.hold_used {
            let current = self.piece.kind;
            match self.hold {
                None => {
                    self.hold = Some(current);
                    self.piece = Piece::new(self.next);
                    self.next = self.random_kind();
                }
                Some(kind) => {
                    self.hold = Some(current);
                    self.piece = Piece::new(kind);
                }
            }
            self.hold_used = true;
        }

        if self.controls.left_pressed {
            self.try_move((-1, 0));
        }
        if self.controls.right_pressed {
            self.try_move((1, 0));
        }
        if self.controls.rotate_pressed {
            self.try_rotate();
        }

        if self.controls.drop_pressed {
            let (end_pos, hit_other) = self.compute_drop();
            self.lock_piece_at(end_pos, self.piece.blocks, hit_other, input);
            self.timer = self.step_time;
            self.flush_events_to_audio(input);
            self.clear_pressed_controls();
            input.restart = false;
            return;
        }

        self.timer -= dt;
        if self.controls.down_held {
            self.timer -= dt * 1.5;
        }

        if self.timer <= 0.0 {
            self.timer += self.step_time;
            if !self.try_move((0, -1)) {
                self.lock_piece(input);
            }
        }

        self.flush_events_to_audio(input);
        self.clear_pressed_controls();
        input.restart = false;
    }

    pub fn board_cell(&self, x: i32, y: i32) -> u8 {
        if x < 0 || x >= TETRIS_BOARD_W as i32 || y < 0 || y >= TETRIS_BOARD_H as i32 {
            return 0;
        }
        self.grid[(y as usize) * TETRIS_BOARD_W + x as usize]
    }

    pub fn active_cells(&self) -> [(i32, i32); 4] {
        offset_cells(self.piece.blocks, self.piece.pos)
    }

    pub fn ghost_cells(&self) -> Option<[(i32, i32); 4]> {
        if !self.show_ghost {
            return None;
        }
        let (end_pos, _) = self.compute_drop();
        Some(offset_cells(self.piece.blocks, end_pos))
    }

    pub fn active_color(&self) -> u8 {
        piece_color(self.piece.kind)
    }

    pub fn next_cells(&self) -> [(i32, i32); 4] {
        Piece::new(self.next).blocks
    }

    pub fn next_color(&self) -> u8 {
        piece_color(self.next)
    }

    pub fn hold_cells(&self) -> Option<[(i32, i32); 4]> {
        self.hold.map(|kind| Piece::new(kind).blocks)
    }

    pub fn hold_color(&self) -> Option<u8> {
        self.hold.map(piece_color)
    }

    pub fn ghost_enabled(&self) -> bool {
        self.show_ghost
    }

    fn flush_events_to_audio(&mut self, input: &mut Input) {
        if self.events.locked {
            input.sound_events.push(SoundEvent::Land);
        }
        if self.events.cleared {
            input.sound_events.push(SoundEvent::PickupCollected);
        }
        self.events = TetrisEvents::default();
    }

    fn clear_pressed_controls(&mut self) {
        self.controls.left_pressed = false;
        self.controls.right_pressed = false;
        self.controls.rotate_pressed = false;
        self.controls.drop_pressed = false;
        self.controls.hold_pressed = false;
        self.controls.ghost_toggle_pressed = false;
    }

    fn try_move(&mut self, delta: (i32, i32)) -> bool {
        let next = (self.piece.pos.0 + delta.0, self.piece.pos.1 + delta.1);
        if self.fits(next, self.piece.blocks) {
            self.piece.pos = next;
            true
        } else {
            false
        }
    }

    fn move_center_to_column(&mut self, target_col: i32) {
        let mut delta = target_col - self.piece_center_column();
        while delta > 0 {
            if !self.try_move((1, 0)) {
                break;
            }
            delta -= 1;
        }
        while delta < 0 {
            if !self.try_move((-1, 0)) {
                break;
            }
            delta += 1;
        }
    }

    fn piece_center_column(&self) -> i32 {
        let cells = self.active_cells();
        let sum = cells.iter().map(|(x, _)| *x as f32).sum::<f32>();
        (sum / 4.0).round() as i32
    }

    fn try_rotate(&mut self) {
        let mut rotated = self.piece.blocks;
        for block in &mut rotated {
            *block = (-block.1, block.0);
        }
        if self.fits(self.piece.pos, rotated) {
            self.piece.blocks = rotated;
        }
    }

    fn fits(&self, pos: (i32, i32), blocks: [(i32, i32); 4]) -> bool {
        for block in &blocks {
            let cell_x = pos.0 + block.0;
            let cell_y = pos.1 + block.1;
            if cell_x < 0
                || cell_y < 0
                || cell_x >= TETRIS_BOARD_W as i32
                || cell_y >= TETRIS_BOARD_H as i32
            {
                return false;
            }
            if self.board_cell(cell_x, cell_y) != 0 {
                return false;
            }
        }
        true
    }

    fn lock_piece(&mut self, input: &mut Input) {
        self.lock_piece_at(self.piece.pos, self.piece.blocks, false, input);
    }

    fn lock_piece_at(
        &mut self,
        pos: (i32, i32),
        blocks: [(i32, i32); 4],
        _hit_other: bool,
        input: &mut Input,
    ) {
        let mut touched_top = false;

        for block in &blocks {
            let cell_x = pos.0 + block.0;
            let cell_y = pos.1 + block.1;
            if cell_y >= TETRIS_BOARD_H as i32 - 1 {
                touched_top = true;
            }
            if cell_x >= 0
                && cell_y >= 0
                && cell_x < TETRIS_BOARD_W as i32
                && cell_y < TETRIS_BOARD_H as i32
            {
                let idx = (cell_y as usize) * TETRIS_BOARD_W + cell_x as usize;
                self.grid[idx] = 1;
            }
        }

        self.events.locked = true;

        if touched_top && !self.game_over {
            self.game_over = true;
            input.sound_events.push(SoundEvent::GameOver);
        }

        self.clear_lines();
        self.piece = self.random_piece();
        self.hold_used = false;
        if !self.fits(self.piece.pos, self.piece.blocks) && !self.game_over {
            self.game_over = true;
            input.sound_events.push(SoundEvent::GameOver);
        }
    }

    fn compute_drop(&self) -> ((i32, i32), bool) {
        let mut pos = self.piece.pos;
        loop {
            let next = (pos.0, pos.1 - 1);
            if self.fits(next, self.piece.blocks) {
                pos = next;
            } else {
                return (pos, self.hits_block(pos, self.piece.blocks));
            }
        }
    }

    fn hits_block(&self, pos: (i32, i32), blocks: [(i32, i32); 4]) -> bool {
        for block in &blocks {
            let cell_x = pos.0 + block.0;
            let cell_y = pos.1 + block.1 - 1;
            if cell_y < 0 {
                continue;
            }
            if self.board_cell(cell_x, cell_y) != 0 {
                return true;
            }
        }
        false
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0u32;
        let mut y = 0;
        while y < TETRIS_BOARD_H as i32 {
            let mut filled = true;
            for x in 0..TETRIS_BOARD_W as i32 {
                if self.board_cell(x, y) == 0 {
                    filled = false;
                    break;
                }
            }
            if filled {
                for row in y..TETRIS_BOARD_H as i32 - 1 {
                    for x in 0..TETRIS_BOARD_W as i32 {
                        let idx = (row as usize) * TETRIS_BOARD_W + x as usize;
                        let above = ((row as usize) + 1) * TETRIS_BOARD_W + x as usize;
                        self.grid[idx] = self.grid[above];
                    }
                }
                for x in 0..TETRIS_BOARD_W {
                    self.grid[(TETRIS_BOARD_H - 1) * TETRIS_BOARD_W + x] = 0;
                }
                cleared += 1;
            } else {
                y += 1;
            }
        }

        if cleared > 0 {
            self.lines += cleared;
            let bonus = match cleared {
                1 => 100,
                2 => 300,
                3 => 500,
                _ => 800,
            };
            self.score = self.score.saturating_add(bonus * self.level);
            self.level = 1 + self.lines / 10;
            self.step_time = (STEP_TIME - self.level as f32 * 0.02).max(0.08);
            self.events.cleared = true;
        }
    }

    fn random_piece(&mut self) -> Piece {
        let kind = self.random_kind();
        Piece::new(kind)
    }

    fn random_kind(&mut self) -> PieceKind {
        let roll = (rand01(&mut self.rng_state) * 7.0) as i32;
        match roll {
            0 => PieceKind::I,
            1 => PieceKind::O,
            2 => PieceKind::T,
            3 => PieceKind::L,
            4 => PieceKind::J,
            5 => PieceKind::S,
            _ => PieceKind::Z,
        }
    }
}

impl Piece {
    fn new(kind: PieceKind) -> Self {
        let (blocks, pos) = match kind {
            PieceKind::I => (
                [(-1, 0), (0, 0), (1, 0), (2, 0)],
                (4, TETRIS_BOARD_H as i32 - 2),
            ),
            PieceKind::O => (
                [(0, 0), (1, 0), (0, 1), (1, 1)],
                (4, TETRIS_BOARD_H as i32 - 3),
            ),
            PieceKind::T => (
                [(-1, 0), (0, 0), (1, 0), (0, 1)],
                (4, TETRIS_BOARD_H as i32 - 3),
            ),
            PieceKind::L => (
                [(-1, 0), (0, 0), (1, 0), (1, 1)],
                (4, TETRIS_BOARD_H as i32 - 3),
            ),
            PieceKind::J => (
                [(-1, 1), (-1, 0), (0, 0), (1, 0)],
                (4, TETRIS_BOARD_H as i32 - 3),
            ),
            PieceKind::S => (
                [(-1, 0), (0, 0), (0, 1), (1, 1)],
                (4, TETRIS_BOARD_H as i32 - 3),
            ),
            PieceKind::Z => (
                [(-1, 1), (0, 1), (0, 0), (1, 0)],
                (4, TETRIS_BOARD_H as i32 - 3),
            ),
        };
        Self { kind, blocks, pos }
    }
}

fn offset_cells(blocks: [(i32, i32); 4], pos: (i32, i32)) -> [(i32, i32); 4] {
    [
        (pos.0 + blocks[0].0, pos.1 + blocks[0].1),
        (pos.0 + blocks[1].0, pos.1 + blocks[1].1),
        (pos.0 + blocks[2].0, pos.1 + blocks[2].1),
        (pos.0 + blocks[3].0, pos.1 + blocks[3].1),
    ]
}

fn piece_color(kind: PieceKind) -> u8 {
    match kind {
        PieceKind::I => 1,
        PieceKind::O => 2,
        PieceKind::T => 3,
        PieceKind::L => 4,
        PieceKind::J => 5,
        PieceKind::S => 6,
        PieceKind::Z => 7,
    }
}

fn rand01(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    ((*state >> 8) as f32) / ((u32::MAX >> 8) as f32)
}

fn is_inside_board(mouse_ui_x: f32, mouse_ui_y: f32) -> bool {
    let board_w = TETRIS_BOARD_W as f32 * TETRIS_CELL_SIZE;
    let board_h = TETRIS_BOARD_H as f32 * TETRIS_CELL_SIZE;
    mouse_ui_x >= TETRIS_BOARD_X
        && mouse_ui_x <= TETRIS_BOARD_X + board_w
        && mouse_ui_y >= TETRIS_BOARD_TOP - board_h
        && mouse_ui_y <= TETRIS_BOARD_TOP
}
