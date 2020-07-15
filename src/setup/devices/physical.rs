use ash::{extensions::khr::Surface, version::InstanceV1_0, vk, Instance};

use crate::setup::devices::utils;

pub fn select(
    instance: &Instance,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, vk::SampleCountFlags) {
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

    let device = suitable_devices
        .first()
        .expect("Failed to retrieve suitable physical device!")
        .to_owned();
    (device, get_max_usable_sample_count(instance, &device))
}

pub fn get_max_usable_sample_count(
    instance: &Instance,
    device: &vk::PhysicalDevice,
) -> vk::SampleCountFlags {
    let properties = unsafe { instance.get_physical_device_properties(*device) };
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;

    if counts.contains(vk::SampleCountFlags::TYPE_64) {
        return vk::SampleCountFlags::TYPE_64;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_32) {
        return vk::SampleCountFlags::TYPE_32;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_16) {
        return vk::SampleCountFlags::TYPE_16;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_8) {
        return vk::SampleCountFlags::TYPE_8;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_4) {
        return vk::SampleCountFlags::TYPE_4;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_2) {
        return vk::SampleCountFlags::TYPE_2;
    }

    vk::SampleCountFlags::TYPE_1
}
