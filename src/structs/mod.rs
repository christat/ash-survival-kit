use std::mem::size_of;

use ash::vk;
use cgmath::{Matrix4, Vector2, Vector3};

use field_offset::offset_of;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, u: f32, v: f32) -> Self {
        Self {
            position: Vector3 { x, y, z },
            color: Vector3 { x: r, y: g, z: b },
            uv: Vector2 { x: u, y: v },
        }
    }

    pub fn get_binding_description() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    pub fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex => position).get_byte_offset() as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex => color).get_byte_offset() as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex => uv).get_byte_offset() as u32)
                .build(),
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UBO {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}
