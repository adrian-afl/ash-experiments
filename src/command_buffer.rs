use crate::command_pool::VECommandPool;
use crate::device::VEDevice;
use crate::main_device_queue::VEMainDeviceQueue;
use crate::semaphore::VESemaphore;
use ash::vk;
use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandBufferUsageFlags,
    PipelineStageFlags,
};
use std::sync::Arc;

pub struct VECommandBuffer {
    device: Arc<VEDevice>,
    command_pool: Arc<VECommandPool>,
    pub handle: CommandBuffer,
}

impl<'a> VECommandBuffer {
    pub fn new(device: Arc<VEDevice>, command_pool: Arc<VECommandPool>) -> VECommandBuffer {
        VECommandBuffer {
            device: device.clone(),
            command_pool: command_pool.clone(),
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

    pub fn begin(&self, flags: CommandBufferUsageFlags) {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(flags | CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.device
                .device
                .begin_command_buffer(self.handle, &begin_info)
                .unwrap();
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device.device.end_command_buffer(self.handle).unwrap();
        }
    }

    pub fn submit(
        &self,
        queue: &VEMainDeviceQueue,
        wait_for_semaphores: &[&VESemaphore],
        signal_semaphores: &[&VESemaphore],
    ) {
        let wait_handles: Vec<vk::Semaphore> =
            wait_for_semaphores.iter().map(|x| x.handle).collect();

        let signal_handles: Vec<vk::Semaphore> =
            signal_semaphores.iter().map(|x| x.handle).collect();

        let wait_masks: Vec<PipelineStageFlags> = wait_for_semaphores
            .iter()
            .map(|_| {
                PipelineStageFlags::ALL_COMMANDS
                    | PipelineStageFlags::ALL_GRAPHICS
                    | PipelineStageFlags::COMPUTE_SHADER
            })
            .collect();

        let command_buffer_handles = [self.handle];

        let submit_info = vk::SubmitInfo::default()
            .signal_semaphores(&signal_handles)
            .wait_semaphores(&wait_handles)
            .wait_dst_stage_mask(&wait_masks)
            .command_buffers(&command_buffer_handles);

        unsafe {
            self.device
                .device
                .queue_submit(queue.main_queue, &[submit_info], vk::Fence::null())
                .unwrap();
        }
    }
}

impl Drop for VECommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .free_command_buffers(self.command_pool.handle, &[self.handle])
        };
    }
}
