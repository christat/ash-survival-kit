use std::ptr::copy_nonoverlapping;

use ash::{version::DeviceV1_0, vk, Device, Instance};
use image::GenericImageView;

use crate::setup::buffer;
use crate::setup::buffer::{begin_single_time_commands, end_single_time_commands};
use ash::version::InstanceV1_0;

pub fn create(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Image, vk::DeviceMemory, u32) {
    let src = image::open("src/resources/textures/viking_room.png")
        .expect("Failed to load image from path!");
    let (width, height) = src.dimensions();
    let mip_levels = f32::floor(f32::log2(u32::max(width, height) as f32)) as u32 + 1;
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
        mip_levels,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::SAMPLED,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    transition_image_layout(
        device,
        command_pool,
        queue,
        texture_image,
        // vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        mip_levels,
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

    // transition_image_layout(
    //     device,
    //     command_pool,
    //     queue,
    //     texture_image,
    //     vk::Format::R8G8B8A8_SRGB,
    //     vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    //     vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    //     mip_levels,
    // );

    generate_mipmaps(
        instance,
        device,
        physical_device,
        command_pool,
        queue,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        width,
        height,
        mip_levels,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_memory, None);
    }

    (texture_image, texture_image_memory, mip_levels)
}

fn generate_mipmaps(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    image: vk::Image,
    image_format: vk::Format,
    width: u32,
    height: u32,
    mip_levels: u32,
) -> () {
    let format_properties =
        unsafe { instance.get_physical_device_format_properties(*physical_device, image_format) };

    let supports_linear_filter_for_format: bool = format_properties
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR);

    if !supports_linear_filter_for_format {
        panic!("Texture format does not support linear blitting!");
    }

    let command_buffer = begin_single_time_commands(device, command_pool);

    let mut mip_width = width;
    let mut mip_height = height;

    for i in 1..mip_levels {
        let mip_level = i - 1;

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .base_mip_level(mip_level)
            .layer_count(1)
            .level_count(1)
            .build();

        let barrier_transfer = vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(subresource_range)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
            .build();

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::default(),
                &[],
                &[],
                &[barrier_transfer],
            )
        };

        let blit = vk::ImageBlit::builder()
            .src_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::builder()
                    .x(mip_width as i32)
                    .y(mip_height as i32)
                    .z(1)
                    .build(),
            ])
            .src_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(mip_level)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .dst_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::builder()
                    .x(if mip_width > 1 {
                        (mip_width / 2) as i32
                    } else {
                        1
                    })
                    .y(if mip_height > 1 {
                        (mip_height / 2) as i32
                    } else {
                        1
                    })
                    .z(1)
                    .build(),
            ])
            .dst_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(i)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .build();

        unsafe {
            device.cmd_blit_image(
                command_buffer,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[blit],
                vk::Filter::LINEAR,
            );
        };

        let barrier_shader = vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(subresource_range)
            .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::TRANSFER_READ)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .build();

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::default(),
                &[],
                &[],
                &[barrier_shader],
            )
        }

        if mip_width > 1 {
            mip_width /= 2;
        }

        if mip_height > 1 {
            mip_height /= 2;
        }
    }

    let barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_array_layer(0)
                .base_mip_level(mip_levels - 1)
                .layer_count(1)
                .level_count(1)
                .build(),
        )
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::SHADER_READ)
        .build();

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::default(),
            &[],
            &[],
            &[barrier],
        )
    }

    end_single_time_commands(device, command_pool, command_buffer, queue);
}

fn create_image(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    width: u32,
    height: u32,
    mip_levels: u32,
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
        .mip_levels(mip_levels)
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

pub fn create_texture_image_view(
    device: &Device,
    image: vk::Image,
    mip_levels: u32,
) -> vk::ImageView {
    create_image_view(
        device,
        image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageAspectFlags::COLOR,
        mip_levels,
    )
}

pub fn create_texture_sampler(device: &Device, mip_levels: u32) -> vk::Sampler {
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
        .max_lod(mip_levels as f32)
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
    // format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    mip_levels: u32,
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
                .level_count(mip_levels)
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

pub fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
    mip_levels: u32,
) -> vk::ImageView {
    let create_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect_flags)
                .base_mip_level(0)
                .level_count(mip_levels)
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

pub fn create_depth_resources(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    swapchain_extent: vk::Extent2D,
) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {
    let depth_format = find_depth_format(instance, physical_device);
    let (depth_image, depth_image_memory) = create_image(
        instance,
        device,
        physical_device,
        swapchain_extent.width,
        swapchain_extent.height,
        1,
        depth_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    let depth_image_view = create_image_view(
        device,
        depth_image,
        depth_format,
        vk::ImageAspectFlags::DEPTH,
        1,
    );
    (depth_image, depth_image_view, depth_image_memory)
}

pub fn find_depth_format(instance: &Instance, physical_device: &vk::PhysicalDevice) -> vk::Format {
    find_supported_format(
        instance,
        physical_device,
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

pub fn find_supported_format(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    for format in candidates.iter() {
        let format_properties =
            unsafe { instance.get_physical_device_format_properties(*physical_device, *format) };

        let linear_supported = tiling == vk::ImageTiling::LINEAR
            && (format_properties.linear_tiling_features & features) == features;

        let optimal_supported = tiling == vk::ImageTiling::OPTIMAL
            && (format_properties.optimal_tiling_features & features) == features;

        if linear_supported || optimal_supported {
            return *format;
        }
    }
    panic!("Failed to find supported format!");
}

// pub fn has_stencil_component(format: vk::Format) -> bool {
//     match format {
//         vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT => true,
//         _ => false,
//     }
// }
