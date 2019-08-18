use std::os::raw::c_char;

use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain, Win32Surface},
};

pub fn get_instance_extensions() -> Vec<*const c_char> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(), // TODO replace with platform-specific getter
        DebugUtils::name().as_ptr(),
    ]
}

pub fn get_device_extensions() -> Vec<*const c_char> {
    vec![Swapchain::name().as_ptr()]
}
