use ash::{
    extensions::ext::DebugUtils,
    vk, Entry, Instance,
};

pub mod utils;

pub fn initialize(
    entry: &Entry,
    instance: &Instance,
    enable_validation_layers: bool,
) -> (Option<DebugUtils>, Option<vk::DebugUtilsMessengerEXT>) {
    if !enable_validation_layers {
        return (None, None);
    }

    let debug_utils = DebugUtils::new(entry, instance);
    let create_info = utils::populate_debug_messenger_create_info();

    unsafe {
        let debug_utils_messenger_ext =
            DebugUtils::create_debug_utils_messenger(&debug_utils, &create_info, None)
                .expect("Failed to create DebugUtilsMessengerEXT");
        (Some(debug_utils), Some(debug_utils_messenger_ext))
    }
}