use std::{collections::HashSet, os::raw::c_char};

use ash::{extensions::khr::Surface, version::InstanceV1_0, vk, Instance};

use crate::{
    setup::{devices::utils, extensions, validation_layers::utils as debug_utils},
};

pub fn create(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
    enable_validation_layers: bool,
) -> (ash::Device, utils::QueueFamilyIndices) {
    let queue_family_indices = utils::get_physical_device_queue_family_indices(
        instance,
        physical_device,
        surface,
        surface_khr,
    )
    .expect("No queue families contain required flags!");
    let unique_queue_family_indices: HashSet<u32> =
        [queue_family_indices.graphics, queue_family_indices.present]
            .iter()
            .cloned()
            .collect();
    let queue_priorities: [f32; 1] = [1.0];

    let queue_create_infos = unique_queue_family_indices
        .into_iter()
        .map(|index| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(index)
                .queue_priorities(&queue_priorities)
                .build()
        })
        .collect::<Vec<vk::DeviceQueueCreateInfo>>();

    let device_features = vk::PhysicalDeviceFeatures::builder().build();

    // variables below in main function body to prevent getting destroyed before entry.create_instance()
    let enabled_layer_names = debug_utils::get_enabled_layer_names();
    let enabled_layer_names: Vec<*const c_char> = enabled_layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let enabled_extension_names = extensions::get_device_extensions();

    let mut create_info_builder = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&device_features)
        .enabled_extension_names(&enabled_extension_names);
    if enable_validation_layers {
        create_info_builder = create_info_builder.enabled_layer_names(&enabled_layer_names);
    }
    let create_info = create_info_builder.build();

    let device = unsafe {
        instance
            .create_device(physical_device, &create_info, None)
            .expect("Failed to create logical device!")
    };
    (device, queue_family_indices)
}
