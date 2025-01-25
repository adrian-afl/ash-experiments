use crate::core::command_buffer::{VECommandBuffer, VECommandBufferError};
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::{VEMainDeviceQueue, VEMainDeviceQueueError};
use crate::core::memory_properties::{get_memory_properties_flags, VEMemoryProperties};
use crate::memory::memory_chunk::{VEMemoryChunkError, VESingleAllocation};
use crate::memory::memory_manager::{VEMemoryManager, VEMemoryManagerError};
use ash::vk;
use ash::vk::{Buffer, BufferCreateInfo, CommandBufferUsageFlags, SharingMode};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEBufferError {
    #[error("creation failed")]
    CreationFailed(#[from] vk::Result),

    #[error("binding buffer memory failed")]
    BindingBufferMemoryFailed(#[from] VEMemoryChunkError),

    #[error("locking memory manager mutex failed")]
    LockingMemoryManagerFailed,

    #[error("mapping failed")]
    MemoryManagerError(#[from] VEMemoryManagerError),

    #[error("main device query error")]
    MainDeviceQueueError(#[from] VEMainDeviceQueueError),

    #[error("command buffer error")]
    CommandBufferError(#[from] VECommandBufferError),

    #[error("no suitable memory type found")]
    NoSuitableMemoryTypeFound,

    #[error("queue locking failed")]
    QueueLockingFailed,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VEBufferUsage {
    Uniform,
    Storage,
    TransferSource,
    TransferDestination,
    Vertex,
}

pub struct VEBuffer {
    device: Arc<VEDevice>,
    queue: Arc<Mutex<VEMainDeviceQueue>>,
    command_pool: Arc<VECommandPool>,
    memory_manager: Arc<Mutex<VEMemoryManager>>,
    allocation: VESingleAllocation,
    pub buffer: Buffer,
    pub size: u64,
    pub usage: Vec<VEBufferUsage>,
}

fn get_buffer_usage_flags(usages: &[VEBufferUsage]) -> vk::BufferUsageFlags {
    let mut flags = vk::BufferUsageFlags::empty();
    for usage in usages {
        match usage {
            VEBufferUsage::Uniform => flags = flags | vk::BufferUsageFlags::UNIFORM_BUFFER,
            VEBufferUsage::Storage => flags = flags | vk::BufferUsageFlags::STORAGE_BUFFER,
            VEBufferUsage::TransferSource => flags = flags | vk::BufferUsageFlags::TRANSFER_SRC,
            VEBufferUsage::TransferDestination => {
                flags = flags | vk::BufferUsageFlags::TRANSFER_DST
            }
            VEBufferUsage::Vertex => flags = flags | vk::BufferUsageFlags::VERTEX_BUFFER,
        }
    }
    flags
}

impl VEBuffer {
    pub fn new(
        device: Arc<VEDevice>,
        queue: Arc<Mutex<VEMainDeviceQueue>>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        usage: &[VEBufferUsage],
        size: u64,
        memory_properties: Option<VEMemoryProperties>,
    ) -> Result<VEBuffer, VEBufferError> {
        unsafe {
            let buffer = device
                .device
                .create_buffer(
                    &BufferCreateInfo::default()
                        .size(size)
                        .usage(get_buffer_usage_flags(usage))
                        .sharing_mode(SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(VEBufferError::CreationFailed)?;

            let mem_reqs = device.device.get_buffer_memory_requirements(buffer);
            let mem_index = device.find_memory_type(
                mem_reqs.memory_type_bits,
                get_memory_properties_flags(memory_properties),
            );

            let allocation = match mem_index {
                None => return Err(VEBufferError::NoSuitableMemoryTypeFound),
                Some(mem_index) => memory_manager
                    .lock()
                    .map_err(|_| VEBufferError::LockingMemoryManagerFailed)?
                    .bind_buffer_memory(mem_index, buffer, mem_reqs.size)?,
            };

            Ok(VEBuffer {
                device,
                queue,
                command_pool,
                memory_manager,
                buffer,
                allocation,
                size,
                usage: usage.to_vec(),
            })
        }
    }

    pub fn map(&mut self) -> Result<*mut core::ffi::c_void, VEBufferError> {
        self.memory_manager
            .lock()
            .map_err(|_| VEBufferError::LockingMemoryManagerFailed)?
            .map(&self.allocation)
            .map_err(VEBufferError::MemoryManagerError)
    }

    pub fn unmap(&mut self) -> Result<(), VEBufferError> {
        self.memory_manager
            .lock()
            .map_err(|_| VEBufferError::LockingMemoryManagerFailed)?
            .unmap(&self.allocation)
            .map_err(VEBufferError::MemoryManagerError)
    }

    pub fn copy_to(
        &self,
        target: &VEBuffer,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) -> Result<(), VEBufferError> {
        let command_buffer = VECommandBuffer::new(self.device.clone(), self.command_pool.clone())?;
        //command_buffer.begin(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        command_buffer.begin(CommandBufferUsageFlags::empty())?;

        let region = vk::BufferCopy::default()
            .size(size)
            .src_offset(src_offset)
            .dst_offset(dst_offset);

        unsafe {
            self.device.device.cmd_copy_buffer(
                command_buffer.handle,
                self.buffer,
                target.buffer,
                &[region],
            );
        }

        command_buffer.end()?;

        let queue = &self
            .queue
            .lock()
            .map_err(|_| VEBufferError::QueueLockingFailed)?;

        command_buffer.submit(queue, vec![], vec![])?;
        queue.wait_idle()?;

        Ok(())
    }
}

impl Drop for VEBuffer {
    fn drop(&mut self) {
        let locking_result = self.memory_manager.lock();
        match locking_result {
            Ok(mut mem) => match { mem.free_allocation(&self.allocation) } {
                Ok(_) => (),
                Err(_) => {}
            },
            Err(_) => {
                panic!("Locking memory manager failed")
            }
        }
    }
}
