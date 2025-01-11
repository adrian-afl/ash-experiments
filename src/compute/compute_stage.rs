use crate::compute::compute_pipeline::{VEComputePipeline, VEComputePipelineError};
use crate::core::command_buffer::{VECommandBuffer, VECommandBufferError};
use crate::core::command_pool::VECommandPool;
use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::descriptor_set_layout::VEDescriptorSetLayout;
use crate::core::device::VEDevice;
use crate::core::shader_module::VEShaderModule;
use ash::vk;
use ash::vk::CommandBufferUsageFlags;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEComputeStageError {
    #[error("compute pipeline error")]
    ComputePipelineError(#[from] VEComputePipelineError),

    #[error("command buffer error")]
    CommandBufferError(#[from] VECommandBufferError),
}

static BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::COMPUTE;

pub struct VEComputeStage {
    device: Arc<VEDevice>,
    pipeline: Arc<VEComputePipeline>,
    pub command_buffer: VECommandBuffer,
}

impl VEComputeStage {
    pub fn new(
        device: Arc<VEDevice>,
        command_pool: Arc<VECommandPool>,
        set_layouts: &[&VEDescriptorSetLayout],
        shader: &VEShaderModule,
    ) -> Result<VEComputeStage, VEComputeStageError> {
        let pipeline = VEComputePipeline::new(device.clone(), set_layouts, &shader)?;
        Ok(VEComputeStage {
            device: device.clone(),
            pipeline: Arc::new(pipeline),
            command_buffer: VECommandBuffer::new(device, command_pool.clone())?,
        })
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

    pub fn begin_recording(&self) -> Result<(), VEComputeStageError> {
        self.command_buffer
            .begin(CommandBufferUsageFlags::empty())?;
        unsafe {
            self.device.device.cmd_bind_pipeline(
                self.command_buffer.handle,
                BIND_POINT,
                self.pipeline.pipeline,
            );
        }
        Ok(())
    }

    pub fn end_recording(&self) -> Result<(), VEComputeStageError> {
        self.command_buffer.end()?;
        Ok(())
    }

    pub fn dispatch(&self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        unsafe {
            self.device.device.cmd_dispatch(
                self.command_buffer.handle,
                group_count_x,
                group_count_y,
                group_count_z,
            );
        }
    }
}
