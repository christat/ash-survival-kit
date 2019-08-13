use std::error::Error;

extern crate ash;
use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

extern crate winit;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod setup;
pub mod utils;
use crate::setup::{
    swapchain::SwapchainData,
    graphics_pipeline::Pipeline,
};

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
    surface: Surface,
    surface_khr: vk::SurfaceKHR,
    swapchain_data: SwapchainData,
    pipeline: Pipeline
}

impl HelloTriangleApplication {
    pub fn new(window: &Window, enable_validation_layers: bool) -> Self {
        let (entry, instance) = setup::init_vulkan(enable_validation_layers);
        let (debug_utils, debug_utils_messenger_ext) =
            setup::init_debug_messenger(&entry, &instance, enable_validation_layers);
        let surface = Surface::new(&entry, &instance);
        let surface_khr = setup::init_surface_khr(&entry, &instance, window);
        let physical_device =
            setup::devices::physical::select_physical_device(&instance, &surface, surface_khr);
        let (device, queue_family_indices) = setup::devices::logical::create_logical_device(
            &instance,
            physical_device,
            &surface,
            surface_khr,
            enable_validation_layers,
        );
        let swapchain_data =
            setup::swapchain::create(&instance, physical_device, &device, &surface, surface_khr);
        let pipeline = setup::graphics_pipeline::create(&device, &swapchain_data);

        unsafe {
            let _graphics_queue = device.get_device_queue(queue_family_indices.graphics, 0);
            let _presentation_queue = device.get_device_queue(queue_family_indices.present, 0);
        }

        Self {
            entry,
            instance,
            debug_utils,
            debug_utils_messenger_ext,
            device,
            surface,
            surface_khr,
            swapchain_data,
            pipeline
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
                self.debug_utils
                    .as_ref()
                    .unwrap()
                    .destroy_debug_utils_messenger(self.debug_utils_messenger_ext.unwrap(), None)
            };
        }

        unsafe {
            self.device.destroy_pipeline_layout(self.pipeline.pipeline_layout, None);
            self.pipeline.shader_modules.iter().for_each(|module| self.device.destroy_shader_module(*module, None));
            self.swapchain_data
                .swapchain_image_views
                .iter()
                .for_each(|image_view| self.device.destroy_image_view(*image_view, None));
            self.swapchain_data
                .swapchain
                .destroy_swapchain(self.swapchain_data.swapchain_khr, None);
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
