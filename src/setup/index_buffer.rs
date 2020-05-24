use ash::{version::DeviceV1_0, vk, Device, Instance};

use std::{mem::size_of, ptr::copy_nonoverlapping};

use crate::setup::buffer;

pub fn create(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    device: &Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    indices: &[u32],
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size = (size_of::<u32>() * indices.len()) as vk::DeviceSize;
    let (staging_buffer, staging_buffer_memory) = buffer::create(
        instance,
        device,
        physical_device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let data_ptr = device
            .map_memory(
                staging_buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to map vertex buffer memory!");
        copy_nonoverlapping(indices.as_ptr(), data_ptr as *mut u32, indices.len());
        device.unmap_memory(staging_buffer_memory);
    };

    let (index_buffer, index_buffer_memory) = buffer::create(
        instance,
        device,
        physical_device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    buffer::copy(
        device,
        command_pool,
        queue,
        staging_buffer,
        index_buffer,
        buffer_size,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    (index_buffer, index_buffer_memory)
}
