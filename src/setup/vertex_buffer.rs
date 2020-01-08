use ash::{
    version::DeviceV1_0,
    Instance,
    Device,
    vk,
};

use std::{
    mem::size_of_val,
    ptr::copy_nonoverlapping
};

use crate::structs::Vertex;
use crate::setup::buffer;

pub fn create(instance: &Instance, physical_device: &vk::PhysicalDevice, device: &Device, command_pool: vk::CommandPool, queue: vk::Queue, vertices: &[Vertex]) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size = (size_of_val(&vertices[0]) * vertices.len()) as vk::DeviceSize;
    let (staging_buffer, staging_buffer_memory) = buffer::create(instance, device, physical_device, buffer_size, vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
   
    unsafe {
        let data_ptr = device.map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty()).expect("Failed to map vertex buffer memory!");
        copy_nonoverlapping(vertices.as_ptr(), data_ptr as *mut Vertex, vertices.len());
        device.unmap_memory(staging_buffer_memory);
    };

    let (vertex_buffer, vertex_buffer_memory) = buffer::create(instance, device, physical_device, buffer_size, vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::DEVICE_LOCAL);
    buffer::copy(device, command_pool, queue, staging_buffer, vertex_buffer, buffer_size);

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }
    
    (vertex_buffer, vertex_buffer_memory)
}
