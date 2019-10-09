use std::error::Error;

extern crate ash;
use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

extern crate cgmath;

extern crate field_offset;

extern crate winit;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::desktop::EventLoopExtDesktop,
    window::{Window, WindowBuilder},
};

mod setup;
use crate::setup::{
    swapchain::SwapchainData,
    graphics_pipeline::PipelineContainer,
    frame_sync::FrameSyncData
};

mod structs;
use structs::Vertex;

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;

pub const WINDOW_WIDTH: usize = 800;
pub const WINDOW_HEIGHT: usize = 600;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct VulkanApp {
    _entry: Entry,
    instance: Instance,
    physical_device: vk::PhysicalDevice,
    device: Device,
    debug_utils: Option<DebugUtils>,
    debug_utils_messenger_ext: Option<vk::DebugUtilsMessengerEXT>,
    surface: Surface,
    surface_khr: vk::SurfaceKHR,

    swapchain_data: SwapchainData,
    render_pass: vk::RenderPass,
    pipeline_container: PipelineContainer,
    framebuffers: Vec<vk::Framebuffer>,
    vertex_buffer: vk::Buffer,
    vertices: Vec<Vertex>,
    vertex_buffer_memory: vk::DeviceMemory,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    frame_sync_data: FrameSyncData,

    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

impl VulkanApp {
    pub fn new(window: &Window, vertices: Vec<Vertex>, enable_validation_layers: bool) -> Self {
        let (entry, instance) = setup::instance::create(enable_validation_layers);
        let (debug_utils, debug_utils_messenger_ext) =
            setup::validation_layers::initialize(&entry, &instance, enable_validation_layers);
        let surface = Surface::new(&entry, &instance);
        let surface_khr = setup::platform::surface_khr::create(&entry, &instance, window);
        let physical_device =
            setup::devices::physical::select(&instance, &surface, surface_khr);
        let (device, queue_family_indices) = setup::devices::logical::create(
            &instance,
            physical_device,
            &surface,
            surface_khr,
            enable_validation_layers,
        );

        let physical_window_size = window.outer_size().to_physical(window.current_monitor().hidpi_factor());
        let swapchain_data = SwapchainData::new(&instance, physical_device, &device, &surface, surface_khr, physical_window_size);
        let render_pass = setup::render_pass::create(&device, &swapchain_data);
        let command_pool = setup::command_pool::create(&device, &queue_family_indices);
        let pipeline_container = setup::graphics_pipeline::create(&device, &swapchain_data, render_pass);
        let graphics_pipeline = pipeline_container.pipelines.first().expect("Failed to fetch pipeline!");
        let framebuffers = setup::framebuffers::create(&device, &swapchain_data, render_pass);
        let (vertex_buffer, vertex_buffer_memory) = setup::vertex_buffer::create(&instance, &physical_device, &device, &vertices);
        let command_buffers = setup::command_buffers::create(&device, command_pool, &framebuffers, render_pass, swapchain_data.image_extent, graphics_pipeline, vertex_buffer, &vertices);

        let frame_sync_data = setup::frame_sync::create(&device, MAX_FRAMES_IN_FLIGHT);
        let graphics_queue = unsafe { device.get_device_queue(queue_family_indices.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(queue_family_indices.present, 0) };

        Self {
            _entry: entry,
            instance,
            debug_utils,
            debug_utils_messenger_ext,
            physical_device,
            device,
            surface,
            surface_khr,
            swapchain_data,
            render_pass,
            pipeline_container,
            framebuffers,
            vertex_buffer,
            vertex_buffer_memory,
            vertices,
            command_pool,
            command_buffers,
            frame_sync_data,
            graphics_queue,
            present_queue
        }
    }

    pub fn run(&mut self, event_loop: &mut EventLoop<()>, window: Window) -> Result<(), Box<dyn Error>> {
        let mut physical_window_size = window.outer_size().to_physical(window.current_monitor().hidpi_factor());
        let mut current_frame: usize = 0;
        let mut framebuffer_resized = false;
        event_loop.run_return(|event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(logical_size) => {
                        physical_window_size = logical_size.to_physical(window.current_monitor().hidpi_factor());
                        framebuffer_resized = true;
                        let (width, height): (u32, u32) = physical_window_size.into();
                        if width == 0 || height == 0 {
                            *control_flow = ControlFlow::Wait;
                        } else {
                            *control_flow = ControlFlow::Poll;
                        }
                    },
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => *control_flow = ControlFlow::Poll
                },
                _ => *control_flow = ControlFlow::Poll,
            }
            self.draw_frame(current_frame, &physical_window_size, &mut framebuffer_resized).expect("Failed to draw frame!");
            current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        });

        self.device_wait_idle();
        Ok(())
    }

    fn draw_frame(&mut self, current_frame: usize, physical_window_size: &winit::dpi::PhysicalSize, framebuffer_resized: &mut bool) -> Result<(), vk::Result> {
        let timeout = std::u64::MAX;
        let fences = [self.frame_sync_data.in_flight_fences[current_frame]];
        unsafe { self.device.wait_for_fences(&fences, true, timeout).expect("Failed to wait for fences!"); }

        let acquire_next_image_result = unsafe { self.swapchain_data.swapchain.acquire_next_image(self.swapchain_data.swapchain_khr, timeout, self.frame_sync_data.image_available_semaphores[current_frame], vk::Fence::null())  };
        let (image_index, _is_suboptimal) = match acquire_next_image_result {
            Ok((image_index, is_suboptimal)) => (image_index, is_suboptimal),
            Err(result) => match result {
                vk::Result::ERROR_OUT_OF_DATE_KHR => {
                    unsafe { self.recreate_swapchain(physical_window_size); }
                    return Ok(());
                },
                result => return Err(result)
            }
        };

        let wait_semaphores = [self.frame_sync_data.image_available_semaphores[current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [
            self.command_buffers[image_index as usize]
        ];
        let signal_semaphores = [self.frame_sync_data.render_finished_semaphores[current_frame]];

        let submit_infos = [
            vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores)
                .build()
        ];

        unsafe {
            self.device.reset_fences(&fences).expect("Failed to reset fences!");
            self.device.queue_submit(self.graphics_queue, &submit_infos, self.frame_sync_data.in_flight_fences[current_frame]).expect("Failed to submit draw command buffer!")
        };

        let swapchains = [self.swapchain_data.swapchain_khr];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .build();

        let queue_present_result = unsafe { self.swapchain_data.swapchain.queue_present(self.present_queue, &present_info) };
        let should_recreate_swapchain = match queue_present_result {
            Ok(is_suboptimal) => is_suboptimal,
            Err(result) => match result {
                vk::Result::ERROR_OUT_OF_DATE_KHR => true,
                result => return Err(result)
            }
        };
        if should_recreate_swapchain || *framebuffer_resized {
            *framebuffer_resized = false;
            unsafe { self.recreate_swapchain(physical_window_size); }
        }
        Ok(())
    }

    fn device_wait_idle(&self) {
        unsafe { self.device.device_wait_idle().expect("Failed to wait for device to become idle!"); }
    }

    unsafe fn recreate_swapchain(&mut self, physical_window_size: &winit::dpi::PhysicalSize) {
        self.device_wait_idle();
        self.drop_swapchain();

        self.swapchain_data = SwapchainData::new(&self.instance, self.physical_device, &self.device, &self.surface, self.surface_khr, *physical_window_size);
        self.render_pass = setup::render_pass::create(&self.device, &self.swapchain_data);
        self.pipeline_container = setup::graphics_pipeline::create(&self.device, &self.swapchain_data, self.render_pass);
        self.framebuffers = setup::framebuffers::create(&self.device, &self.swapchain_data, self.render_pass);
        let graphics_pipeline = self.pipeline_container.pipelines.first().expect("Failed to fetch pipeline!");
        self.command_buffers = setup::command_buffers::create(&self.device, self.command_pool, &self.framebuffers, self.render_pass, self.swapchain_data.image_extent, graphics_pipeline, self.vertex_buffer, &self.vertices);
    }

    unsafe fn drop_swapchain(&self) {
        self.framebuffers.iter().for_each(|framebuffer| self.device.destroy_framebuffer(*framebuffer, None));
        self.device.free_command_buffers(self.command_pool, &self.command_buffers);
        self.pipeline_container.pipelines.iter().for_each(|pipeline| self.device.destroy_pipeline(*pipeline, None));
        self.device.destroy_pipeline_layout(self.pipeline_container.pipeline_layout, None);
        self.device.destroy_render_pass(self.render_pass, None);

        self.swapchain_data.swapchain_image_views.iter().for_each(|view| self.device.destroy_image_view(*view, None));
        self.swapchain_data.swapchain.destroy_swapchain(self.swapchain_data.swapchain_khr, None);
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.drop_swapchain();

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.frame_sync_data.image_available_semaphores.iter().for_each(|semaphore| self.device.destroy_semaphore(*semaphore, None));
            self.frame_sync_data.render_finished_semaphores.iter().for_each(|semaphore| self.device.destroy_semaphore(*semaphore, None));
            self.frame_sync_data.in_flight_fences.iter().for_each(|fence| self.device.destroy_fence(*fence, None));

            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);

            if self.debug_utils.is_some() && self.debug_utils_messenger_ext.is_some() {
                self.debug_utils
                    .as_ref()
                    .unwrap()
                    .destroy_debug_utils_messenger(self.debug_utils_messenger_ext.unwrap(), None)
            }

            self.surface.destroy_surface(self.surface_khr, None);
            self.instance.destroy_instance(None);
        };
    }
}

fn main() {
    let vertices: Vec<Vertex> = vec![
        Vertex::new(0.0, -0.5, 1.0, 1.0, 1.0),
        Vertex::new(0.5, 0.5, 0.0, 1.0, 0.0),
        Vertex::new(-0.5, 0.5, 0.0, 0.0, 1.0)
    ];

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(
            WINDOW_WIDTH as f64,
            WINDOW_HEIGHT as f64,
        ))
        .with_title("Vulkan tutorial")
        .build(&event_loop)
        .expect("Failed to create window!");

    let mut app = VulkanApp::new(&window, vertices,  ENABLE_VALIDATION_LAYERS);
    app.run(&mut event_loop, window).expect("Application crashed!");
}
