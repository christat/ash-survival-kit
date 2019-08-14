use ash::{
    Device,
    vk,
    version::DeviceV1_0
};

use crate::setup::devices::utils::QueueFamilyIndices;

pub fn create(device: &Device, queue_family_indices: &QueueFamilyIndices) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics)
        .build();

    let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None).expect("Failed to create command pool!") };
    command_pool
}