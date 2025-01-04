use crate::command_buffer::VECommandBuffer;
use crate::compute_pipeline::VEComputePipeline;
use crate::descriptor_set::VEDescriptorSet;
use crate::descriptor_set_layout::VEDescriptorSetLayout;
use crate::device::VEDevice;
use crate::shader_module::VEShaderModule;
use ash::vk;
use ash::vk::CommandBufferUsageFlags;
use std::sync::Arc;

pub struct VEComputeStage {
    device: Arc<VEDevice>,
    pipeline: Arc<VEComputePipeline>,
    sets: Vec<Arc<VEDescriptorSet>>,
}

static BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::COMPUTE;

impl VEComputeStage {
    pub fn new(
        device: Arc<VEDevice>,
        set_layouts: &[&VEDescriptorSetLayout],
        shader: &VEShaderModule,
    ) -> VEComputeStage {
        let pipeline = VEComputePipeline::new(device.clone(), set_layouts, &shader);
        VEComputeStage {
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

    pub fn dispatch(
        &self,
        command_buffer: &VECommandBuffer,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            self.device.device.cmd_dispatch(
                command_buffer.handle,
                group_count_x,
                group_count_y,
                group_count_z,
            );
        }
    }
}
