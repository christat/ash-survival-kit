use std::{error::Error, mem::size_of, ptr::copy_nonoverlapping, time::Instant};

extern crate ash;
use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

extern crate cgmath;
use cgmath::{Deg, Matrix4, Point3, Vector3};

extern crate field_offset;

extern crate winapi;
extern crate winit;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::desktop::EventLoopExtDesktop,
    window::{Window, WindowBuilder},
};

mod setup;
use crate::setup::{frame_sync::FrameSyncData, swapchain::SwapchainData};

mod structs;
use structs::{Vertex, UBO};

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

    pipelines: Vec<vk::Pipeline>,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,

    framebuffers: Vec<vk::Framebuffer>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    _vertices: Vec<Vertex>,

    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    indices: Vec<u32>,

    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    frame_sync_data: FrameSyncData,

    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    texture_image: vk::Image,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    texture_image_memory: vk::DeviceMemory,
    _texture_image_mip_levels: u32,

    color_image: vk::Image,
    color_image_view: vk::ImageView,
    color_image_memory: vk::DeviceMemory,

    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,

    msaa_samples: vk::SampleCountFlags,
}

impl VulkanApp {
    pub fn new(window: &Window, enable_validation_layers: bool) -> Self {
        let (entry, instance) = setup::instance::create(enable_validation_layers);
        let (debug_utils, debug_utils_messenger_ext) =
            setup::validation_layers::initialize(&entry, &instance, enable_validation_layers);
        let surface = Surface::new(&entry, &instance);
        let surface_khr = setup::platform::surface_khr::create(&entry, &instance, window);
        let (physical_device, msaa_samples) =
            setup::devices::physical::select(&instance, &surface, surface_khr);
        let (device, queue_family_indices) = setup::devices::logical::create(
            &instance,
            physical_device,
            &surface,
            surface_khr,
            enable_validation_layers,
        );

        let swapchain_data = SwapchainData::new(
            &instance,
            physical_device,
            &device,
            &surface,
            surface_khr,
            window.outer_size(),
        );
        let render_pass = setup::render_pass::create(
            &instance,
            &device,
            &physical_device,
            &swapchain_data,
            msaa_samples,
        );
        let command_pool = setup::command_pool::create(&device, &queue_family_indices);

        let descriptor_set_layout = setup::uniform_buffers::create_descriptor_set_layout(&device);
        let (pipelines, pipeline_layout) = setup::graphics_pipeline::create(
            &device,
            &swapchain_data,
            render_pass,
            &descriptor_set_layout,
            msaa_samples,
        );
        let graphics_pipeline = pipelines.first().expect("Failed to fetch pipeline!");
        let graphics_queue = unsafe { device.get_device_queue(queue_family_indices.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(queue_family_indices.present, 0) };

        let (color_image, color_image_view, color_image_memory) =
            setup::image::create_color_resources(
                &instance,
                &device,
                &physical_device,
                swapchain_data.image_extent,
                swapchain_data.image_format,
                msaa_samples,
            );

        let (depth_image, depth_image_view, depth_image_memory) =
            setup::image::create_depth_resources(
                &instance,
                &device,
                &physical_device,
                swapchain_data.image_extent,
                msaa_samples,
            );

        let framebuffers = setup::framebuffers::create(
            &device,
            &swapchain_data,
            render_pass,
            &color_image_view,
            &depth_image_view,
        );

        let (texture_image, texture_image_memory, texture_image_mip_levels) = setup::image::create(
            &instance,
            &device,
            &physical_device,
            command_pool,
            graphics_queue,
        );
        let texture_image_view = setup::image::create_texture_image_view(
            &device,
            texture_image,
            texture_image_mip_levels,
        );
        let texture_sampler =
            setup::image::create_texture_sampler(&device, texture_image_mip_levels);
        let (vertices, indices) = setup::model::load();
        let (vertex_buffer, vertex_buffer_memory) = setup::vertex_buffer::create(
            &instance,
            &physical_device,
            &device,
            command_pool,
            graphics_queue,
            &vertices,
        );
        let (index_buffer, index_buffer_memory) = setup::index_buffer::create(
            &instance,
            &physical_device,
            &device,
            command_pool,
            graphics_queue,
            &indices,
        );

        let (uniform_buffers, uniform_buffers_memory) = setup::uniform_buffers::create(
            &instance,
            &device,
            &physical_device,
            &swapchain_data.swapchain_images,
        );
        let descriptor_pool = setup::uniform_buffers::create_descriptor_pool(
            &device,
            &swapchain_data.swapchain_images,
        );
        let descriptor_sets = setup::uniform_buffers::create_descriptor_sets(
            &device,
            descriptor_pool,
            descriptor_set_layout,
            &uniform_buffers,
            &swapchain_data.swapchain_images,
            texture_image_view,
            texture_sampler,
        );

        let command_buffers = setup::command_buffers::create(
            &device,
            command_pool,
            &framebuffers,
            render_pass,
            swapchain_data.image_extent,
            graphics_pipeline,
            pipeline_layout,
            &descriptor_sets,
            vertex_buffer,
            index_buffer,
            &indices,
        );

        let frame_sync_data = setup::frame_sync::create(&device, MAX_FRAMES_IN_FLIGHT);

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
            pipelines,
            pipeline_layout,
            descriptor_set_layout,
            framebuffers,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            uniform_buffers,
            uniform_buffers_memory,
            descriptor_pool,
            descriptor_sets,
            command_pool,
            command_buffers,
            frame_sync_data,
            graphics_queue,
            present_queue,
            _vertices: vertices,
            indices,
            texture_image,
            texture_image_view,
            texture_sampler,
            texture_image_memory,
            color_image,
            color_image_view,
            color_image_memory,
            depth_image,
            depth_image_view,
            depth_image_memory,
            _texture_image_mip_levels: texture_image_mip_levels,
            msaa_samples,
        }
    }

