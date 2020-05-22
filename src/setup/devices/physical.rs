use ash::{extensions::khr::Surface, version::InstanceV1_0, vk, Instance};

use crate::setup::devices::utils;

pub fn select(
    instance: &Instance,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };

    if physical_devices.is_empty() {
        panic!("No physical devices with Vulkan support!");
    }

    let suitable_devices = physical_devices
        .into_iter()
        .filter(|device| {
            utils::is_physical_device_suitable(instance, *device, surface, surface_khr)
        })
        .collect::<Vec<vk::PhysicalDevice>>();

    if suitable_devices.is_empty() {
        panic!("No suitable devices found!")
    }

    suitable_devices[0]
}
