use std::{ffi::CString, fs::File, io::Read, path::Path};

extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

use ash::{version::DeviceV1_0, vk, Device};

use crate::setup::swapchain::SwapchainData;
use crate::structs::Vertex;

pub fn create(
    device: &Device,
    swapchain_data: &SwapchainData,
    render_pass: vk::RenderPass,
    descriptor_set_layout: &vk::DescriptorSetLayout,
    msaa_samples: vk::SampleCountFlags,
) -> (Vec<vk::Pipeline>, vk::PipelineLayout) {
    let vert_shader_raw = read_shader(Path::new("src/shaders/vert.spv"));
    let frag_shader_raw = read_shader(Path::new("src/shaders/frag.spv"));

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

    let shader_stages = vec![vert_shader_stage_info, frag_shader_stage_info];

    let vertex_binding_descriptions = Vertex::get_binding_description();
    let vertex_attribute_descriptions = Vertex::get_attribute_descriptions();
    let pipeline_vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&vertex_binding_descriptions)
        .vertex_attribute_descriptions(&vertex_attribute_descriptions)
        .build();

    let pipeline_input_assembly_state_create_info =
        vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

    let viewports = [vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain_data.image_extent.width as f32)
        .height(swapchain_data.image_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0)
        .build()];

    let scissors = [vk::Rect2D::builder()
        .offset(vk::Offset2D::builder().x(0).y(0).build())
        .extent(swapchain_data.image_extent)
        .build()];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .viewports(&viewports)
        .scissor_count(1)
        .scissors(&scissors)
        .build();

    let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.0)
        .depth_bias_clamp(0.0)
        .depth_bias_slope_factor(0.0)
        .build();

    let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(msaa_samples)
        .min_sample_shading(1.0)
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false)
        .build();

    let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .build()];

    let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0])
        .build();

    /*
    let dynamic_states = [
        vk::DynamicState::VIEWPORT,
        vk::DynamicState::LINE_WIDTH
    ];

    let pipeline_dynamic_state_create_info = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&dynamic_states)
        .build();
    */

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&[*descriptor_set_layout])
        .build();

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Failed to create pipeline layout!")
    };

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        // .min_depth_bounds(0.0)
        // .max_depth_bounds(1.0)
        .stencil_test_enable(false)
        //.front(vk::StencilOpState::builder().build())
        //.back(vk::StencilOpState::builder().build())
        .build();

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&pipeline_vertex_input_state_create_info)
        .input_assembly_state(&pipeline_input_assembly_state_create_info)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
        .depth_stencil_state(&depth_stencil_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .base_pipeline_handle(vk::Pipeline::default())
        .base_pipeline_index(-1)
        //.dynamic_state(&pipeline_dynamic_state_create_info)
        .build();

    let pipelines = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::default(), &[pipeline_create_info], None)
            .expect("Failed to create graphics pipeline!")
    };

    // modules are safe to destroy right after creating pipelines
    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    }

    (pipelines, pipeline_layout)
}

fn read_shader(file_path: &Path) -> Vec<u32> {
    let shader_file =
        File::open(file_path).expect(&format!("Failed to read shader: {}", file_path.display()));
    let shader_bytes = shader_file
        .bytes()
        .filter_map(|byte| byte.ok())
        .collect::<Vec<u8>>();
    let shader_raw: Vec<u32> = (0..shader_bytes.len())
        .step_by(4)
        .fold(vec![], |mut acc, i| {
            acc.push(LittleEndian::read_u32(&shader_bytes[i..]));
            acc
        });
    shader_raw
}

fn create_shader_module(device: &Device, shader_raw: Vec<u32>) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&shader_raw)
        .build();

    let shader_module = unsafe {
        device
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create shader module!")
    };
    shader_module
}
