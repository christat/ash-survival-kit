use std::ffi::{CStr, CString};

use ash::{version::EntryV1_0, Entry};

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
