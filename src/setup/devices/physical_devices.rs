extern crate ash;
use ash::{
    version::InstanceV1_0,
    vk, Instance,
    extensions::khr::Surface
};

use super::device_utils;

pub fn select_physical_device(instance: &Instance, surface: &Surface, surface_khr: vk::SurfaceKHR) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };

    if physical_devices.len() == 0 {
        panic!("No physical devices with Vulkan support!");
    }

    let suitable_devices = physical_devices
        .into_iter()
        .filter_map(|device| {
            if device_utils::is_physical_device_suitable(instance, device, surface, surface_khr) {
                Some(device)
            } else {
                None
            }
        }).collect::<Vec<vk::PhysicalDevice>>();

    if suitable_devices.len() == 0 {
        panic!("No suitable devices found!")
    }

    suitable_devices[0]
}