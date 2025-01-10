use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::core::device::VEDevice;
use crate::image::image::VEImage;
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
        sampler: &VESampler,
    ) -> Result<(), VEDescriptorSetError> {
        let infos = [vk::DescriptorImageInfo::default()
            .image_view(image.view.ok_or(VEDescriptorSetError::ImageViewNotFound)?)
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
    ) -> Result<(), VEDescriptorSetError> {
        let infos = [vk::DescriptorImageInfo::default()
            .image_view(image.view.ok_or(VEDescriptorSetError::ImageViewNotFound)?)
            .image_layout(image.current_layout)];
        Ok(self.write(
            vk::WriteDescriptorSet::default()
                .dst_binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(&infos),
        ))
    }

    pub fn bind_buffer(&self, binding: u32, buffer: &VEBuffer) {
        let infos = [vk::DescriptorBufferInfo::default()
            .buffer(buffer.buffer)
            .offset(0)
            .range(buffer.size)];
        self.write(
            vk::WriteDescriptorSet::default()
                .dst_binding(binding)
                .descriptor_type(match buffer.typ {
                    VEBufferType::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
                    VEBufferType::Storage => vk::DescriptorType::STORAGE_BUFFER,
                    _ => panic!(
                        "Cannot use buffer type {:?} in a descriptor set",
                        buffer.typ
                    ),
                })
                .buffer_info(&infos),
        )
    }

    fn write(&self, write: vk::WriteDescriptorSet) {
        let writes = [write.dst_set(self.set)];
        let copies = [];
        unsafe {
            self.device.device.update_descriptor_sets(&writes, &copies);
        }
    }
}
