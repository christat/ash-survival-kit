use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
};

use ash::{version::EntryV1_0, vk, Entry};

const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub fn check_validation_layer_support(entry: &Entry) -> bool {
    let layer_properties = match entry.enumerate_instance_layer_properties() {
        Ok(layer_properties) => layer_properties,
        Err(e) => panic!("Failed to enumerate instance layer properties: {}", e),
    };
    let layer_names = layer_properties
        .iter()
        .map(|lp| unsafe { CStr::from_ptr(lp.layer_name.as_ptr()).to_str().unwrap() })
        .collect::<Vec<&str>>();
    for validation_layer in VALIDATION_LAYERS.iter() {
        if !layer_names.contains(validation_layer) {
            return false;
        }
    }
    true
}

pub fn get_enabled_layer_names() -> Vec<CString> {
    VALIDATION_LAYERS
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect()
}

pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(pfn_user_callback))
        .build()
}

#[allow(unused_variables)]
unsafe extern "system" fn pfn_user_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void,
) -> vk::Bool32 {
    println!(
        "validation layer: {}",
        CStr::from_ptr((*p_callback_data).p_message)
            .to_str()
            .unwrap()
    );
    vk::FALSE
}
