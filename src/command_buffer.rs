use crate::command_pool::VECommandPool;
use crate::device::VEDevice;
use ash::vk::{CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel};
use std::sync::Arc;

pub struct VECommandBuffer {
    device: Arc<VEDevice>,
    pub handle: CommandBuffer,
}

impl<'a> VECommandBuffer {
    pub fn new(device: Arc<VEDevice>, command_pool: &VECommandPool) -> VECommandBuffer {
        VECommandBuffer {
            device: device.clone(),
            handle: unsafe {
                device
                    .device
                    .allocate_command_buffers(
                        &CommandBufferAllocateInfo::default()
                            .level(CommandBufferLevel::PRIMARY)
                            .command_buffer_count(1)
                            .command_pool(command_pool.handle),
                    )
                    .unwrap()[0]
            },
        }
    }
}
