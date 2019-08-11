use std::os::raw::c_char;

use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Win32Surface, Swapchain},
};

pub fn get_required_extensions() -> Vec<*const c_char> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

pub fn get_required_device_extensions() -> Vec<*const c_char> {
    vec![
        Swapchain::name().as_ptr()
    ]
}