    pub fn run(
        &mut self,
        event_loop: &mut EventLoop<()>,
        window: Window,
    ) -> Result<(), Box<dyn Error>> {
        let physical_window_size = window.outer_size();
        let mut current_frame: usize = 0;
        let mut framebuffer_resized = false;
        let init_stamp = Instant::now();

        event_loop.run_return(|event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(physical_window_size) => {
                        framebuffer_resized = true;
                        let (width, height): (u32, u32) = physical_window_size.into();
                        if width == 0 || height == 0 {
                            *control_flow = ControlFlow::Wait;
                        } else {
                            *control_flow = ControlFlow::Poll;
                        }
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => *control_flow = ControlFlow::Poll,
                },
                _ => *control_flow = ControlFlow::Poll,
            }
            self.draw_frame(
                current_frame,
                &physical_window_size,
                &mut framebuffer_resized,
                &init_stamp,
            )
            .expect("Failed to draw frame!");
            current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        });

        self.device_wait_idle();
        Ok(())
    }

    fn draw_frame(
        &mut self,
        current_frame: usize,
        physical_window_size: &winit::dpi::PhysicalSize<u32>,
        framebuffer_resized: &mut bool,
        init_timestamp: &Instant,
    ) -> Result<(), vk::Result> {
        let timeout = std::u64::MAX;
        let fences = [self.frame_sync_data.in_flight_fences[current_frame]];
        unsafe {
            self.device
                .wait_for_fences(&fences, true, timeout)
                .expect("Failed to wait for fences!");
        }

        let acquire_next_image_result = unsafe {
            self.swapchain_data.swapchain.acquire_next_image(
                self.swapchain_data.swapchain_khr,
                timeout,
                self.frame_sync_data.image_available_semaphores[current_frame],
                vk::Fence::null(),
            )
        };
        let (image_index, _is_suboptimal) = match acquire_next_image_result {
            Ok((image_index, is_suboptimal)) => (image_index, is_suboptimal),
            Err(result) => match result {
                vk::Result::ERROR_OUT_OF_DATE_KHR => {
                    unsafe {
                        self.recreate_swapchain(physical_window_size);
                    }
                    return Ok(());
                }
                result => return Err(result),
            },
        };

        let wait_semaphores = [self.frame_sync_data.image_available_semaphores[current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.command_buffers[image_index as usize]];
        let signal_semaphores = [self.frame_sync_data.render_finished_semaphores[current_frame]];

        self.update_uniform_buffer(image_index, init_timestamp);

        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build()];

        unsafe {
            self.device
                .reset_fences(&fences)
                .expect("Failed to reset fences!");
            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.frame_sync_data.in_flight_fences[current_frame],
                )
                .expect("Failed to submit draw command buffer!")
        };

        let swapchains = [self.swapchain_data.swapchain_khr];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .build();

        let queue_present_result = unsafe {
            self.swapchain_data
                .swapchain
                .queue_present(self.present_queue, &present_info)
        };
        let should_recreate_swapchain = match queue_present_result {
            Ok(is_suboptimal) => is_suboptimal,
            Err(result) => match result {
                vk::Result::ERROR_OUT_OF_DATE_KHR => true,
                result => return Err(result),
            },
        };
        if should_recreate_swapchain || *framebuffer_resized {
            *framebuffer_resized = false;
            unsafe {
                self.recreate_swapchain(physical_window_size);
            }
        }
        Ok(())
    }

    fn update_uniform_buffer(&self, image_index: u32, init_timestamp: &Instant) {
        let elapsed_seconds = init_timestamp.elapsed().as_secs_f32();

        let ubo = UBO {
            model: Matrix4::from_angle_z(Deg(30.0 * elapsed_seconds)),
            view: Matrix4::look_at(
                Point3::new(2.0, 2.0, 1.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ),
            projection: self.build_projection_matrix(45.0, 1.0, 10.0),
        };

        unsafe {
            let data_ptr = self
                .device
                .map_memory(
                    self.uniform_buffers_memory[image_index as usize],
                    0,
                    size_of::<UBO>() as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map uniform buffer memory!");
            copy_nonoverlapping(&ubo, data_ptr as *mut UBO, 1);
            self.device
                .unmap_memory(self.uniform_buffers_memory[image_index as usize]);
        };
    }

    fn build_projection_matrix(&self, fov: f32, f_near: f32, f_far: f32) -> Matrix4<f32> {
        let vk::Extent2D { width, height } = self.swapchain_data.image_extent;
        let aspect_ratio = width as f32 / height as f32;

        let scale = 1.0 / (0.5 * fov * std::f32::consts::PI / 180.0).tan();
        let c0r0 = scale / aspect_ratio;
        let c2r2 = -f_far / (f_far - f_near);
        let c3r2 = -(f_far * f_near) / (f_far - f_near);

        // perspective projection borrowed from:
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/perspective-and-orthographic-projection-matrix/building-basic-perspective-projection-matrix
        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::new(
            c0r0,    0.0,    0.0,      0.0,
             0.0, -scale,    0.0,      0.0,
             0.0,    0.0,   c2r2,     c3r2,
             0.0,    0.0,   -1.0,      0.0
        )
    }

    fn device_wait_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for device to become idle!");
        }
    }

    unsafe fn recreate_swapchain(&mut self, physical_window_size: &winit::dpi::PhysicalSize<u32>) {
        self.device_wait_idle();
        self.drop_swapchain();

        self.swapchain_data = SwapchainData::new(
            &self.instance,
            self.physical_device,
            &self.device,
            &self.surface,
            self.surface_khr,
            *physical_window_size,
        );
        self.render_pass = setup::render_pass::create(
            &self.instance,
            &self.device,
            &self.physical_device,
            &self.swapchain_data,
            self.msaa_samples,
        );
        let (pipelines, pipeline_layout) = setup::graphics_pipeline::create(
            &self.device,
            &self.swapchain_data,
            self.render_pass,
            &self.descriptor_set_layout,
            self.msaa_samples,
        );
        self.pipelines = pipelines;
        self.pipeline_layout = pipeline_layout;
        let graphics_pipeline = self.pipelines.first().expect("Failed to fetch pipeline!");

        let (color_image, color_image_view, color_image_memory) =
            setup::image::create_color_resources(
                &self.instance,
                &self.device,
                &self.physical_device,
                self.swapchain_data.image_extent,
                self.swapchain_data.image_format,
                self.msaa_samples,
            );
        self.color_image = color_image;
        self.color_image_view = color_image_view;
        self.color_image_memory = color_image_memory;

        let (depth_image, depth_image_view, depth_image_memory) =
            setup::image::create_depth_resources(
                &self.instance,
                &self.device,
                &self.physical_device,
                self.swapchain_data.image_extent,
                self.msaa_samples,
            );
        self.depth_image = depth_image;
        self.depth_image_view = depth_image_view;
        self.depth_image_memory = depth_image_memory;

        self.framebuffers = setup::framebuffers::create(
            &self.device,
            &self.swapchain_data,
            self.render_pass,
            &self.color_image_view,
            &self.depth_image_view,
        );

        let (uniform_buffers, uniform_buffers_memory) = setup::uniform_buffers::create(
            &self.instance,
            &self.device,
            &self.physical_device,
            &self.swapchain_data.swapchain_images,
        );
        self.uniform_buffers = uniform_buffers;
        self.uniform_buffers_memory = uniform_buffers_memory;

        self.descriptor_pool = setup::uniform_buffers::create_descriptor_pool(
            &self.device,
            &self.swapchain_data.swapchain_images,
        );
        self.descriptor_sets = setup::uniform_buffers::create_descriptor_sets(
            &self.device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            &self.uniform_buffers,
            &self.swapchain_data.swapchain_images,
            self.texture_image_view,
            self.texture_sampler,
        );

        self.command_buffers = setup::command_buffers::create(
            &self.device,
            self.command_pool,
            &self.framebuffers,
            self.render_pass,
            self.swapchain_data.image_extent,
            graphics_pipeline,
            self.pipeline_layout,
            &self.descriptor_sets,
            self.vertex_buffer,
            self.index_buffer,
            &self.indices,
        );
    }

    unsafe fn drop_swapchain(&self) {
        self.device.destroy_image_view(self.color_image_view, None);
        self.device.destroy_image(self.color_image, None);
        self.device.free_memory(self.color_image_memory, None);
        self.device.destroy_image_view(self.depth_image_view, None);
        self.device.destroy_image(self.depth_image, None);
        self.device.free_memory(self.depth_image_memory, None);
        self.framebuffers
            .iter()
            .for_each(|framebuffer| self.device.destroy_framebuffer(*framebuffer, None));
        self.device
            .free_command_buffers(self.command_pool, &self.command_buffers);
        self.pipelines
            .iter()
            .for_each(|pipeline| self.device.destroy_pipeline(*pipeline, None));
        self.device
            .destroy_pipeline_layout(self.pipeline_layout, None);
        self.device.destroy_render_pass(self.render_pass, None);

        self.swapchain_data
            .swapchain_image_views
            .iter()
            .for_each(|view| self.device.destroy_image_view(*view, None));
        self.swapchain_data
            .swapchain
            .destroy_swapchain(self.swapchain_data.swapchain_khr, None);

        self.device
            .destroy_descriptor_pool(self.descriptor_pool, None);
        self.uniform_buffers
            .iter()
            .zip(&self.uniform_buffers_memory)
            .for_each(|(buffer, memory)| {
                self.device.destroy_buffer(*buffer, None);
                self.device.free_memory(*memory, None);
            });
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.drop_swapchain();

            self.device.destroy_sampler(self.texture_sampler, None);
            self.device
                .destroy_image_view(self.texture_image_view, None);
            self.device.destroy_image(self.texture_image, None);
            self.device.free_memory(self.texture_image_memory, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);
            self.frame_sync_data
                .image_available_semaphores
                .iter()
                .for_each(|semaphore| self.device.destroy_semaphore(*semaphore, None));
            self.frame_sync_data
                .render_finished_semaphores
                .iter()
                .for_each(|semaphore| self.device.destroy_semaphore(*semaphore, None));
            self.frame_sync_data
                .in_flight_fences
                .iter()
                .for_each(|fence| self.device.destroy_fence(*fence, None));

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
    // let vertices: Vec<Vertex> = vec![
    //     // quad 0
    //     Vertex::new(-0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0),
    //     Vertex::new(0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0),
    //     Vertex::new(0.5, 0.5, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
    //     Vertex::new(-0.5, 0.5, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0),
    //     // quad 1
    //     Vertex::new(-0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 0.0),
    //     Vertex::new(0.5, -0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 0.0),
    //     Vertex::new(0.5, 0.5, -0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
    //     Vertex::new(-0.5, 0.5, -0.5, 1.0, 1.0, 1.0, 0.0, 1.0),
    // ];
    //
    // let indices: Vec<u32> = vec![
    //     0, 1, 2, 2, 3, 0, // quad 0
    //     4, 5, 6, 6, 7, 4, // quad 1
    // ];

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
        .with_title("Vulkan tutorial")
        .build(&event_loop)
        .expect("Failed to create window!");

    let mut app = VulkanApp::new(&window, ENABLE_VALIDATION_LAYERS);
    app.run(&mut event_loop, window)
        .expect("Application crashed!");
}
