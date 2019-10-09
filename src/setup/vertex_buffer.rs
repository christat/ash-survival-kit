use ash::{
    version::{InstanceV1_0, DeviceV1_0},
    Instance,
    Device,
    vk
};

use std::{
    mem::size_of_val,
    ptr::copy_nonoverlapping
};

use crate::structs::Vertex;

pub fn create(instance: &Instance, physical_device: &vk::PhysicalDevice, device: &Device, vertices: &[Vertex]) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size((size_of_val(&vertices[0]) * vertices.len()) as vk::DeviceSize)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let vertex_buffer = unsafe { device.create_buffer(&buffer_create_info, None).expect("Failed to create vertex buffer!") };

    let memory_requirements = unsafe { device.get_buffer_memory_requirements(vertex_buffer) };

    let memory_type_index = find_memory_type_index(instance, physical_device, memory_requirements.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT).expect("Failed to find vertex buffer memory index!");
    let memory_allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(memory_requirements.size)
        .memory_type_index(memory_type_index)
        .build();

    let vertex_buffer_memory = unsafe { device.allocate_memory(&memory_allocate_info, None).expect("Failed to allocate vertex buffer memory!") };

    unsafe {
        device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0).expect("Failed to bind vertex buffer memory!");
        let data_ptr = device.map_memory(vertex_buffer_memory, 0, memory_allocate_info.allocation_size, vk::MemoryMapFlags::empty()).expect("Failed to map vertex buffer memory!");
        copy_nonoverlapping(vertices.as_ptr(), data_ptr as *mut Vertex, vertices.len());
        device.unmap_memory(vertex_buffer_memory);
    };

    (vertex_buffer, vertex_buffer_memory)
}

fn find_memory_type_index(instance: &Instance, physical_device: &vk::PhysicalDevice, type_filter: u32, memory_property_flags: vk::MemoryPropertyFlags) -> Result<u32, &'static str> {
    let physical_device_memory_properties = unsafe { instance.get_physical_device_memory_properties(*physical_device) };

    for (index, memory_type) in physical_device_memory_properties.memory_types.iter().enumerate() {
        let index = index as u32;
        if (type_filter & (1 << index)) > 0 && memory_type.property_flags.contains(memory_property_flags) {
            return Ok(index);
        }
    }
    Err("Failed to find suitable memory type for buffer!")
}

