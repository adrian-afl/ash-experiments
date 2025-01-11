use crate::core::descriptor_set_layout::VEDescriptorSetLayout;
use crate::core::device::VEDevice;
use crate::core::shader_module::VEShaderModule;
use ash::vk;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEComputePipelineError {
    #[error("layout creation failed")]
    LayoutCreationFailed(#[source] vk::Result),

    #[error("pipeline creation failed")]
    PipelineCreationFailed(#[source] vk::Result),
}

pub struct VEComputePipeline {
    device: Arc<VEDevice>,
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

impl VEComputePipeline {
    pub fn new(
        device: Arc<VEDevice>,
        set_layouts: &[&VEDescriptorSetLayout],
        shader: &VEShaderModule,
    ) -> Result<VEComputePipeline, VEComputePipelineError> {
        let shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader.handle)
            .name(c"main");
        let layouts: Vec<vk::DescriptorSetLayout> = set_layouts.iter().map(|x| x.layout).collect();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        let pipeline_layout = unsafe {
            device
                .device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(VEComputePipelineError::LayoutCreationFailed)?
        };

        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(shader_stage_info)
            .layout(pipeline_layout);

        let pipeline = unsafe {
            device
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|e| VEComputePipelineError::LayoutCreationFailed(e.1))?[0]
        };

        Ok(VEComputePipeline {
            device,
            pipeline,
            layout: pipeline_layout,
        })
    }
}

// TODO drop
