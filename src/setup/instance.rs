use std::{
    ffi::CString,
    os::raw::c_char,
};

use ash::{version::EntryV1_0, vk, Entry, Instance};

use super::extensions;
use super::validation_layers::utils as debug_utils;

pub fn create(
    enable_validation_layers: bool,
) -> (Entry, Instance) {
    let entry = Entry::new().expect("Failed to instantiate Vulkan entry!");
    if enable_validation_layers && !debug_utils::check_validation_layer_support(&entry) {
        panic!("Validation layers requested but not available!")
    }

    // Application Info
    let application_name = CString::new("Hello triangle").unwrap();
    let engine_name = CString::new("No engine").unwrap();
    let version = vk::make_version(1, 0, 0);

    let application_info = vk::ApplicationInfo::builder()
        .application_name(&application_name)
        .application_version(version)
        .engine_name(&engine_name)
        .engine_version(version)
        .api_version(version)
        .build();

    // Instance Create Info
    // variables below in main function body to prevent getting destroyed before entry.create_instance()
    let enabled_layer_names = debug_utils::get_enabled_layer_names();
    let enabled_layer_names: Vec<*const c_char> = enabled_layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let enabled_extension_names = extensions::get_instance_extensions();
    let mut debug_utils_messenger_create_info = debug_utils::populate_debug_messenger_create_info();

    let mut instance_create_info_builder =
        vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(&enabled_extension_names);

    if enable_validation_layers {
        instance_create_info_builder = instance_create_info_builder
            .enabled_layer_names(&enabled_layer_names)
            .push_next(&mut debug_utils_messenger_create_info);
    }

    let instance_create_info = instance_create_info_builder.build();

    // Instance creation
    let instance = unsafe { entry.create_instance(&instance_create_info, None).expect("Failed to create Vulkan instance!") };
    (entry, instance)
}
