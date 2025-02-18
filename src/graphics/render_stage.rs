use crate::core::command_buffer::VECommandBuffer;
use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::descriptor_set_layout::VEDescriptorSetLayout;
use crate::core::device::VEDevice;
use crate::core::shader_module::VEShaderModule;
use crate::graphics::attachment::VEAttachment;
use crate::graphics::framebuffer::{VEFrameBuffer, VEFrameBufferError};
use crate::graphics::graphics_pipeline::{VEGraphicsPipeline, VEGraphicsPipelineError};
use crate::graphics::renderpass::{VERenderPass, VERenderPassError};
use crate::graphics::vertex_attributes::VertexAttribFormat;
use ash::vk;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VERenderStageError {
    #[error("graphics pipeline error")]
    GraphicsPipelineError(#[from] VEGraphicsPipelineError),

    #[error("render pass error")]
    RenderPassError(#[from] VERenderPassError),

    #[error("framebuffer error")]
    FrameBufferError(#[from] VEFrameBufferError),
}

static BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;

pub struct VERenderStage {
    device: Arc<VEDevice>,
    pipeline: Arc<VEGraphicsPipeline>,
    render_pass: VERenderPass,
    framebuffer: VEFrameBuffer,
    viewport_width: u32,
    viewport_height: u32,
    clear_values: Vec<vk::ClearValue>,
}

#[derive(Clone)]
pub enum VECullMode {
    None,
    Front,
    Back,
}

#[derive(Clone)]
pub enum VEPrimitiveTopology {
    Points,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
    TriangleFan,
}

fn get_primitive_topology(topo: VEPrimitiveTopology) -> vk::PrimitiveTopology {
    match topo {
        VEPrimitiveTopology::Points => vk::PrimitiveTopology::POINT_LIST,
        VEPrimitiveTopology::LineList => vk::PrimitiveTopology::LINE_LIST,
        VEPrimitiveTopology::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
        VEPrimitiveTopology::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
        VEPrimitiveTopology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
        VEPrimitiveTopology::TriangleFan => vk::PrimitiveTopology::TRIANGLE_FAN,
    }
}

fn get_cull_flags(mode: VECullMode) -> vk::CullModeFlags {
    match mode {
        VECullMode::None => vk::CullModeFlags::NONE,
        VECullMode::Front => vk::CullModeFlags::FRONT,
        VECullMode::Back => vk::CullModeFlags::BACK,
    }
}

impl VERenderStage {
    pub fn new(
        device: Arc<VEDevice>,
        viewport_width: u32,
        viewport_height: u32,
        attachments: &[&VEAttachment],
        set_layouts: &[&VEDescriptorSetLayout],
        vertex_shader: &VEShaderModule,
        fragment_shader: &VEShaderModule,
        vertex_attributes: &[VertexAttribFormat],
        primitive_topology: VEPrimitiveTopology,
        cull_mode: VECullMode,
    ) -> Result<VERenderStage, VERenderStageError> {
        let render_pass = VERenderPass::new(device.clone(), attachments)?;

        let framebuffer = VEFrameBuffer::new(
            device.clone(),
            viewport_width,
            viewport_height,
            &render_pass,
            attachments,
        )?;

        let pipeline = VEGraphicsPipeline::new(
            device.clone(),
            viewport_width,
            viewport_height,
            set_layouts,
            vertex_shader,
            fragment_shader,
            &render_pass,
            attachments,
            vertex_attributes,
            get_primitive_topology(primitive_topology),
            get_cull_flags(cull_mode),
        )?;

        let clear_values = attachments
            .iter()
            .map(|a| a.clear.unwrap_or_else(|| vk::ClearValue::default()))
            .collect();

        Ok(VERenderStage {
            device: device.clone(),
            pipeline: Arc::new(pipeline),
            render_pass,
            framebuffer,
            viewport_width,
            viewport_height,
            clear_values,
        })
    }

    pub fn set_descriptor_set(
        &self,
        command_buffer: &VECommandBuffer,
        index: u32,
        set: &VEDescriptorSet,
    ) {
        unsafe {
            self.device.device.cmd_bind_descriptor_sets(
                command_buffer.handle,
                BIND_POINT,
                self.pipeline.layout,
                index,
                &[set.set],
                &[],
            );
        }
    }

    pub fn bind(&self, command_buffer: &VECommandBuffer) {
        let rect = vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(
                vk::Extent2D::default()
                    .width(self.viewport_width)
                    .height(self.viewport_height),
            );

        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .framebuffer(self.framebuffer.handle)
            .render_pass(self.render_pass.handle)
            .clear_values(&self.clear_values)
            .render_area(rect);

        unsafe {
            self.device.device.cmd_begin_render_pass(
                command_buffer.handle,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            self.device.device.cmd_bind_pipeline(
                command_buffer.handle,
                BIND_POINT,
                self.pipeline.pipeline,
            );
        }
    }

    pub fn end_render_pass(&self, command_buffer: &VECommandBuffer) {
        unsafe {
            self.device
                .device
                .cmd_end_render_pass(command_buffer.handle);
        }
    }
}
