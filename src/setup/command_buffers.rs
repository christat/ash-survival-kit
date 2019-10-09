use ash::{
    Device,
    vk,
    version::DeviceV1_0
};

use crate::structs::Vertex;

pub fn create(device: &Device, command_pool: vk::CommandPool, framebuffers: &[vk::Framebuffer], render_pass: vk::RenderPass, swapchain_extent: vk::Extent2D, pipeline: &vk::Pipeline, vertex_buffer: vk::Buffer, vertices: &[Vertex]) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(framebuffers.len() as u32)
        .build();

    let command_buffers = unsafe { device.allocate_command_buffers(&command_buffer_allocate_info).expect("Failed to allocate command buffers!") };
    let command_buffers = command_buffers.into_iter().zip(framebuffers).map(|(command_buffer, framebuffer)| {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().build();

        unsafe { device.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("Failed to begin recording command buffer!") };

        let render_area = vk::Rect2D::builder()
            .offset(
                vk::Offset2D::builder()
                    .x(0)
                    .y(0)
                    .build()
            )
            .extent(swapchain_extent)
            .build();

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0]
                }
            }
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(*framebuffer)
            .render_area(render_area)
            .clear_values(&clear_values)
            .build();

        unsafe {
            device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, *pipeline);
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            device.cmd_draw(command_buffer, vertices.len() as u32, 1, 0, 0);
            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer).expect("Failed to record command buffer!");
        };
        command_buffer
    }).collect::<Vec<vk::CommandBuffer>>();

    command_buffers
}