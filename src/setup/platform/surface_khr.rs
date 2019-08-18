use std::{os::raw::c_void, ptr};

use ash::{
    extensions::khr::Win32Surface,
    vk, Entry, Instance,
};

extern crate winapi;
use winapi::um::libloaderapi::GetModuleHandleW;

use winit::{
    platform::windows::WindowExtWindows,
    window::Window,
};

pub fn create(entry: &Entry, instance: &Instance, window: &Window) -> vk::SurfaceKHR {
    let win_32_surface = Win32Surface::new(entry, instance);
    let win_32_surface_create_info = vk::Win32SurfaceCreateInfoKHR::builder()
        .hwnd(window.hwnd())
        .hinstance(unsafe { GetModuleHandleW(ptr::null()) as *const c_void })
        .build();

    let surface = unsafe {
        win_32_surface
            .create_win32_surface(&win_32_surface_create_info, None)
            .expect("Failed to create window surface!")
    };
    surface
}