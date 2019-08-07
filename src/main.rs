use std::{
    default::Default,
    error::Error,
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
    ptr,
};

extern crate ash;
use ash::{
    Device,
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Win32Surface},
    },
    prelude::VkResult,
    version::{EntryV1_0, InstanceV1_0},
    vk, Entry, Instance,
};

extern crate winit;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    //platform::desktop, TODO figure out how to pass to Vulkan context
    window::{Window, WindowBuilder},
};

mod utils;
use utils::debugging;
use ash::version::DeviceV1_0;

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;

struct HelloTriangleApplication {
    entry: Entry,
    instance: Instance,
    debug_utils: DebugUtils,
    debug_utils_messenger_ext: vk::DebugUtilsMessengerEXT,
    device: Device
}

impl HelloTriangleApplication {
    pub fn new() -> Self {
        let (entry, instance) = Self::init_vulkan();
        let (debug_utils, debug_utils_messenger_ext) =
            Self::setup_debug_messenger(&entry, &instance);
        let physical_device = Self::select_physical_device(&instance);
        let (device, queue_family_index) = Self::create_logical_device(&instance, physical_device);

        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Self {
            entry,
            instance,
            debug_utils,
            debug_utils_messenger_ext,
            device
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.main_loop();
        Ok(())
    }

    fn init_window() -> (EventLoop<()>, Window) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
            .with_title("Vulkan tutorial")
            .build(&event_loop);

        match window {
            Ok(window) => (event_loop, window),
            Err(e) => panic!("Failed to create window: {}", e),
        }
    }

    fn init_vulkan() -> (Entry, Instance) {
        let instance_result = Self::create_vk_instance();
        match instance_result {
            Ok((entry, instance)) => (entry, instance),
            Err(e) => panic!("Failed to create Vulkan instance: {}", e),
        }
    }

    fn create_vk_instance() -> Result<(Entry, Instance), Box<dyn Error>> {
        let entry = Entry::new()?;
        if ENABLE_VALIDATION_LAYERS && !debugging::check_validation_layer_support(&entry) {
            panic!("Validation layers requested but not available!")
        }

        let application_name = CString::new("Hello triangle").unwrap();
        let engine_name = CString::new("No engine").unwrap();
        let version = ash::vk_make_version!(1, 0, 0);

        let application_info = vk::ApplicationInfo::builder()
            .application_name(&application_name)
            .application_version(version)
            .engine_name(&engine_name)
            .engine_version(version)
            .api_version(version)
            .build();

        // variables below in main function body to prevent getting destroyed before entry.create_instance()
        let enabled_layer_names = debugging::get_enabled_layer_names();
        let enabled_layer_names: Vec<*const c_char> = enabled_layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
        let enabled_extension_names = Self::get_required_extensions();
        let create_info = Self::populate_debug_messenger_create_info();

        let mut instance_create_info_builder =
            vk::InstanceCreateInfo::builder().application_info(&application_info);
        if ENABLE_VALIDATION_LAYERS {
            instance_create_info_builder = instance_create_info_builder
                .enabled_layer_names(&enabled_layer_names)
                .enabled_extension_names(&enabled_extension_names);
        }

        let mut instance_create_info = instance_create_info_builder.build();
        if ENABLE_VALIDATION_LAYERS {
            instance_create_info.p_next =
                &create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void;
        }

        let instance = unsafe { entry.create_instance(&instance_create_info, None)? };
        Ok((entry, instance))
    }

    fn get_required_extensions() -> Vec<*const c_char> {
        vec![
            Surface::name().as_ptr(),
            Win32Surface::name().as_ptr(),
            DebugUtils::name().as_ptr(),
        ]
    }

    fn setup_debug_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> (DebugUtils, vk::DebugUtilsMessengerEXT) {
        let debug_utils = DebugUtils::new(entry, instance);

        if !ENABLE_VALIDATION_LAYERS {
            return (debug_utils, vk::DebugUtilsMessengerEXT::null());
        }

        let create_info = Self::populate_debug_messenger_create_info();

        unsafe {
            let debug_utils_messenger_ext =
                DebugUtils::create_debug_utils_messenger(&debug_utils, &create_info, None)
                    .expect("Failed to create DebugUtilsMessengerEXT");
            (debug_utils, debug_utils_messenger_ext)
        }
    }

    fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT {
            s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            pfn_user_callback: Some(Self::debug_callback),
            ..Default::default()
        }
    }

    unsafe extern "system" fn debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        p_user_data: *mut c_void,
    ) -> vk::Bool32 {
            eprintln!(
                "validation layer: {}",
                CStr::from_ptr((*p_callback_data).p_message)
                    .to_str()
                    .unwrap()
            );
        vk::FALSE
    }

    fn select_physical_device(instance: &Instance) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        if physical_devices.len() == 0 {
            panic!("No physical devices with Vulkan support!");
        }

        let selected_device = physical_devices
            .into_iter()
            .filter_map(|device| {
                if Self::is_physical_device_suitable(instance, device) {
                    Some(device)
                } else {
                    None
                }
            }).collect::<Vec<vk::PhysicalDevice>>();

        if selected_device.len() == 0 {
            panic!("No suitable devices found!")
        }

        selected_device[0]
    }

    fn is_physical_device_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        let queue_family_index = Self::get_physical_device_queue_families(instance, device);
        queue_family_index.is_some()
    }

    fn get_physical_device_queue_families(instance: &Instance, device: vk::PhysicalDevice) -> Option<usize> {
        let queue_families_properties = unsafe { instance.get_physical_device_queue_family_properties(device) };
        let queue_families_indices = 0..queue_families_properties.len();
        let queue_family_indices = queue_families_properties.into_iter().zip(queue_families_indices).flat_map(|(queue_family, index)| {
            if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                Some(index)
            } else {
                None
            }
        }).collect::<Vec<usize>>();

        if queue_family_indices.len() == 0 {
            None
        } else {
            Some(queue_family_indices[0])
        }
    }

    fn create_logical_device(instance: &Instance, physical_device: vk::PhysicalDevice) -> (ash::Device, u32) {
        let queue_family_index = Self::get_physical_device_queue_families(instance, physical_device).expect("No queue families contain required flags!") as u32;
        let queue_priorities: [f32; 1] = [1.0];

        let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)
            .build()];

        let device_features = vk::PhysicalDeviceFeatures::builder().build();

        // variables below in main function body to prevent getting destroyed before entry.create_instance()
        let enabled_layer_names = debugging::get_enabled_layer_names();
        let enabled_layer_names: Vec<*const c_char> = enabled_layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let mut create_info_builder = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features);
        if ENABLE_VALIDATION_LAYERS {
            create_info_builder = create_info_builder.enabled_layer_names(&enabled_layer_names);
        }
        let create_info = create_info_builder.build();

        let device = unsafe { instance.create_device(physical_device, &create_info, None).expect("Failed to create logical device!") };
        (device, queue_family_index)
    }

    fn main_loop(&mut self) {
        let (event_loop, window) = Self::init_window();
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => *control_flow = ControlFlow::Wait,
        });
    }
}

impl Drop for HelloTriangleApplication {
    fn drop(&mut self) {
        if ENABLE_VALIDATION_LAYERS {
            unsafe {
                self.debug_utils
                    .destroy_debug_utils_messenger(self.debug_utils_messenger_ext, None)
            };
        }

        unsafe { self.instance.destroy_instance(None) };
    }
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    let status = app.run();
    match status {
        Ok(_) => (),
        Err(e) => panic!("Application crashed! Trace: {}", e),
    }
}
