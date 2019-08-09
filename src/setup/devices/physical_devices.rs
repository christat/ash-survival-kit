extern crate ash;
use ash::{
    version::InstanceV1_0,
    vk, Instance,
};

use super::device_utils;

pub fn select_physical_device(instance: &Instance) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };

    if physical_devices.len() == 0 {
        panic!("No physical devices with Vulkan support!");
    }

    let selected_device = physical_devices
        .into_iter()
        .filter_map(|device| {
            if device_utils::is_physical_device_suitable(instance, device) {
                Some(device)
            } else {
                None
            }
        }).collect::<Vec<vk::PhysicalDevice>>();

    if selected_device.len() == 0 {
        panic!("No suitable devices found!")
    }

    selected_device[0]
}