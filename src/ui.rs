use cgmath::{ortho, Matrix4, Vector2};

use crate::graphics::Vertex;

#[derive(Copy, Clone)]
pub enum MenuButton {
    SpaceShooter,
    Snake,
    Fps,
    Platformer,
    Tetris,
    Start,
    Quit,
    Settings,
    BackFromSettings,
}

#[derive(Copy, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(self, point: Vector2<f32>) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }
}

pub fn ui_projection(aspect: f32) -> Matrix4<f32> {
    let half_height = 5.0;
    let half_width = half_height * aspect;
    ortho(
        -half_width,
        half_width,
        -half_height,
        half_height,
        -1.0,
        1.0,
    )
}

pub fn mouse_to_ui(mouse_ndc: Vector2<f32>, aspect: f32) -> Vector2<f32> {
    Vector2::new(mouse_ndc.x * 5.0 * aspect, mouse_ndc.y * 5.0)
}

pub fn menu_button(button: MenuButton) -> Rect {
    match button {
        MenuButton::SpaceShooter => Rect::new(-4.05, 0.15, 1.55, 0.92),
        MenuButton::Snake => Rect::new(-2.42, 0.15, 1.55, 0.92),
        MenuButton::Fps => Rect::new(-0.79, 0.15, 1.55, 0.92),
        MenuButton::Platformer => Rect::new(0.84, 0.15, 1.55, 0.92),
        MenuButton::Tetris => Rect::new(2.47, 0.15, 1.55, 0.92),
        MenuButton::Start => Rect::new(-1.7, -0.45, 3.4, 0.62),
        MenuButton::Quit => Rect::new(-1.7, -1.25, 3.4, 0.62),
        MenuButton::Settings => Rect::new(-1.7, -2.05, 3.4, 0.55),
        MenuButton::BackFromSettings => Rect::new(-1.5, -3.6, 3.0, 0.62),
    }
}

/// Track rect for the nth volume slider (0 = master, 1 = music, 2 = sfx).
pub fn slider_track_rect(index: usize) -> Rect {
    let y = 1.4 - index as f32 * 1.4;
    Rect::new(-2.8, y - 0.14, 5.6, 0.28)
}

/// Given a slider track rect and a 0..=1 fill fraction, returns the filled
/// portion rect and the handle rect.
pub fn slider_fill_and_handle(track: Rect, value: f32) -> (Rect, Rect) {
    let fill = Rect::new(track.x, track.y, track.w * value, track.h);
    let handle = Rect::new(
        track.x + track.w * value - 0.2,
        track.y - 0.1,
        0.4,
        track.h + 0.2,
    );
    (fill, handle)
}

pub fn text_width(text: &str, scale: f32) -> f32 {
    text.chars()
        .map(|ch| if ch == ' ' { 4.0 } else { 6.0 })
        .sum::<f32>()
        * scale
}

pub fn text_height(scale: f32) -> f32 {
    7.0 * scale
}

pub fn text_mesh(
    text: &str,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut cursor = x;

    for ch in text.chars() {
        if ch == ' ' {
            cursor += 4.0 * scale;
            continue;
        }

        let glyph = glyph(ch);
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) == 0 {
                    continue;
                }

                let gx = cursor + col as f32 * scale;
                let gy = y + (6 - row) as f32 * scale;
                let base = vertices.len() as u32;
                let n = [0.0, 0.0, 1.0];
                vertices.extend_from_slice(&[
                    Vertex {
                        position: [gx, gy, 0.0],
                        normal: n,
                        color,
                    },
                    Vertex {
                        position: [gx + scale * 0.78, gy, 0.0],
                        normal: n,
                        color,
                    },
                    Vertex {
                        position: [gx + scale * 0.78, gy + scale * 0.78, 0.0],
                        normal: n,
                        color,
                    },
                    Vertex {
                        position: [gx, gy + scale * 0.78, 0.0],
                        normal: n,
                        color,
                    },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        cursor += 6.0 * scale;
    }

    (vertices, indices)
}

fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10011, 0b10101, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        _ => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
    }
}
