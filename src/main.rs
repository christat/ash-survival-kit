use std::error::Error;

extern crate ash;
use ash::{
    Device,
    extensions::{
        ext::DebugUtils,
    },
    version::{DeviceV1_0, InstanceV1_0},
    vk, Entry, Instance,
};

extern crate winit;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    //platform::desktop, TODO figure out how to pass to Vulkan context
    window::{Window, WindowBuilder},
};

mod setup;
use setup::devices::{physical_devices, logical_devices};
pub mod utils;

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;

pub const WINDOW_WIDTH: usize = 800;
pub const WINDOW_HEIGHT: usize = 600;

struct HelloTriangleApplication {
    entry: Entry,
    instance: Instance,
    device: Device,
    debug_utils: Option<DebugUtils>,
    debug_utils_messenger_ext: Option<vk::DebugUtilsMessengerEXT>,
}

impl HelloTriangleApplication {
    pub fn new(enable_validation_layers: bool) -> Self {
        let (entry, instance) = setup::init_vulkan(enable_validation_layers);
        let (debug_utils, debug_utils_messenger_ext) = match setup::init_debug_messenger(&entry, &instance, enable_validation_layers) {
            Some((debug_utils, debug_utils_messenger_ext)) => (Some(debug_utils), Some(debug_utils_messenger_ext)),
            None => (None, None)
        };

        let physical_device = physical_devices::select_physical_device(&instance);
        let (device, queue_family_index) = logical_devices::create_logical_device(&instance, physical_device, enable_validation_layers);

        let _graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Self {
            entry,
            instance,
            debug_utils,
            debug_utils_messenger_ext,
            device
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.main_loop();
        Ok(())
    }

    fn main_loop(&mut self) {
        let (event_loop, window) = Self::init_window();
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => *control_flow = ControlFlow::Wait,
        });
    }

    fn init_window() -> (EventLoop<()>, Window) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
            .with_title("Vulkan tutorial")
            .build(&event_loop);

        match window {
            Ok(window) => (event_loop, window),
            Err(e) => panic!("Failed to create window: {}", e),
        }
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

        unsafe { self.instance.destroy_instance(None) };
    }
}

fn main() {
    let mut app = HelloTriangleApplication::new(ENABLE_VALIDATION_LAYERS);
    app.run().expect("Application crashed!");
}
