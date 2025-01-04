use crate::attachment::VEAttachment;
use crate::command_buffer::VECommandBuffer;
use crate::compute_pipeline::VEComputePipeline;
use crate::compute_stage::VEComputeStage;
use crate::descriptor_set::VEDescriptorSet;
use crate::descriptor_set_layout::VEDescriptorSetLayout;
use crate::device::VEDevice;
use crate::framebuffer::VEFrameBuffer;
use crate::graphics_pipeline::VEGraphicsPipeline;
use crate::renderpass::VERenderPass;
use crate::shader_module::VEShaderModule;
use crate::subpass::{create_subpass, create_subpass_attachment_reference};
use crate::vertex_attributes::VertexAttribFormat;
use crate::vertex_buffer::VEVertexBuffer;
use ash::vk;
use ash::vk::{CommandBufferUsageFlags, RenderPass};
use std::sync::Arc;

static BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;

pub struct VERenderStage {
    device: Arc<VEDevice>,
    pipeline: Arc<VEGraphicsPipeline>,
    sets: Vec<Arc<VEDescriptorSet>>,
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

        let color_attas: Vec<&&VEAttachment> =
            attachments.iter().filter(|x| !x.image.is_depth()).collect();
        let depth_atta = attachments.iter().filter(|x| x.image.is_depth()).last();

        let color_references: Vec<vk::AttachmentReference> = (0..color_attas.len())
            .map(|i| create_subpass_attachment_reference(i as i32, false))
            .collect();

        let depth_reference_maybe =
            create_subpass_attachment_reference(color_attas.len() as i32, true);
        let depth_reference = match depth_atta {
            None => None,
            Some(_) => Some(&depth_reference_maybe), // depth last
        };

        let subpass = create_subpass(&color_references, depth_reference);
        let subpasses = [subpass];

        let render_pass = VERenderPass::new(device.clone(), attachments, &subpasses);

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
            render_pass,
            attachments,
            vertex_attributes,
            primitive_topology,
            cull_flags,
        );
        VERenderStage {
            device,
            pipeline: Arc::new(pipeline),
            sets: vec![],
        }
    }

    pub fn set_descriptor_set(
        &mut self,
        command_buffer: &VECommandBuffer,
        index: usize,
        set: Arc<VEDescriptorSet>,
    ) {
        while self.sets.len() <= index {
            self.sets.push(set.clone()); // TODO weird but can work
        }
        self.sets[index] = set;
        self.bind_descriptor_sets(command_buffer);
    }

    fn bind_descriptor_sets(&self, command_buffer: &VECommandBuffer) {
        let handles: Vec<vk::DescriptorSet> = self.sets.iter().map(|x| x.set).collect();
        unsafe {
            self.device.device.cmd_bind_descriptor_sets(
                command_buffer.handle,
                BIND_POINT,
                self.pipeline.layout,
                0,
                &handles,
                &[],
            );
        }
    }

    pub fn begin_recording(&self, command_buffer: &VECommandBuffer) {
        command_buffer.begin(CommandBufferUsageFlags::empty());
        unsafe {
            self.device.device.cmd_bind_pipeline(
                command_buffer.handle,
                BIND_POINT,
                self.pipeline.pipeline,
            );
        }
    }

    pub fn end_recording(&self, command_buffer: &VECommandBuffer) {
        command_buffer.end();
    }
}
