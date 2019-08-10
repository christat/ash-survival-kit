use std::error::Error;

extern crate ash;
use ash::{
    Device,
    extensions::{
        ext::DebugUtils,
        khr::Surface,
    },
    version::{DeviceV1_0, InstanceV1_0},
    vk, Entry, Instance,
};

extern crate winit;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod setup;
use setup::devices::{physical_devices, logical_devices};
pub mod utils;

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;

struct HelloTriangleApplication {
    entry: Entry,
    instance: Instance,
    device: Device,
    debug_utils: Option<DebugUtils>,
    debug_utils_messenger_ext: Option<vk::DebugUtilsMessengerEXT>,
    surface: Surface,
    surface_khr: vk::SurfaceKHR
}

impl HelloTriangleApplication {
    pub fn new(window: &Window, enable_validation_layers: bool) -> Self {
        let (entry, instance) = setup::init_vulkan(enable_validation_layers);
        let (debug_utils, debug_utils_messenger_ext) = match setup::init_debug_messenger(&entry, &instance, enable_validation_layers) {
            Some((debug_utils, debug_utils_messenger_ext)) => (Some(debug_utils), Some(debug_utils_messenger_ext)),
            None => (None, None)
        };
        let surface = Surface::new(&entry, &instance);
        let surface_khr = setup::init_surface_khr(&entry, &instance, window);
        let physical_device = physical_devices::select_physical_device(&instance, &surface, surface_khr);
        let (device, queue_family_indices) = logical_devices::create_logical_device(&instance, physical_device, &surface, surface_khr, enable_validation_layers);

        unsafe {
            let _graphics_queue = device.get_device_queue(queue_family_indices.graphics, 0);
            let _presentation_queue = device.get_device_queue(queue_family_indices.presentation, 0);
        }

        Self {
            entry,
            instance,
            debug_utils,
            debug_utils_messenger_ext,
            device,
            surface,
            surface_khr
        }
    }

    pub fn run(&mut self, event_loop: EventLoop<()>, window: Window) -> Result<(), Box<dyn Error>> {
        self.main_loop(event_loop, window);
        Ok(())
    }

    fn main_loop(&mut self, event_loop: EventLoop<()>, window: Window) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => *control_flow = ControlFlow::Wait,
        });
    }
}

impl Drop for HelloTriangleApplication {
    fn drop(&mut self) {
        if self.debug_utils.is_some() && self.debug_utils_messenger_ext.is_some() {
            unsafe {
                self.debug_utils.as_ref().unwrap()
                    .destroy_debug_utils_messenger(self.debug_utils_messenger_ext.unwrap(), None)
            };
        }

        unsafe {
            self.surface.destroy_surface(self.surface_khr, None);
            self.instance.destroy_instance(None);
        };
    }
}

fn main() {
    // TODO not too happy with this "borrow, then transfer" of event_loop/window; Find better way to interact with winit.
    let (event_loop, window) = setup::init_window();
    let mut app = HelloTriangleApplication::new(&window, ENABLE_VALIDATION_LAYERS);
    app.run(event_loop, window).expect("Application crashed!");
}
