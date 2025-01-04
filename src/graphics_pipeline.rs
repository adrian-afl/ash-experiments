use crate::attachment::AttachmentBlending;
use crate::descriptor_set_layout::VEDescriptorSetLayout;
use crate::device::VEDevice;
use crate::renderpass::VERenderPass;
use crate::shader_module::VEShaderModule;
use crate::vertex_attributes::{create_vertex_input_state_descriptions, VertexAttribFormat};
use ash::vk;
use ash::vk::{ColorComponentFlags, PrimitiveTopology, RenderPass};
use std::sync::Arc;

pub struct VEGraphicsPipeline {
    device: Arc<VEDevice>,
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

impl VEGraphicsPipeline {
    pub fn new(
        device: Arc<VEDevice>,
        viewport_width: u32,
        viewport_height: u32,
        set_layouts: &[&VEDescriptorSetLayout],
        vertex_shader: &VEShaderModule,
        fragment_shader: &VEShaderModule,
        render_pass: Arc<VERenderPass>,
        vertex_attributes: &[VertexAttribFormat],
        enable_depth: bool,
        primitive_topology: vk::PrimitiveTopology,
        cull_flags: vk::CullModeFlags,
    ) -> VEGraphicsPipeline {
        let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader.handle)
            .name(c"main");

        let fragment_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader.handle)
            .name(c"main");

        let shader_stage_infos = [vertex_shader_stage_info, fragment_shader_stage_info];

        let layouts: Vec<vk::DescriptorSetLayout> = set_layouts.iter().map(|x| x.layout).collect();

        let vertex_attrib_descriptions = create_vertex_input_state_descriptions(vertex_attributes);
        let tmp_binds = [vertex_attrib_descriptions.0];
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&tmp_binds)
            .vertex_attribute_descriptions(&vertex_attrib_descriptions.1);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(primitive_topology)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(viewport_width as f32)
            .height(viewport_height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(
                vk::Extent2D::default()
                    .width(viewport_width)
                    .height(viewport_height),
            );

        let tmp_viewports = [viewport];
        let tmp_scissors = [scissor];
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&tmp_viewports)
            .scissors(&tmp_scissors);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(true)
            .rasterizer_discard_enable(false) // TODO sure????
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(cull_flags)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(enable_depth)
            .depth_write_enable(enable_depth)
            .depth_compare_op(if enable_depth {
                vk::CompareOp::LESS
            } else {
                vk::CompareOp::ALWAYS
            })
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0);

        let mut attas: Vec<vk::PipelineColorBlendAttachmentState> = vec![];

        //for att in render_pass.attachments {
        for i in 0..render_pass.attachments.len() {
            let att = &render_pass.attachments[i];
            if !att.image.is_depth() {
                // not a depth buffer
                let mut blend_state = vk::PipelineColorBlendAttachmentState::default()
                    .color_write_mask(ColorComponentFlags::RGBA);
                match &att.blending {
                    None => {
                        blend_state = blend_state
                            .color_blend_op(vk::BlendOp::ADD)
                            .src_color_blend_factor(vk::BlendFactor::ONE)
                            .dst_color_blend_factor(vk::BlendFactor::ONE)
                            .alpha_blend_op(vk::BlendOp::ADD)
                            .src_alpha_blend_factor(vk::BlendFactor::ONE)
                            .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                            .blend_enable(false);
                    }
                    Some(blending) => match blending {
                        AttachmentBlending::Alpha => {
                            blend_state = blend_state
                                .color_blend_op(vk::BlendOp::ADD)
                                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                                .alpha_blend_op(vk::BlendOp::ADD)
                                .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                                .blend_enable(true);
                        }
                        AttachmentBlending::Additive => {
                            blend_state = blend_state
                                .color_blend_op(vk::BlendOp::ADD)
                                .src_color_blend_factor(vk::BlendFactor::ONE)
                                .dst_color_blend_factor(vk::BlendFactor::ONE)
                                .alpha_blend_op(vk::BlendOp::ADD)
                                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                                .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                                .blend_enable(true);
                        }
                    },
                }
                attas.push(blend_state);
            }
        }

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&attas)
            .blend_constants([1.0, 1.0, 1.0, 1.0]);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        let pipeline_layout = unsafe {
            device
                .device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .unwrap()
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stage_infos)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass.handle)
            .subpass(0);

        let pipeline = unsafe {
            device
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .unwrap()[0]
        };

        VEGraphicsPipeline {
            device,
            pipeline,
            layout: pipeline_layout,
        }
    }
}
