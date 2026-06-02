use cgmath::{vec3, InnerSpace};
use rand::rngs::StdRng;
use rand::Rng;

use crate::config::STAR_COUNT_PER_LAYER;
use crate::graphics::{StarVertex, Vertex};

pub fn procedural_ship(rng: &mut StdRng) -> Vec<Vertex> {
    let accent = [
        rng.gen_range(0.15..0.35),
        rng.gen_range(0.55..0.95),
        rng.gen_range(0.88..1.0),
    ];
    let hull = [0.68, 0.78, 0.9];
    let dark = [0.08, 0.12, 0.2];
    let glow = [0.2, 0.95, 1.0];
    let points = [
        (vec3(0.0, 0.72, -1.85), hull),
        (vec3(-0.58, -0.18, -0.3), hull),
        (vec3(0.58, -0.18, -0.3), hull),
        (vec3(0.0, -0.36, 1.0), dark),
        (vec3(-1.45, -0.34, 0.55), accent),
        (vec3(1.45, -0.34, 0.55), accent),
        (vec3(-0.32, -0.2, 1.28), glow),
        (vec3(0.32, -0.2, 1.28), glow),
    ];
    points
        .into_iter()
        .map(|(position, color)| Vertex {
            position: position.into(),
            normal: [0.0, 1.0, 0.0],
            color,
        })
        .collect()
}

pub fn ship_indices() -> Vec<u32> {
    vec![
        0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 4, 3, 2, 3, 5, 3, 7, 6, 3, 6, 1, 3, 2, 7, 1, 6, 4, 2, 5, 7,
    ]
}

pub fn bullet_mesh() -> Vec<Vertex> {
    let c = [0.25, 0.95, 1.0];
    let n = [0.0, 0.0, 1.0];
    [
        [-0.035, -0.035, -0.38],
        [0.035, -0.035, -0.38],
        [0.035, 0.035, -0.38],
        [-0.035, 0.035, -0.38],
        [-0.055, -0.055, 0.22],
        [0.055, -0.055, 0.22],
        [0.055, 0.055, 0.22],
        [-0.055, 0.055, 0.22],
    ]
    .into_iter()
    .map(|position| Vertex {
        position,
        normal: n,
        color: c,
    })
    .collect()
}

pub fn procedural_asteroid(lats: usize, longs: usize, seed: u64) -> (Vec<Vertex>, Vec<u32>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=lats {
        let theta = std::f32::consts::PI * lat as f32 / lats as f32;
        for lon in 0..=longs {
            let phi = std::f32::consts::TAU * lon as f32 / longs as f32;
            let rough = rng.gen_range(0.72..1.22);
            let p = vec3(
                theta.sin() * phi.cos() * rough,
                theta.cos() * rough,
                theta.sin() * phi.sin() * rough,
            );
            let tint = rng.gen_range(0.0..0.13);
            vertices.push(Vertex {
                position: p.into(),
                normal: p.normalize().into(),
                color: [0.36 + tint, 0.31 + tint, 0.27 + tint],
            });
        }
    }

    let row = longs + 1;
    for lat in 0..lats {
        for lon in 0..longs {
            let a = (lat * row + lon) as u32;
            let b = a + row as u32;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }

    (vertices, indices)
}

pub fn generate_stars(layer: usize, rng: &mut StdRng) -> Vec<StarVertex> {
    let depth_offset = layer as f32 * 18.0;
    let spread = 55.0 + layer as f32 * 16.0;
    (0..STAR_COUNT_PER_LAYER)
        .map(|_| {
            let brightness = rng.gen_range(0.45..1.0);
            StarVertex {
                position: [
                    rng.gen_range(-spread..spread),
                    rng.gen_range(-spread * 0.55..spread * 0.55),
                    rng.gen_range(-130.0 - depth_offset..-18.0 - depth_offset),
                ],
                color: [
                    brightness * rng.gen_range(0.72..1.0),
                    brightness * rng.gen_range(0.78..1.0),
                    brightness,
                ],
                size: rng.gen_range(1.5..4.4) + layer as f32,
            }
        })
        .collect()
}

pub fn quad_mesh(x: f32, y: f32, w: f32, h: f32, color: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
    let z = 0.0;
    let n = [0.0, 0.0, 1.0];
    let vertices = vec![
        Vertex {
            position: [x, y, z],
            normal: n,
            color,
        },
        Vertex {
            position: [x + w, y, z],
            normal: n,
            color,
        },
        Vertex {
            position: [x + w, y + h, z],
            normal: n,
            color,
        },
        Vertex {
            position: [x, y + h, z],
            normal: n,
            color,
        },
    ];
    (vertices, vec![0, 1, 2, 0, 2, 3])
}

pub fn cube_mesh(color: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let faces = [
        (
            [0.0, 0.0, 1.0],
            [
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
        ),
        (
            [0.0, 0.0, -1.0],
            [
                [0.5, -0.5, -0.5],
                [-0.5, -0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [0.5, 0.5, -0.5],
            ],
        ),
        (
            [0.0, 1.0, 0.0],
            [
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
            ],
        ),
        (
            [0.0, -1.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
        ),
        (
            [1.0, 0.0, 0.0],
            [
                [0.5, -0.5, 0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
            ],
        ),
        (
            [-1.0, 0.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, 0.5, -0.5],
            ],
        ),
    ];

    for (normal, positions) in faces {
        let base = vertices.len() as u32;
        for position in positions {
            vertices.push(Vertex {
                position,
                normal,
                color,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    (vertices, indices)
}

pub fn arena_floor_mesh(half_size: f32, color: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
    let n = [0.0, 1.0, 0.0];
    let vertices = vec![
        Vertex {
            position: [-half_size, 0.0, -half_size],
            normal: n,
            color,
        },
        Vertex {
            position: [half_size, 0.0, -half_size],
            normal: n,
            color,
        },
        Vertex {
            position: [half_size, 0.0, half_size],
            normal: n,
            color,
        },
        Vertex {
            position: [-half_size, 0.0, half_size],
            normal: n,
            color,
        },
    ];
    (vertices, vec![0, 1, 2, 0, 2, 3])
}

use rand::SeedableRng;
