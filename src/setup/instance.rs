use std::{
    error::Error,
    ffi::CString,
    os::raw::{c_char, c_void},
};

use ash::{
    version::EntryV1_0,
    vk, Entry, Instance,
};

use super::debugging;
use super::extensions;

pub fn create_vk_instance(enable_validation_layers: bool) -> Result<(Entry, Instance), Box<dyn Error>> {
    let entry = Entry::new()?;
    if enable_validation_layers && !debugging::check_validation_layer_support(&entry) {
        panic!("Validation layers requested but not available!")
    }

    let application_name = CString::new("Hello triangle").unwrap();
    let engine_name = CString::new("No engine").unwrap();
    let version = ash::vk_make_version!(1, 0, 0);

    let application_info = vk::ApplicationInfo::builder()
        .application_name(&application_name)
        .application_version(version)
        .engine_name(&engine_name)
        .engine_version(version)
        .api_version(version)
        .build();

    // variables below in main function body to prevent getting destroyed before entry.create_instance()
    let enabled_layer_names = debugging::get_enabled_layer_names();
    let enabled_layer_names: Vec<*const c_char> = enabled_layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let enabled_extension_names = extensions::get_required_extensions();
    let create_info = debugging::populate_debug_messenger_create_info();

    let mut instance_create_info_builder =
        vk::InstanceCreateInfo::builder().application_info(&application_info);
    if enable_validation_layers {
        instance_create_info_builder = instance_create_info_builder
            .enabled_layer_names(&enabled_layer_names)
            .enabled_extension_names(&enabled_extension_names);
    }

    let mut instance_create_info = instance_create_info_builder.build();
    if enable_validation_layers {
        instance_create_info.p_next =
            &create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void;
    }

    let instance = unsafe { entry.create_instance(&instance_create_info, None)? };
    Ok((entry, instance))
}

