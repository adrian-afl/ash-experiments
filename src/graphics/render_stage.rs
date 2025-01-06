use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::descriptor_set_layout::VEDescriptorSetLayout;
use crate::core::device::VEDevice;
use crate::core::shader_module::VEShaderModule;
use crate::graphics::attachment::VEAttachment;
use crate::graphics::framebuffer::VEFrameBuffer;
use crate::graphics::graphics_pipeline::VEGraphicsPipeline;
use crate::graphics::renderpass::VERenderPass;
use crate::graphics::vertex_attributes::VertexAttribFormat;
use crate::graphics::vertex_buffer::VEVertexBuffer;
use ash::vk;
use std::sync::Arc;

static BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;

pub struct VERenderStage {
    device: Arc<VEDevice>,
    pipeline: Arc<VEGraphicsPipeline>,
    pub command_buffer: VECommandBuffer,
    render_pass: VERenderPass,
    framebuffer: VEFrameBuffer,
    viewport_width: u32,
    viewport_height: u32,
    clear_values: Vec<vk::ClearValue>,
}

#[derive(Clone)]
pub enum CullMode {
    None,
    Front,
    Back,
}

impl VERenderStage {
    pub fn new(
        device: Arc<VEDevice>,
        command_pool: Arc<VECommandPool>,
        viewport_width: u32,
        viewport_height: u32,
        attachments: &[&VEAttachment],
        set_layouts: &[&VEDescriptorSetLayout],
        vertex_shader: &VEShaderModule,
        fragment_shader: &VEShaderModule,
        vertex_attributes: &[VertexAttribFormat],
        primitive_topology: vk::PrimitiveTopology,
        cull_mode: CullMode,
    ) -> VERenderStage {
        let cull_flags = match cull_mode {
            CullMode::None => vk::CullModeFlags::NONE,
            CullMode::Front => vk::CullModeFlags::FRONT,
            CullMode::Back => vk::CullModeFlags::BACK,
        };

        let render_pass = VERenderPass::new(device.clone(), attachments);

        let framebuffer = VEFrameBuffer::new(
            // for what is this used? is this needed?? TODO
            device.clone(),
            viewport_width,
            viewport_height,
            &render_pass,
            attachments,
        );

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
            primitive_topology,
            cull_flags,
        );

        let clear_values = attachments
            .iter()
            .map(|a| match a.clear {
                None => vk::ClearValue::default(),
                Some(c) => c,
            })
            .collect();

        VERenderStage {
            device: device.clone(),
            pipeline: Arc::new(pipeline),
            command_buffer: VECommandBuffer::new(device, command_pool.clone()),
            render_pass,
            framebuffer,
            viewport_width,
            viewport_height,
            clear_values,
        }
    }

    pub fn set_descriptor_set(&self, index: u32, set: &VEDescriptorSet) {
        unsafe {
            self.device.device.cmd_bind_descriptor_sets(
                self.command_buffer.handle,
                BIND_POINT,
                self.pipeline.layout,
                index,
                &[set.set],
                &[],
            );
        }
    }

    pub fn begin_recording(&self) {
        self.command_buffer
            .begin(vk::CommandBufferUsageFlags::empty());

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
                self.command_buffer.handle,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            self.device.device.cmd_bind_pipeline(
                self.command_buffer.handle,
                BIND_POINT,
                self.pipeline.pipeline,
            );
        }
    }

    pub fn end_recording(&self) {
        unsafe {
            self.device
                .device
                .cmd_end_render_pass(self.command_buffer.handle);
        }
        self.command_buffer.end();
    }

    pub fn draw_instanced(&self, vertex_buffer: &VEVertexBuffer, instances: u32) {
        unsafe {
            self.device.device.cmd_bind_vertex_buffers(
                self.command_buffer.handle,
                0,
                &[vertex_buffer.buffer.buffer],
                &[0],
            );
            self.device.device.cmd_draw(
                self.command_buffer.handle,
                vertex_buffer.vertex_count,
                instances,
                0,
                0,
            );
        }
    }
}
