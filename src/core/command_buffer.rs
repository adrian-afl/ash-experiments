use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::semaphore::{SemaphoreState, VESemaphore};
use ash::vk;
use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandBufferUsageFlags,
    PipelineStageFlags,
};
use std::sync::{Arc, Mutex};
use tracing::{event, Level};

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
        wait_for_semaphores: Vec<Arc<Mutex<VESemaphore>>>,
        signal_semaphores: Vec<Arc<Mutex<VESemaphore>>>,
    ) {
        let wait_handles: Vec<vk::Semaphore> = wait_for_semaphores
            .iter()
            .filter(|x| {
                let x = x.lock().unwrap();
                match x.state {
                    SemaphoreState::Fresh => false,
                    SemaphoreState::Pending => true,
                    SemaphoreState::Awaited => panic!("Waiting for awaited semaphore"),
                }
            })
            .map(|mut x| {
                let x = x.lock().unwrap();
                x.handle
            })
            .collect();

        let wait_masks: Vec<PipelineStageFlags> = wait_for_semaphores
            .iter()
            .filter(|x| {
                let x = x.lock().unwrap();
                match x.state {
                    SemaphoreState::Fresh => false,
                    SemaphoreState::Pending => true,
                    SemaphoreState::Awaited => panic!("Waiting for awaited semaphore"),
                }
            })
            .map(|_| {
                PipelineStageFlags::ALL_COMMANDS
                    | PipelineStageFlags::ALL_GRAPHICS
                    | PipelineStageFlags::COMPUTE_SHADER
            })
            .collect();

        for semaphore in wait_for_semaphores.iter() {
            let mut semaphore = semaphore.lock().unwrap();
            if (semaphore).state == SemaphoreState::Pending {
                event!(Level::TRACE, "Setting semaphore to Awaited");
                (semaphore).state = SemaphoreState::Awaited;
            }
        }

        let signal_handles: Vec<vk::Semaphore> = signal_semaphores
            .iter()
            .map(|mut x| {
                let x = x.lock().unwrap();
                x.handle
            })
            .collect();

        for mut semaphore in signal_semaphores.iter() {
            let mut semaphore = semaphore.lock().unwrap();
            event!(Level::TRACE, "Setting semaphore to Pending");
            (semaphore).state = SemaphoreState::Pending;
        }

        let command_buffer_handles = [self.handle];

        // println!(
        //     "SUBMIT Wait For {:?}, Signal {:?}",
        //     wait_handles, signal_handles
        // );
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
