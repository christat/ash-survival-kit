use ash::{version::DeviceV1_0, vk, Device};

use crate::setup::swapchain::SwapchainData;

pub fn create(
    device: &Device,
    swapchain_data: &SwapchainData,
    render_pass: vk::RenderPass,
    color_image_view: &vk::ImageView,
    depth_image_view: &vk::ImageView,
) -> Vec<vk::Framebuffer> {
    swapchain_data.swapchain_image_views.iter().fold(
        Vec::with_capacity(swapchain_data.swapchain_image_views.len()),
        |mut acc, image_view| {
            let attachments = vec![*color_image_view, *depth_image_view, *image_view];
            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain_data.image_extent.width)
                .height(swapchain_data.image_extent.height)
                .layers(1);

            let framebuffer = unsafe {
                device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create framebuffer!")
            };
            acc.push(framebuffer);
            acc
        },
    )
}
