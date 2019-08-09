extern crate ash;
use ash::{
    version::InstanceV1_0,
    vk, Instance,
    extensions::khr::Surface,
};

pub struct QueueFamilyIndices {
    pub graphics: Option<u32>,
    pub presentation: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn default() -> Self {
        Self {
            graphics: None,
            presentation: None
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.presentation.is_some()
    }
}
pub fn is_physical_device_suitable(instance: &Instance, device: vk::PhysicalDevice, surface: &Surface, surface_khr: vk::SurfaceKHR) -> bool {
    let queue_family_indices = get_physical_device_queue_family_indices(instance, device, surface, surface_khr);
    queue_family_indices.is_some()
}

pub fn get_physical_device_queue_family_indices(instance: &Instance, device: vk::PhysicalDevice, surface: &Surface, surface_khr: vk::SurfaceKHR) -> Option<QueueFamilyIndices> {
    let queue_family_properties_vec = unsafe { instance.get_physical_device_queue_family_properties(device) };;
    let indices = 0..queue_family_properties_vec.len() as u32;

    let queue_family_indices = queue_family_properties_vec.into_iter().zip(indices).fold(QueueFamilyIndices::default(), |mut queue_family_indices, (queue_family_properties, index)| {
        if queue_family_indices.graphics.is_none() && queue_family_properties.queue_count > 0 && queue_family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            queue_family_indices.graphics = Some(index);
        }

        let supports_surface = unsafe { surface.get_physical_device_surface_support(device, index, surface_khr) };
        if  queue_family_indices.presentation.is_none() && supports_surface {
            queue_family_indices.presentation = Some(index);
        }

        queue_family_indices
    });

    if queue_family_indices.is_complete() { Some(queue_family_indices) } else { None }
}