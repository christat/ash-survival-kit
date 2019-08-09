extern crate ash;
use ash::{
    vk,
    Entry,
    Instance,
    extensions::{
        ext::DebugUtils
    }
};

pub mod devices;
pub mod extensions;
pub mod instance;

use super::utils::debugging;

pub fn init_vulkan(enable_validation_layers: bool) -> (Entry, Instance) {
    let instance_result = instance::create_vk_instance(enable_validation_layers);
    match instance_result {
        Ok((entry, instance)) => (entry, instance),
        Err(e) => panic!("Failed to create Vulkan instance: {}", e),
    }
}

pub fn init_debug_messenger(
    entry: &Entry,
    instance: &Instance,
    enable_validation_layers: bool
) -> Option<(DebugUtils, vk::DebugUtilsMessengerEXT)> {
    if !enable_validation_layers { return None }

    let debug_utils = DebugUtils::new(entry, instance);
    let create_info = debugging::populate_debug_messenger_create_info();

    unsafe {
        let debug_utils_messenger_ext =
            DebugUtils::create_debug_utils_messenger(&debug_utils, &create_info, None)
                .expect("Failed to create DebugUtilsMessengerEXT");
        Some((debug_utils, debug_utils_messenger_ext))
    }
}

