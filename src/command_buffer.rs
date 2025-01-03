use crate::command_pool::VECommandPool;
use crate::device::VEDevice;
use ash::vk::{CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel};

pub struct VECommandBuffer<'a> {
    device: &'a VEDevice,
    pub handle: CommandBuffer,
}

impl<'a> VECommandBuffer<'a> {
    pub fn new(device: &'a VEDevice, command_pool: &VECommandPool) -> VECommandBuffer<'a> {
        VECommandBuffer {
            device,
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
