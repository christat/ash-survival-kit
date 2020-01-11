use std::mem::size_of;

use ash::vk;
use cgmath::{
    Matrix4,
    Vector2,
    Vector3
};
use cgmath::prelude::*;

use field_offset::offset_of;

pub struct Vertex {
    position: Vector2<f32>,
    color: Vector3<f32>
}

impl Vertex {
    pub fn new(x: f32, y: f32, r: f32, g: f32, b: f32) -> Self {
        Self {
            position: Vector2 { x, y },
            color: Vector3 { x: r, y: g, z: b }
        }
    }

    pub fn get_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        let binding_description = [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()];
        binding_description
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let attribute_descriptions = [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex => position).get_byte_offset() as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex => color).get_byte_offset() as u32)
                .build()
        ];
        attribute_descriptions
    }
}

pub struct UBO {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    projection: Matrix4<f32>
}

impl UBO {
    pub fn new(model: Matrix4<f32>, view: Matrix4<f32>, projection: Matrix4<f32>) -> Self {
        Self { model, view, projection }
    }
}