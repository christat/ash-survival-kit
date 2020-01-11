use ash::{
	Instance,
	Device,
	vk,
	vk::{
		DeviceSize,
		BufferUsageFlags,
		MemoryPropertyFlags,
		PhysicalDevice,
	},
	version::{DeviceV1_0, InstanceV1_0}
};

pub fn create(instance: &Instance, device: &Device, phys_device: &PhysicalDevice, size: DeviceSize, usage: BufferUsageFlags, properties: MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory) {
	let buffer_info = vk::BufferCreateInfo::builder()
		.size(size)
		.usage(usage)
		.sharing_mode(vk::SharingMode::EXCLUSIVE)
		.build();

	let buffer = unsafe { device.create_buffer(&buffer_info, None).expect("Failed to create buffer!") };
	let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

	let memory_type_index = find_memory_type_index(instance, phys_device, mem_requirements.memory_type_bits, properties).expect("failed to find memory type index!");
	let alloc_info = vk::MemoryAllocateInfo::builder()
		.allocation_size(mem_requirements.size)
		.memory_type_index(memory_type_index)
		.build();
		
	let buffer_memory = unsafe { device.allocate_memory(&alloc_info, None).expect("Failed to allocate buffer memory!") };

	unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0).expect("Failed to bind buffer memory!") };
	(buffer, buffer_memory)
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

pub fn copy(device: &Device, command_pool: vk::CommandPool, queue: vk::Queue, src_buffer: vk::Buffer, dst_buffer: vk::Buffer, size: vk::DeviceSize) {
	let allocate_info = vk::CommandBufferAllocateInfo::builder()
		.level(vk::CommandBufferLevel::PRIMARY)
		.command_pool(command_pool)
		.command_buffer_count(1)
		.build();

	let command_buffers = unsafe { device.allocate_command_buffers(&allocate_info).expect("Failed to allocate command buffers!") };
	let command_buffer = command_buffers.get(0).expect("Failed to retrieve command buffer!");

	let begin_info = vk::CommandBufferBeginInfo::builder()
		.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
		.build();

	unsafe { device.begin_command_buffer(*command_buffer, &begin_info).expect("Failed to begin command buffer!") };

	let copy_region = vk::BufferCopy::builder()
		.src_offset(0)
		.dst_offset(0)
		.size(size)
		.build();

	unsafe { 
		device.cmd_copy_buffer(*command_buffer, src_buffer, dst_buffer, &[copy_region]);
		device.end_command_buffer(*command_buffer).expect("Failed to end command buffer!");
	}

	let submit_info = vk::SubmitInfo::builder()
		.command_buffers(&command_buffers)
		.build();

	unsafe {
		device.queue_submit(queue, &[submit_info], vk::Fence::null()).expect("Failed to submit to queue!");
		device.queue_wait_idle(queue).expect("Failed waiting for queue idle!");
		device.free_command_buffers(command_pool, &command_buffers)
	}
}