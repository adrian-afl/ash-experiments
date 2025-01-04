use crate::device::VEDevice;
use ash::vk;
use std::sync::Arc;

enum DescriptorSetFieldStage {
    All,
    AllGraphics,
    Compute,
    Vertex,
    Fragment,
}

enum DescriptorSetFieldType {
    Sampler,
    UniformBuffer,
    StorageBuffer,
    StorageImage,
}

pub struct DescriptorSetLayout<'a> {
    device: Arc<VEDevice>,
    allocation_counter: u32,
    layout: vk::DescriptorSetLayout,
    bindings: Vec<Arc<vk::DescriptorSetLayoutBinding<'a>>>,
    pools: Vec<Arc<vk::DescriptorPool>>,
}

pub struct DescriptorSetLayoutField {
    binding: u32,
    typ: DescriptorSetFieldType,
    stage: DescriptorSetFieldStage,
}

static DEFAULT_POOL_SIZE: u32 = 1000;

impl<'a> DescriptorSetLayout<'a> {
    pub fn new(
        device: Arc<VEDevice>,
        fields: Vec<DescriptorSetLayoutField>,
    ) -> DescriptorSetLayout {
        let mut bindings = vec![];
        for field in fields {
            let typ = match field.typ {
                DescriptorSetFieldType::Sampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                DescriptorSetFieldType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
                DescriptorSetFieldType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
                DescriptorSetFieldType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
            };
            let stage = match field.stage {
                DescriptorSetFieldStage::All => vk::ShaderStageFlags::ALL,
                DescriptorSetFieldStage::AllGraphics => vk::ShaderStageFlags::ALL_GRAPHICS,
                DescriptorSetFieldStage::Compute => vk::ShaderStageFlags::COMPUTE,
                DescriptorSetFieldStage::Vertex => vk::ShaderStageFlags::VERTEX,
                DescriptorSetFieldStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            };
            bindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(field.binding)
                    .descriptor_type(typ)
                    .stage_flags(stage),
            )
        }

        DescriptorSetLayout {}
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
            .max_sets(1000);
        let pool = unsafe {
            self.device
                .device
                .create_descriptor_pool(&info, None)
                .unwrap()
        };
        self.pools.push(Arc::new(pool));
    }
}
