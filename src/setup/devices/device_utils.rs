extern crate ash;
use ash::{
    version::InstanceV1_0,
    vk, Instance,
};

pub fn is_physical_device_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool {
    let queue_family_index = get_physical_device_queue_families(instance, device);
    queue_family_index.is_some()
}

pub fn get_physical_device_queue_families(instance: &Instance, device: vk::PhysicalDevice) -> Option<usize> {
    let queue_families_properties = unsafe { instance.get_physical_device_queue_family_properties(device) };
    let queue_families_indices = 0..queue_families_properties.len();
    let queue_family_indices = queue_families_properties.into_iter().zip(queue_families_indices).flat_map(|(queue_family, index)| {
        if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            Some(index)
        } else {
            None
        }
    }).collect::<Vec<usize>>();

    if queue_family_indices.len() == 0 {
        None
    } else {
        Some(queue_family_indices[0])
    }
}