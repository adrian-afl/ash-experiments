use crate::buffer::{VEBuffer, VEBufferType};
use crate::device::VEDevice;
use ash::vk;
use std::sync::Arc;

pub struct VEDescriptorSet {
    device: Arc<VEDevice>,
    pub set: vk::DescriptorSet,
}

impl VEDescriptorSet {
    pub fn new(
        device: Arc<VEDevice>,
        layout: vk::DescriptorSetLayout,
        pool: &vk::DescriptorPool,
    ) -> VEDescriptorSet {
        let layouts = [layout];
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(*pool)
            .set_layouts(&layouts);
        let set = unsafe { device.device.allocate_descriptor_sets(&info).unwrap()[0] };
        VEDescriptorSet { device, set }
    }

    //pub fn bind_image_view_sampler(binding:u32, )//

    pub fn bind_buffer(&self, binding: u32, buffer: &VEBuffer) {
        let buffer_infos = [vk::DescriptorBufferInfo::default()
            .buffer(buffer.buffer)
            .offset(0)
            .range(buffer.size)];
        self.write(
            vk::WriteDescriptorSet::default()
                .dst_binding(binding)
                .descriptor_type(match buffer.typ {
                    VEBufferType::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
                    VEBufferType::Storage => vk::DescriptorType::STORAGE_BUFFER,
                    _ => panic!("Cannot use buffer typ {:?} in a descriptor set", buffer.typ),
                })
                .buffer_info(&buffer_infos),
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
