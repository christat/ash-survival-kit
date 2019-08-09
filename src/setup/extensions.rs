use std::os::raw::c_char;

extern crate ash;
use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Win32Surface},
};

pub fn get_required_extensions() -> Vec<*const c_char> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}