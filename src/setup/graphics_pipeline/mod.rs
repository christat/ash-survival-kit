use std::{
    ffi::CString,
    path::Path,
};

use ash::{
    Device,
    vk
};
use ash::version::DeviceV1_0;

mod utils;

pub struct Pipeline {
    pub shader_modules: Vec<vk::ShaderModule>
}

pub fn create(device: &Device) -> Pipeline {
    let vert_shader_raw = utils::read_shader(Path::new("src/shaders/vert.spv"));
    let frag_shader_raw = utils::read_shader(Path::new("src/shaders/frag.spv"));

    let vert_shader_module = create_shader_module(device, vert_shader_raw);
    let frag_shader_module = create_shader_module(device, frag_shader_raw);

    // /!\ entry point (function) of shader; we're sticking to main functions.
    let entry_point = CString::new("main").unwrap();

    let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(entry_point.as_c_str())
        .build();

    let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(entry_point.as_c_str())
        .build();

    let _shader_stages = vec![vert_shader_stage_info, frag_shader_stage_info];

    Pipeline {
        shader_modules: vec![
            vert_shader_module,
            frag_shader_module
        ]
    }
}

fn create_shader_module(device: &Device, shader_raw: Vec<u32>) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&shader_raw)
        .build();

    let shader_module = unsafe { device.create_shader_module(&shader_module_create_info, None).expect("Failed to create shader module!") };
    shader_module
}
