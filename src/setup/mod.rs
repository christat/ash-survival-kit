use std::{
    os::raw::c_void,
    ptr
};

use ash::{
    vk,
    Entry,
    Instance,
    extensions::{
        ext::DebugUtils,
        khr::Win32Surface
    }
};

use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
    platform::windows::WindowExtWindows
};

extern crate winapi;
use winapi::um::libloaderapi::GetModuleHandleW;

pub mod devices;
pub mod extensions;
pub mod instance;
pub mod swapchain;

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
) -> (Option<DebugUtils>, Option<vk::DebugUtilsMessengerEXT>) {
    if !enable_validation_layers { return (None, None) }

    let debug_utils = DebugUtils::new(entry, instance);
    let create_info = debugging::populate_debug_messenger_create_info();

    unsafe {
        let debug_utils_messenger_ext =
            DebugUtils::create_debug_utils_messenger(&debug_utils, &create_info, None)
                .expect("Failed to create DebugUtilsMessengerEXT");
        (Some(debug_utils), Some(debug_utils_messenger_ext))
    }
}

pub fn init_window() -> (EventLoop<()>, Window) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(crate::WINDOW_WIDTH as f64, crate::WINDOW_HEIGHT as f64))
        .with_title("Vulkan tutorial")
        .build(&event_loop)
        .expect("Failed to create window!");

    (event_loop, window)
}

pub fn init_surface_khr(entry: &Entry, instance: &Instance, window: &Window) -> vk::SurfaceKHR {
    let win_32_surface = Win32Surface::new(entry, instance);
    let win_32_surface_create_info = vk::Win32SurfaceCreateInfoKHR::builder()
        .hwnd(window.hwnd())
        .hinstance(unsafe { GetModuleHandleW(ptr::null()) as *const c_void })
        .build();

    let surface = unsafe { win_32_surface.create_win32_surface(&win_32_surface_create_info, None).expect("Failed to create window surface!") };
    surface
}

