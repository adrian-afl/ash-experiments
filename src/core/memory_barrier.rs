use crate::core::command_buffer::VECommandBuffer;
use crate::core::device::VEDevice;
use ash::vk;
use std::sync::Arc;

pub struct VEMemoryBarrier {
    pub src_access: vk::AccessFlags,
    pub dst_access: vk::AccessFlags,
}

impl VEMemoryBarrier {
    pub fn build(&self) -> vk::MemoryBarrier {
        vk::MemoryBarrier::default()
            .src_access_mask(self.src_access)
            .dst_access_mask(self.dst_access)
    }
}

pub struct VEImageMemoryBarrier {
    pub image: vk::Image,
    pub aspect: vk::ImageAspectFlags,
    pub old_layout: vk::ImageLayout,
    pub new_layout: vk::ImageLayout,
    pub src_access: vk::AccessFlags,
    pub dst_access: vk::AccessFlags,
}

impl VEImageMemoryBarrier {
    pub fn build(&self) -> vk::ImageMemoryBarrier {
        vk::ImageMemoryBarrier::default()
            .old_layout(self.old_layout)
            .new_layout(self.new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.image)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(self.aspect)
                    .base_mip_level(0)
                    .level_count(1) // TODO mip mapping
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .src_access_mask(self.src_access)
            .dst_access_mask(self.dst_access)
    }
}

pub struct VEBufferMemoryBarrier {
    pub buffer: vk::Buffer,
    pub src_access: vk::AccessFlags,
    pub dst_access: vk::AccessFlags,
    pub offset: u64,
    pub size: u64,
}

impl VEBufferMemoryBarrier {
    pub fn build(&self) -> vk::BufferMemoryBarrier {
        vk::BufferMemoryBarrier::default()
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(self.buffer)
            .src_access_mask(self.src_access)
            .dst_access_mask(self.dst_access)
            .offset(self.offset)
            .size(self.size)
    }
}

pub fn submit_barriers(
    device: &VEDevice,
    command_buffer: &VECommandBuffer,
    source_stage: vk::PipelineStageFlags,
    destination_stage: vk::PipelineStageFlags,
    memory_barriers: &[vk::MemoryBarrier],
    buffer_memory_barriers: &[vk::BufferMemoryBarrier],
    image_memory_barriers: &[vk::ImageMemoryBarrier],
) {
    unsafe {
        device.device.cmd_pipeline_barrier(
            command_buffer.handle,
            source_stage,
            destination_stage,
            vk::DependencyFlags::empty(),
            memory_barriers,
            buffer_memory_barriers,
            image_memory_barriers,
        )
    }
}
