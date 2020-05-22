use ash::{version::DeviceV1_0, vk, Device, Instance};

use std::mem::size_of;

use crate::setup::buffer;
use crate::structs::UBO;

pub fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
    let bindings = [
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
    ];

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings)
        .build();

    let descriptor_set_layout = unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .expect("Failed to create descriptor set layout!")
    };
    descriptor_set_layout
}

pub fn create(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    swapchain_images: &[vk::Image],
) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {
    let buffer_size = size_of::<UBO>() as vk::DeviceSize;

    let capacity = swapchain_images.len();
    let mut uniform_buffers = Vec::with_capacity(capacity);
    let mut uniform_buffers_memory = Vec::with_capacity(capacity);

    for _ in 0..capacity {
        let (buffer, memory) = buffer::create(
            instance,
            device,
            physical_device,
            buffer_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        uniform_buffers.push(buffer);
        uniform_buffers_memory.push(memory);
    }

    (uniform_buffers, uniform_buffers_memory)
}

pub fn create_descriptor_pool(
    device: &Device,
    swapchain_images: &[vk::Image],
) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(swapchain_images.len() as u32)
            .build(),
        vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(swapchain_images.len() as u32)
            .build(),
    ];

    let pool_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(swapchain_images.len() as u32)
        .build();

    let descriptor_pool = unsafe {
        device
            .create_descriptor_pool(&pool_info, None)
            .expect("Failed to create descriptor pool!")
    };
    descriptor_pool
}

pub fn create_descriptor_sets(
    device: &Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    uniform_buffers: &[vk::Buffer],
    swapchain_images: &[vk::Image],
    image_view: vk::ImageView,
    sampler: vk::Sampler,
) -> Vec<vk::DescriptorSet> {
    let layouts = vec![descriptor_set_layout; swapchain_images.len()];

    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts)
        .build();

    let descriptor_sets = unsafe {
        device
            .allocate_descriptor_sets(&alloc_info)
            .expect("Failed to allocate descriptor sets!")
    };

    descriptor_sets
        .iter()
        .zip(uniform_buffers)
        .for_each(|(descriptor_set, uniform_buffer)| {
            let buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(*uniform_buffer)
                .offset(0)
                .range(size_of::<UBO>() as u64)
                .build();

            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(image_view)
                .sampler(sampler)
                .build();

            let descriptor_writes = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(*descriptor_set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&[buffer_info])
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(*descriptor_set)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[image_info])
                    .build(),
            ];

            unsafe {
                device.update_descriptor_sets(&descriptor_writes, &[]);
            }
        });

    descriptor_sets
}
