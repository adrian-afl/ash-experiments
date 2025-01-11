use crate::core::descriptor_set::{VEDescriptorSet, VEDescriptorSetError};
use crate::core::device::VEDevice;
use ash::vk;
use std::sync::Arc;
use thiserror::Error;

pub enum VEDescriptorSetFieldStage {
    All,
    AllGraphics,
    Compute,
    Vertex,
    Fragment,
}

pub enum VEDescriptorSetFieldType {
    Sampler,
    UniformBuffer,
    StorageBuffer,
    StorageImage,
}

pub struct VEDescriptorSetLayout {
    device: Arc<VEDevice>,
    allocation_counter: u32,
    pub layout: vk::DescriptorSetLayout,
    pools: Vec<Arc<vk::DescriptorPool>>,
}

pub struct VEDescriptorSetLayoutField {
    pub binding: u32,
    pub typ: VEDescriptorSetFieldType,
    pub stage: VEDescriptorSetFieldStage,
}

#[derive(Error, Debug)]
pub enum VEDescriptorSetLayoutError {
    #[error("creation failed")]
    CreationFailed(#[source] vk::Result),

    #[error("pool creation failed")]
    PoolCreationFailed(#[source] vk::Result),

    #[error("no pool found")]
    NoPoolFound,

    #[error("descriptor set creation failed")]
    DescriptorSetCreationFailed(#[source] VEDescriptorSetError),
}

static DEFAULT_POOL_SIZE: u32 = 256;

impl VEDescriptorSetLayout {
    pub fn new(
        device: Arc<VEDevice>,
        fields: &[VEDescriptorSetLayoutField],
    ) -> Result<VEDescriptorSetLayout, VEDescriptorSetLayoutError> {
        let mut bindings = vec![];
        for field in fields {
            let typ = match field.typ {
                VEDescriptorSetFieldType::Sampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                VEDescriptorSetFieldType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
                VEDescriptorSetFieldType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
                VEDescriptorSetFieldType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
            };
            let stage = match field.stage {
                VEDescriptorSetFieldStage::All => vk::ShaderStageFlags::ALL,
                VEDescriptorSetFieldStage::AllGraphics => vk::ShaderStageFlags::ALL_GRAPHICS,
                VEDescriptorSetFieldStage::Compute => vk::ShaderStageFlags::COMPUTE,
                VEDescriptorSetFieldStage::Vertex => vk::ShaderStageFlags::VERTEX,
                VEDescriptorSetFieldStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            };
            bindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(field.binding)
                    .descriptor_count(1)
                    .descriptor_type(typ)
                    .stage_flags(stage),
            )
        }

        let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

        let layout = unsafe {
            device
                .device
                .create_descriptor_set_layout(&info, None)
                .map_err(VEDescriptorSetLayoutError::CreationFailed)?
        };

        Ok(VEDescriptorSetLayout {
            device,
            layout,
            pools: vec![],
            allocation_counter: 0,
        })
    }

    pub fn create_descriptor_set(&mut self) -> Result<VEDescriptorSet, VEDescriptorSetLayoutError> {
        if self.pools.len() == 0 {
            self.generate_new_set_pool()?;
        } else {
            self.allocation_counter += 1;
            if self.allocation_counter >= DEFAULT_POOL_SIZE {
                self.generate_new_set_pool()?;
                self.allocation_counter = 0;
            }
        }
        let pool = self
            .pools
            .last()
            .ok_or_else(|| VEDescriptorSetLayoutError::NoPoolFound)?;
        VEDescriptorSet::new(self.device.clone(), self.layout, pool)
            .map_err(VEDescriptorSetLayoutError::DescriptorSetCreationFailed)
    }

    pub fn generate_new_set_pool(&mut self) -> Result<(), VEDescriptorSetLayoutError> {
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(DEFAULT_POOL_SIZE),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(DEFAULT_POOL_SIZE),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(DEFAULT_POOL_SIZE),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(DEFAULT_POOL_SIZE),
        ];
        let info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(DEFAULT_POOL_SIZE); // TODO weird
        let pool = unsafe {
            self.device
                .device
                .create_descriptor_pool(&info, None)
                .map_err(VEDescriptorSetLayoutError::PoolCreationFailed)?
        };
        self.pools.push(Arc::new(pool));
        Ok(())
    }
}

impl Drop for VEDescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .destroy_descriptor_set_layout(self.layout, None)
        }
    }
}
