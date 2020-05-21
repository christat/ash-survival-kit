use ash::{extensions::khr::Surface, vk};
use winit::dpi::PhysicalSize;

pub struct SwapchainDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn query_swapchain_support(
    physical_device: vk::PhysicalDevice,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
) -> SwapchainDetails {
    unsafe {
        let capabilities = surface
            .get_physical_device_surface_capabilities(physical_device, surface_khr)
            .expect("Failed to query physical device surface capabilities!");
        let formats = surface
            .get_physical_device_surface_formats(physical_device, surface_khr)
            .expect("Failed to query physical device surface formats!");
        let present_modes = surface
            .get_physical_device_surface_present_modes(physical_device, surface_khr)
            .expect("Failed to query physical device surface present modes!");
        SwapchainDetails {
            capabilities,
            formats,
            present_modes,
        }
    }
}

pub fn select_swapchain_surface_format(
    available_formats: Vec<vk::SurfaceFormatKHR>,
) -> vk::SurfaceFormatKHR {
    if available_formats.len() == 0 {
        panic!("No swapchain surface formats available in provided vector!")
    };
    let first_available_format = available_formats.first().unwrap().to_owned();
    let selected_format = available_formats.into_iter().skip(1).fold(
        first_available_format,
        |acc, available_format| {
            if available_format.format == vk::Format::B8G8R8A8_UNORM
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                available_format
            } else {
                acc
            }
        },
    );
    selected_format
}

pub fn select_swapchain_present_mode(
    available_present_modes: Vec<vk::PresentModeKHR>,
) -> vk::PresentModeKHR {
    if available_present_modes.len() == 0 {
        panic!("No swapchain present modes available in provided vector!")
    };
    if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::FIFO
    }
}

pub fn select_swapchain_extent(capabilities: vk::SurfaceCapabilitiesKHR, window_size: PhysicalSize<u32>) -> vk::Extent2D {
    if capabilities.current_extent.width != std::u32::MAX {
        capabilities.current_extent
    } else {
        let (width, height): (u32, u32) = window_size.into();
        let extent = vk::Extent2D::builder()
            .width(u32::max(
                capabilities.min_image_extent.width,
                u32::min(
                    capabilities.max_image_extent.width,
                    width
                ),
            ))
            .height(u32::max(
                capabilities.min_image_extent.height,
                u32::min(
                    capabilities.max_image_extent.height,
                    height,
                ),
            ))
            .build();
        extent
    }
}
