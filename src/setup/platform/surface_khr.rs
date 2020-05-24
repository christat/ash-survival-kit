use std::{os::raw::c_void, ptr};

use ash::{extensions::khr::Win32Surface, vk, Entry, Instance};

use winapi::um::libloaderapi::GetModuleHandleW;
use winit::{platform::windows::WindowExtWindows, window::Window};

pub fn create(entry: &Entry, instance: &Instance, window: &Window) -> vk::SurfaceKHR {
    let win32_surface_loader = Win32Surface::new(entry, instance);

    let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
        .hinstance(unsafe { GetModuleHandleW(ptr::null()) as vk::HINSTANCE })
        .hwnd(window.hwnd() as vk::HWND)
        .build();

    unsafe {
        win32_surface_loader
            .create_win32_surface(&create_info, None)
            .expect("Failed to create win32 surface!")
    }
}
