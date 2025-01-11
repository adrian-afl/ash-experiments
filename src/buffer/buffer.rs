use crate::core::device::VEDevice;
use crate::core::memory_properties::{get_memory_properties_flags, VEMemoryProperties};
use crate::memory::memory_chunk::{VEMemoryChunkError, VESingleAllocation};
use crate::memory::memory_manager::{VEMemoryManager, VEMemoryManagerError};
use ash::vk;
use ash::vk::{Buffer, BufferCreateInfo, SharingMode};
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

    #[error("no suitable memory type found")]
    NoSuitableMemoryTypeFound,
}

#[derive(Debug)]
pub enum VEBufferType {
    Uniform,
    Storage,
    TransferSource,
    TransferDestination,
    Vertex,
}

pub struct VEBuffer {
    device: Arc<VEDevice>,
    memory_manager: Arc<Mutex<VEMemoryManager>>,
    allocation: VESingleAllocation,
    pub buffer: Buffer,
    pub size: u64,
    pub typ: VEBufferType,
}

impl VEBuffer {
    pub fn new(
        device: Arc<VEDevice>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        typ: VEBufferType,
        size: u64,
        memory_properties: Option<VEMemoryProperties>,
    ) -> Result<VEBuffer, VEBufferError> {
        let usage = match typ {
            VEBufferType::Uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
            VEBufferType::Storage => vk::BufferUsageFlags::STORAGE_BUFFER,
            VEBufferType::TransferSource => vk::BufferUsageFlags::TRANSFER_SRC,
            VEBufferType::TransferDestination => vk::BufferUsageFlags::TRANSFER_DST,
            VEBufferType::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
        };
        unsafe {
            let buffer = device
                .device
                .create_buffer(
                    &BufferCreateInfo::default()
                        .size(size)
                        .usage(usage)
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
                memory_manager,
                buffer,
                allocation,
                size,
                typ,
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
}

impl Drop for VEBuffer {
    fn drop(&mut self) {
        let mut locking_result = self.memory_manager.lock();
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
