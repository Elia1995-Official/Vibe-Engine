use cgmath::{vec3, Matrix4, Rad, Vector3};
use glium::backend::Facade;
use glium::implement_vertex;
use glium::index::PrimitiveType;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

implement_vertex!(Vertex, position, normal, color);

#[derive(Copy, Clone)]
pub struct StarVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub size: f32,
}

implement_vertex!(StarVertex, position, color, size);

pub struct Mesh {
    pub vertices: glium::VertexBuffer<Vertex>,
    pub indices: glium::IndexBuffer<u32>,
}

impl Mesh {
    pub fn new(display: &impl Facade, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices: glium::VertexBuffer::new(display, &vertices).expect("vertex buffer"),
            indices: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices)
                .expect("index buffer"),
        }
    }
}

pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: f32,
}

impl Transform {
    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from_angle_y(Rad(self.rotation.y))
            * Matrix4::from_angle_x(Rad(self.rotation.x))
            * Matrix4::from_angle_z(Rad(self.rotation.z))
            * Matrix4::from_scale(self.scale)
    }
}

pub fn identity_transform() -> Transform {
    Transform {
        position: vec3(0.0, 0.0, 0.0),
        rotation: vec3(0.0, 0.0, 0.0),
        scale: 1.0,
    }
}

pub fn mat4(m: Matrix4<f32>) -> [[f32; 4]; 4] {
    m.into()
}
