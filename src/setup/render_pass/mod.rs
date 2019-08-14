use ash::{
    Device,
    vk,
    version::DeviceV1_0
};

use crate::setup::swapchain::SwapchainData;

pub fn create(device: &Device, swapchain_data: &SwapchainData) -> vk::RenderPass {
    let color_attachments = [vk::AttachmentDescription::builder()
        .format(swapchain_data.image_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build()
    ];

    let color_attachment_refs = [vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build()];

    let subpasses = [vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachment_refs)
        .build()];

    let render_pass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&color_attachments)
        .subpasses(&subpasses)
        .build();

    let render_pass = unsafe { device.create_render_pass(&render_pass_create_info, None).expect("Failed to create render pass!") };
    render_pass
}