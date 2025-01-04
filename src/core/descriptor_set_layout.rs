use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::device::VEDevice;
use ash::vk;
use std::sync::Arc;

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

static DEFAULT_POOL_SIZE: u32 = 256;

impl VEDescriptorSetLayout {
    pub fn new(
        device: Arc<VEDevice>,
        fields: &[VEDescriptorSetLayoutField],
    ) -> VEDescriptorSetLayout {
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
                .unwrap()
        };

        VEDescriptorSetLayout {
            device,
            layout,
            pools: vec![],
            allocation_counter: 0,
        }
    }

    pub fn create_descriptor_set(&mut self) -> VEDescriptorSet {
        if (self.pools.len() == 0) {
            self.generate_new_set_pool();
        } else {
            self.allocation_counter += 1;
            if (self.allocation_counter >= DEFAULT_POOL_SIZE) {
                self.generate_new_set_pool();
                self.allocation_counter = 0;
            }
        }
        let pool = self.pools.last().unwrap();
        VEDescriptorSet::new(self.device.clone(), self.layout, pool)
    }

    pub fn generate_new_set_pool(&mut self) {
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
                .unwrap()
        };
        self.pools.push(Arc::new(pool));
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
