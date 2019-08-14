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
    platform::desktop::EventLoopExtDesktop,
    window::Window,
};

mod setup;
pub mod utils;
use crate::setup::{
    swapchain::SwapchainData,
    graphics_pipeline::Pipeline,
    frame_sync::FrameSyncData
};

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;

pub const WINDOW_WIDTH: usize = 800;
pub const WINDOW_HEIGHT: usize = 600;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct HelloTriangleApplication {
    entry: Entry,
    instance: Instance,
    device: Device,
    debug_utils: Option<DebugUtils>,
    debug_utils_messenger_ext: Option<vk::DebugUtilsMessengerEXT>,
    surface: Surface,
    surface_khr: vk::SurfaceKHR,
    swapchain_data: SwapchainData,
    render_pass: vk::RenderPass,
    pipeline: Pipeline,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    frame_sync_data: FrameSyncData,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue
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
        let render_pass = setup::render_pass::create(&device, &swapchain_data);
        let pipeline = setup::graphics_pipeline::create(&device, &swapchain_data, render_pass);
        let framebuffers = setup::framebuffers::create(&device, &swapchain_data, render_pass);
        let command_pool = setup::command_pool::create(&device, &queue_family_indices);
        let graphics_pipeline = pipeline.pipelines.first().expect("Failed to fetch pipeline!");
        let command_buffers = setup::command_buffers::create(&device, command_pool, &framebuffers, render_pass, swapchain_data.image_extent, graphics_pipeline);
        let frame_sync_data = setup::frame_sync::create(&device, MAX_FRAMES_IN_FLIGHT);

        let graphics_queue = unsafe { device.get_device_queue(queue_family_indices.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(queue_family_indices.present, 0) };

        Self {
            entry,
            instance,
            debug_utils,
            debug_utils_messenger_ext,
            device,
            surface,
            surface_khr,
            swapchain_data,
            render_pass,
            pipeline,
            framebuffers,
            command_pool,
            command_buffers,
            frame_sync_data,
            graphics_queue,
            present_queue
        }
    }

    pub fn run(&self, event_loop: &mut EventLoop<()>, window: Window) -> Result<(), Box<dyn Error>> {
        self.main_loop(event_loop, window);
        Ok(())
    }

    fn main_loop(&self, event_loop: &mut EventLoop<()>, window: Window) {
        let mut current_frame: usize = 0;
        event_loop.run_return(|event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                _ => *control_flow = ControlFlow::Wait,
            }
            self.draw_frame(current_frame);
            current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        });

        unsafe { self.device.device_wait_idle().expect("Failed to wait for logical device to finish operations!") };
    }

    fn draw_frame(&self, current_frame: usize) {
        let timeout = std::u64::MAX;
        unsafe {
            let fences = [self.frame_sync_data.in_flight_fences[current_frame]];
            self.device.wait_for_fences(&fences, true, timeout).expect("Failed to wait for fences!");
            self.device.reset_fences(&fences).expect("Failed to reset fences!");
        }

        let (image_index, _is_suboptimal) = unsafe { self.swapchain_data.swapchain.acquire_next_image(self.swapchain_data.swapchain_khr, timeout, self.frame_sync_data.image_available_semaphores[current_frame], vk::Fence::null()).expect("Failed to acquire next image!")  };

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

        unsafe { self.device.queue_submit(self.graphics_queue, &submit_infos, self.frame_sync_data.in_flight_fences[current_frame]).expect("Failed to submit draw command buffer!") };

        let swapchains = [self.swapchain_data.swapchain_khr];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .build();

        unsafe { self.swapchain_data.swapchain.queue_present(self.present_queue, &present_info).expect("Failed to queue image to presentation!") };
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
            self.frame_sync_data.image_available_semaphores.iter().for_each(|semaphore| self.device.destroy_semaphore(*semaphore, None));
            self.frame_sync_data.render_finished_semaphores.iter().for_each(|semaphore| self.device.destroy_semaphore(*semaphore, None));
            self.frame_sync_data.in_flight_fences.iter().for_each(|fence| self.device.destroy_fence(*fence, None));
            self.device.destroy_command_pool(self.command_pool, None);
            self.framebuffers.iter().for_each(|framebuffer| self.device.destroy_framebuffer(*framebuffer, None));
            self.pipeline.pipelines.iter().for_each(|pipeline| self.device.destroy_pipeline(*pipeline, None));
            self.device.destroy_pipeline_layout(self.pipeline.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.pipeline.shader_modules.iter().for_each(|module| self.device.destroy_shader_module(*module, None));
            self.swapchain_data
                .swapchain_image_views
                .iter()
                .for_each(|image_view| self.device.destroy_image_view(*image_view, None));
            self.swapchain_data
                .swapchain
                .destroy_swapchain(self.swapchain_data.swapchain_khr, None);
            self.device.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);
            self.instance.destroy_instance(None);
        };
    }
}

fn main() {
    // TODO not too happy with this "borrow, then transfer" of event_loop/window; Find better way to interact with winit.
    let (mut event_loop, window) = setup::init_window();
    let app = HelloTriangleApplication::new(&window, ENABLE_VALIDATION_LAYERS);
    app.run(&mut event_loop, window).expect("Application crashed!");
}
