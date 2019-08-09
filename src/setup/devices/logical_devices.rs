use std::os::raw::c_char;

extern crate ash;
use ash::{
    version::InstanceV1_0,
    vk, Instance,
};

use super::device_utils;
use crate::utils::debugging;

pub fn create_logical_device(instance: &Instance, physical_device: vk::PhysicalDevice, enable_validation_layers: bool) -> (ash::Device, u32) {
    let queue_family_index = device_utils::get_physical_device_queue_families(instance, physical_device).expect("No queue families contain required flags!") as u32;
    let queue_priorities: [f32; 1] = [1.0];

    let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&queue_priorities)
        .build()];

    let device_features = vk::PhysicalDeviceFeatures::builder().build();

    // variables below in main function body to prevent getting destroyed before entry.create_instance()
    let enabled_layer_names = debugging::get_enabled_layer_names();
    let enabled_layer_names: Vec<*const c_char> = enabled_layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let mut create_info_builder = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&device_features);
    if enable_validation_layers {
        create_info_builder = create_info_builder.enabled_layer_names(&enabled_layer_names);
    }
    let create_info = create_info_builder.build();

    let device = unsafe { instance.create_device(physical_device, &create_info, None).expect("Failed to create logical device!") };
    (device, queue_family_index)
}