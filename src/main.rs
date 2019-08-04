use std::{
    default::Default,
    error::Error,
    ffi::CString,
    fmt,
    ptr
};

extern crate ash;
use ash::{
    vk,
    Entry,
    Instance,
    version::{EntryV1_0, InstanceV1_0},
};

extern crate winit;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    //platform::desktop, TODO figure out how to pass to Vulkan context
    window::{Window, WindowBuilder},
};

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;

struct HelloTriangleApplication {}

impl HelloTriangleApplication {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run (&mut self) -> Result<(), Box<dyn Error>> {
        let (event_loop, window) = self.init_window();
        let instance= self.init_vulkan();

        self.main_loop(event_loop, window);
        self.cleanup(instance);
        Ok(())
    }

    fn init_window(&mut self) -> (EventLoop<()>, Window) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
            .with_title("Vulkan tutorial")
            .build(&event_loop);

        match window {
            Ok(window) => (event_loop, window),
            Err(e) => panic!("Failed to create window: {}", e)
        }
    }

    fn init_vulkan(&self) -> Instance {
        let instance_result = self.create_vk_instance();
        match instance_result {
            Ok((_entry, instance)) => instance,
            Err(e) => panic!("Failed to create Vulkan instance: {}", e)
        }
    }

    fn create_vk_instance(&self) -> Result<(Entry, Instance), Box<dyn Error>> {
        let application_name = CString::new("Hello triangle").unwrap();
        let engine_name = CString::new("No engine").unwrap();
        let version = ash::vk_make_version!(1, 0, 0);

        let application_info = vk::ApplicationInfo::builder()
            .application_name(&application_name)
            .application_version(version)
            .engine_name(&engine_name)
            .engine_version(version)
            .api_version(version)
            .build();

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .build();

        let entry= Entry::new()?;
        let instance = unsafe { entry.create_instance(&instance_create_info, None)? };
        Ok((entry, instance))
    }

    fn main_loop(&mut self, event_loop: EventLoop<()>, window: Window) {
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                _ => *control_flow = ControlFlow::Wait,
            }
        });
    }

    fn cleanup(&self, instance: Instance) {
        unsafe { instance.destroy_instance(None) };
    }
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    let status = app.run();
    match status {
        Ok(_) => (),
        Err(e) => panic!("Application crashed! Trace: {}", e)
    }
}
