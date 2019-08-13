use ash::{
    extensions::khr::{Surface, Swapchain},
    version::DeviceV1_0,
    vk, Device, Instance,
};

pub mod utils;
use crate::setup::devices::utils as device_utils;

pub struct SwapchainData {
    pub swapchain: Swapchain,
    pub swapchain_khr: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub image_format: vk::Format,
    pub image_extent: vk::Extent2D,
}

pub fn create(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    device: &Device,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
) -> SwapchainData {
    let utils::SwapchainDetails {
        capabilities,
        formats,
        present_modes,
    } = utils::query_swapchain_support(physical_device, surface, surface_khr);

    let vk::SurfaceFormatKHR {
        format: image_format,
        color_space,
    } = utils::select_swapchain_surface_format(formats);
    let present_mode = utils::select_swapchain_present_mode(present_modes);
    let image_extent = utils::select_swapchain_extent(capabilities);

    // 0 is a special case (== unlimited max count); otherwise, guard from max count
    let image_count = if capabilities.max_image_count == 0 {
        capabilities.min_image_count + 1
    } else {
        u32::min(
            capabilities.min_image_count + 1,
            capabilities.max_image_count,
        )
    };

    let device_utils::QueueFamilyIndices { graphics, present } =
        device_utils::get_physical_device_queue_family_indices(
            instance,
            physical_device,
            surface,
            surface_khr,
        )
        .expect("No queue families contain required flags!");;
    // enable swapchaing sharing and pass relevant indices to struct iff both queue indices are the different.
    let (image_sharing_mode, queue_family_indices) = match graphics == present {
        true => (vk::SharingMode::EXCLUSIVE, vec![]),
        false => (vk::SharingMode::CONCURRENT, vec![graphics, present]),
    };

    let mut swapchain_create_info_builder = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface_khr)
        .min_image_count(image_count)
        .image_format(image_format)
        .image_color_space(color_space)
        .image_extent(image_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .pre_transform(capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    if !queue_family_indices.is_empty() {
        swapchain_create_info_builder =
            swapchain_create_info_builder.queue_family_indices(&queue_family_indices);
    }

    let swapchain_create_info = swapchain_create_info_builder.build();

    let swapchain = Swapchain::new(instance, device);
    let swapchain_khr = unsafe {
        swapchain
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create swapchain!")
    };
    let swapchain_images = unsafe {
        swapchain
            .get_swapchain_images(swapchain_khr)
            .expect("Failed to get swapchain images!")
    };
    let swapchain_image_views = create_image_views(device, &swapchain_images, image_format);

    SwapchainData {
        swapchain,
        swapchain_khr,
        swapchain_images,
        swapchain_image_views,
        image_format,
        image_extent,
    }
}

fn create_image_views(
    device: &Device,
    swapchain_images: &[vk::Image],
    format: vk::Format,
) -> Vec<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1)
        .build();

    let image_views = swapchain_images.iter().fold(vec![], |mut acc, image| {
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(*image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping::default())
            .subresource_range(subresource_range)
            .build();

        let image_view = unsafe {
            device
                .create_image_view(&image_view_create_info, None)
                .expect("Failed to create image views!")
        };
        acc.push(image_view);
        acc
    });
    image_views
}
