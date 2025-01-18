use crate::buffer::buffer::{VEBuffer, VEBufferError, VEBufferUsage};
use crate::core::device::VEDevice;
use crate::image::image::{VEImage, VEImageViewCreateInfo};
use crate::image::sampler::VESampler;
use ash::vk;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEDescriptorSetError {
    #[error("creation failed")]
    CreationFailed(#[from] vk::Result),

    #[error("image view not found when binding an image")]
    ImageViewNotFound,

    #[error("invalid buffer type for descriptor set")]
    InvalidBufferType,
}

pub struct VEDescriptorSet {
    device: Arc<VEDevice>,
    pub set: vk::DescriptorSet,
}

impl VEDescriptorSet {
    pub fn new(
        device: Arc<VEDevice>,
        layout: vk::DescriptorSetLayout,
        pool: &vk::DescriptorPool,
    ) -> Result<VEDescriptorSet, VEDescriptorSetError> {
        let layouts = [layout];
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(*pool)
            .set_layouts(&layouts);
        let set = unsafe { device.device.allocate_descriptor_sets(&info)?[0] };
        Ok(VEDescriptorSet { device, set })
    }

    pub fn bind_image_sampler(
        &self,
        binding: u32,
        image: &VEImage,
        view: vk::ImageView,
        sampler: &VESampler,
    ) -> Result<(), VEDescriptorSetError> {
        let infos = [vk::DescriptorImageInfo::default()
            .image_view(view)
            .sampler(sampler.handle)
            .image_layout(image.current_layout)];
        Ok(self.write(
            vk::WriteDescriptorSet::default()
                .dst_binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&infos),
        ))
    }

    pub fn bind_image_storage(
        &self,
        binding: u32,
        image: &VEImage,
        view: vk::ImageView,
    ) -> Result<(), VEDescriptorSetError> {
        let infos = [vk::DescriptorImageInfo::default()
            .image_view(view)
            .image_layout(image.current_layout)];
        Ok(self.write(
            vk::WriteDescriptorSet::default()
                .dst_binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(&infos),
        ))
    }

    pub fn bind_buffer(&self, binding: u32, buffer: &VEBuffer) -> Result<(), VEDescriptorSetError> {
        let infos = [vk::DescriptorBufferInfo::default()
            .buffer(buffer.buffer)
            .offset(0)
            .range(buffer.size)];
        let is_usage_uniform = buffer.usage.contains(&VEBufferUsage::Uniform);
        let is_usage_storage = buffer.usage.contains(&VEBufferUsage::Storage);
        let typ = if is_usage_uniform {
            Ok(vk::DescriptorType::UNIFORM_BUFFER)
        } else if is_usage_storage {
            Ok(vk::DescriptorType::STORAGE_BUFFER)
        } else {
            Err(VEDescriptorSetError::InvalidBufferType)
        }?;
        Ok(self.write(
            vk::WriteDescriptorSet::default()
                .dst_binding(binding)
                .descriptor_type(typ)
                .buffer_info(&infos),
        ))
    }

    fn write(&self, write: vk::WriteDescriptorSet) {
        let writes = [write.dst_set(self.set)];
        let copies = [];
        unsafe {
            self.device.device.update_descriptor_sets(&writes, &copies);
        }
    }
}
