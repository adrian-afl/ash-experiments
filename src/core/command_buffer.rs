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
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VECommandBufferError {
    #[error("creation failed")]
    CreationFailed(#[source] vk::Result),

    #[error("begin failed")]
    BeginFailed(#[source] vk::Result),

    #[error("end failed")]
    EndFailed(#[source] vk::Result),

    #[error("submit failed")]
    SubmitFailed(#[source] vk::Result),

    #[error("semaphore locking failed")]
    SemaphoreLockingFailed,

    #[error("waiting for awaited semaphore")]
    WaitingForAwaitedSemaphore,
}

pub struct VECommandBuffer {
    device: Arc<VEDevice>,
    command_pool: Arc<VECommandPool>,
    pub handle: CommandBuffer,
}

impl VECommandBuffer {
    pub fn new(
        device: Arc<VEDevice>,
        command_pool: Arc<VECommandPool>,
    ) -> Result<VECommandBuffer, VECommandBufferError> {
        let handle = unsafe {
            device
                .device
                .allocate_command_buffers(
                    &CommandBufferAllocateInfo::default()
                        .level(CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1)
                        .command_pool(command_pool.handle),
                )
                .map_err(VECommandBufferError::CreationFailed)?[0]
        };
        Ok(VECommandBuffer {
            device: device.clone(),
            command_pool: command_pool.clone(),
            handle,
        })
    }

    pub fn begin_with_flags(
        &self,
        flags: CommandBufferUsageFlags,
    ) -> Result<(), VECommandBufferError> {
        let begin_info = vk::CommandBufferBeginInfo::default().flags(flags);

        unsafe {
            self.device
                .device
                .begin_command_buffer(self.handle, &begin_info)
                .map_err(VECommandBufferError::BeginFailed)?;
        }

        Ok(())
    }

    pub fn begin(&self) -> Result<(), VECommandBufferError> {
        let begin_info =
            vk::CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::empty());

        unsafe {
            self.device
                .device
                .begin_command_buffer(self.handle, &begin_info)
                .map_err(VECommandBufferError::BeginFailed)?;
        }

        Ok(())
    }

    pub fn end(&self) -> Result<(), VECommandBufferError> {
        unsafe {
            self.device
                .device
                .end_command_buffer(self.handle)
                .map_err(VECommandBufferError::EndFailed)?;
        }
        Ok(())
    }

    pub fn submit(
        &self,
        queue: &VEMainDeviceQueue,
        wait_for_semaphores: Vec<Arc<Mutex<VESemaphore>>>,
        signal_semaphores: Vec<Arc<Mutex<VESemaphore>>>,
    ) -> Result<(), VECommandBufferError> {
        let mut wait_handles: Vec<vk::Semaphore> = vec![];
        let mut wait_masks: Vec<PipelineStageFlags> = vec![];
        for x in wait_for_semaphores {
            let mut x = x
                .lock()
                .map_err(|_| VECommandBufferError::SemaphoreLockingFailed)?;
            let should = match x.state {
                SemaphoreState::Fresh => Ok(false),
                SemaphoreState::Pending => Ok(true),
                SemaphoreState::Awaited => Err(VECommandBufferError::WaitingForAwaitedSemaphore),
            }?;
            if should {
                wait_handles.push(x.handle);
                wait_masks.push(
                    PipelineStageFlags::ALL_COMMANDS
                        | PipelineStageFlags::ALL_GRAPHICS
                        | PipelineStageFlags::COMPUTE_SHADER,
                );
                if x.state == SemaphoreState::Pending {
                    x.state = SemaphoreState::Awaited;
                }
            }
        }

        let mut signal_handles: Vec<vk::Semaphore> = vec![];

        for x in signal_semaphores {
            let mut x = x
                .lock()
                .map_err(|_| VECommandBufferError::SemaphoreLockingFailed)?;
            signal_handles.push(x.handle);
            x.state = SemaphoreState::Pending;
        }

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
                .map_err(VECommandBufferError::SubmitFailed)?;
        }
        Ok(())
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
