use std::ptr::copy_nonoverlapping;

use ash::{version::DeviceV1_0, vk, Device, Instance};
use image::GenericImageView;

use crate::setup::buffer;

pub fn create(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Image, vk::DeviceMemory) {
    let src = image::open("src/resources/img/rmd_logo.jpg").unwrap();
    let (width, height) = src.dimensions();
    let image_size = (std::mem::size_of::<u8>() as u32 * width * height * 4) as vk::DeviceSize;
    let src_bytes = src.to_bytes();

    if image_size == 0 {
        panic!("Failed to load texture image!")
    }

    let (staging_buffer, staging_memory) = buffer::create(
        instance,
        device,
        physical_device,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let data_ptr = device
            .map_memory(staging_memory, 0, image_size, vk::MemoryMapFlags::empty())
            .expect("Failed to map texture buffer memory!");
        copy_nonoverlapping(src_bytes.as_ptr(), data_ptr as *mut u8, src_bytes.len());
        device.unmap_memory(staging_memory);
    };

    let (texture_image, texture_image_memory) = create_image(
        instance,
        device,
        physical_device,
        width,
        height,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    transition_image_layout(
        device,
        command_pool,
        queue,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );

    copy_buffer_to_image(
        device,
        command_pool,
        queue,
        staging_buffer,
        texture_image,
        width,
        height,
    );

    transition_image_layout(
        device,
        command_pool,
        queue,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_memory, None);
    }

    (texture_image, texture_image_memory)
}

fn create_image(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(
            vk::Extent3D::builder()
                .width(width)
                .height(height)
                .depth(1)
                .build(),
        )
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1)
        .build();

    let image = unsafe {
        device
            .create_image(&image_create_info, None)
            .expect("Failed to create image!")
    };

    let memory_requirements = unsafe { device.get_image_memory_requirements(image) };

    let memory_type_index = buffer::find_memory_type_index(
        instance,
        physical_device,
        memory_requirements.memory_type_bits,
        properties,
    )
    .expect("Failed to find image memory type index!");

    let memory_allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(memory_requirements.size)
        .memory_type_index(memory_type_index)
        .build();

    let texture_image_memory = unsafe {
        device
            .allocate_memory(&memory_allocate_info, None)
            .expect("Failed to allocate image memory!")
    };

    unsafe {
        device
            .bind_image_memory(image, texture_image_memory, 0)
            .expect("Failed to bind image memory!");
    }
    (image, texture_image_memory)
}

pub fn create_texture_image_view(device: &Device, image: vk::Image) -> vk::ImageView {
    create_image_view(device, image, vk::Format::R8G8B8A8_SRGB)
}

pub fn create_texture_sampler(device: &Device) -> vk::Sampler {
    let create_info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(0.0)
        .build();

    unsafe {
        device
            .create_sampler(&create_info, None)
            .expect("Failed to create texture sampler!")
    }
}

struct LayoutTransitionMasks {
    pub src_access_mask: vk::AccessFlags,
    pub dst_access_mask: vk::AccessFlags,
    pub src_stage_mask: vk::PipelineStageFlags,
    pub dst_stage_mask: vk::PipelineStageFlags,
}

fn transition_image_layout(
    device: &Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    image: vk::Image,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    let command_buffer = buffer::begin_single_time_commands(device, command_pool);

    let masks: LayoutTransitionMasks = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => {
            LayoutTransitionMasks {
                src_access_mask: vk::AccessFlags::default(),
                dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                src_stage_mask: vk::PipelineStageFlags::TOP_OF_PIPE,
                dst_stage_mask: vk::PipelineStageFlags::TRANSFER,
            }
        }
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => {
            LayoutTransitionMasks {
                src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                dst_access_mask: vk::AccessFlags::SHADER_READ,
                src_stage_mask: vk::PipelineStageFlags::TRANSFER,
                dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            }
        }
        (_, _) => panic!("Unsupported layout transition!"),
    };

    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        )
        .src_access_mask(masks.src_access_mask)
        .dst_access_mask(masks.dst_access_mask)
        .build();

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            masks.src_stage_mask,
            masks.dst_stage_mask,
            vk::DependencyFlags::BY_REGION,
            &[],
            &[],
            &[barrier],
        )
    };

    buffer::end_single_time_commands(device, command_pool, command_buffer, queue);
}

pub fn copy_buffer_to_image(
    device: &Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) {
    let command_buffer = buffer::begin_single_time_commands(device, command_pool);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(
            vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        )
        .image_offset(vk::Offset3D::builder().x(0).y(0).z(0).build())
        .image_extent(
            vk::Extent3D::builder()
                .width(width)
                .height(height)
                .depth(1)
                .build(),
        )
        .build();

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }

    buffer::end_single_time_commands(device, command_pool, command_buffer, queue);
}

pub fn create_image_view(device: &Device, image: vk::Image, format: vk::Format) -> vk::ImageView {
    let create_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        )
        .build();

    unsafe {
        device
            .create_image_view(&create_info, None)
            .expect("Failed to create image view!")
    }
}
