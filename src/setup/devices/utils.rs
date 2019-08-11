use std::ffi::{CStr, CString};

use ash::{
    version::InstanceV1_0,
    vk, Instance,
    extensions::khr::Surface,
};

use crate::setup::{extensions, swapchain};

pub fn is_physical_device_suitable(instance: &Instance, device: vk::PhysicalDevice, surface: &Surface, surface_khr: vk::SurfaceKHR) -> bool {
    let supports_required_extensions = check_device_extension_support(instance, device);
    let supports_required_queue_families = get_physical_device_queue_family_indices(instance, device, surface, surface_khr).is_some();

    let swap_chain_is_adequate = if supports_required_extensions {
        let swap_chain_details = swapchain::utils::query_swapchain_support(device, surface, surface_khr);
        !swap_chain_details.formats.is_empty() && !swap_chain_details.present_modes.is_empty()
    } else {
        false
    };

    supports_required_queue_families && supports_required_extensions && swap_chain_is_adequate
}

pub fn check_device_extension_support(instance: &Instance, device: vk::PhysicalDevice) -> bool {
    // TODO improve extension name generation; this feels overly complicated ¯\_(ツ)_/¯
    let supported_extensions = unsafe { instance.enumerate_device_extension_properties(device).expect("Failed to request supported device extensions") };
    let supported_extension_names = supported_extensions.into_iter().map(|properties| unsafe { CStr::from_ptr(properties.extension_name.as_ptr()).to_owned().into_boxed_c_str().into_c_string() } ).collect::<Vec<CString>>();
    let mut required_extension_names = extensions::get_required_device_extensions().into_iter().map(|extension_name| unsafe { CStr::from_ptr(extension_name).to_owned().into_boxed_c_str().into_c_string() }).collect::<Vec<CString>>();

    required_extension_names.retain(|required_extension_name| !supported_extension_names.contains(required_extension_name));
    required_extension_names.len() == 0
}

pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}

struct InnerQueueFamilyIndices {
    pub graphics: Option<u32>,
    pub present: Option<u32>
}

impl InnerQueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
}

pub fn get_physical_device_queue_family_indices(instance: &Instance, physical_device: vk::PhysicalDevice, surface: &Surface, surface_khr: vk::SurfaceKHR) -> Option<QueueFamilyIndices> {
    let queue_family_properties_vec = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let indices = 0..queue_family_properties_vec.len() as u32;

    let inner_queue_family_indices = queue_family_properties_vec.into_iter().zip(indices).fold(InnerQueueFamilyIndices { graphics: None, present: None }, |mut indices, (queue_family_properties, queue_index)| {
        if indices.graphics.is_none() && queue_family_properties.queue_count > 0 && queue_family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            indices.graphics = Some(queue_index);
        }

        let supports_surface = unsafe { surface.get_physical_device_surface_support(physical_device, queue_index, surface_khr) };
        if  indices.present.is_none() && supports_surface {
            indices.present = Some(queue_index);
        }

        indices
    });

    let InnerQueueFamilyIndices { graphics, present } = inner_queue_family_indices;
    if inner_queue_family_indices.is_complete() { Some(QueueFamilyIndices { graphics: graphics.unwrap(), present: present.unwrap() }) } else { None }
}

