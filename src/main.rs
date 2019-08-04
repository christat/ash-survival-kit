use std::{error::Error, fmt};

// Cross-platform window management crate; handles event loop and provides raw window handle for Vulkan.
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

    pub fn run (&mut self) -> Result<u8, RunError> {
        let (event_loop, window) = self.init_window();
        self.init_vulkan();
        self.main_loop(event_loop, window);
        self.cleanup();
        Err(RunError)
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

    fn init_vulkan(&self) {

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

    fn cleanup(&self) {

    }
}

#[derive(Debug, PartialEq)]
pub struct RunError;

impl Error for RunError{}

impl fmt::Display for RunError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Something went horribly wrong! :scream_cat:")
    }
}


fn main() {
    let mut app = HelloTriangleApplication::new();
    let status = app.run();
    match status {
        Ok(code) => println!("Sall good man, code was {}", code),
        Err(e) => println!("Error: {}", e)
    }
}
