use ash::{
    version::DeviceV1_0,
    Instance,
    Device,
    vk
};

use std::mem::size_of;

use crate::structs::UBO;
use crate::setup::buffer;

pub fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .build();

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&[ubo_layout_binding])
        .build();

    let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None).expect("Failed to create descriptor set layout!") };
    descriptor_set_layout
}

pub fn create(instance: &Instance, device: &Device, physical_device: &vk::PhysicalDevice, swapchain_images: &[vk::Image]) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {
    let buffer_size = size_of::<UBO>() as vk::DeviceSize;

    let capacity = swapchain_images.len();
    let mut uniform_buffers = Vec::with_capacity(capacity);
    let mut uniform_buffers_memory = Vec::with_capacity(capacity);

    for _ in 0..capacity {
        let (buffer, memory) = buffer::create(instance, device, physical_device, buffer_size, vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        uniform_buffers.push(buffer);
        uniform_buffers_memory.push(memory);
    }

    (uniform_buffers, uniform_buffers_memory)
}