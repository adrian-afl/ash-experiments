use crate::core::device::VEDevice;
use ash::vk;
use ash::vk::{Buffer, DeviceMemory, DeviceSize, Image, MemoryAllocateInfo, MemoryMapFlags};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use thiserror::Error;
use tracing::{event, Level};

static CHUNK_SIZE: u64 = 256 * 1024 * 1024;

#[derive(Error, Debug)]
pub enum VEMemoryChunkError {
    #[error("allocation failed")]
    AllocationFailed(#[source] vk::Result),
    #[error("binding buffer memory failed")]
    BindingBufferMemoryFailed(#[source] vk::Result),
    #[error("binding image memory failed")]
    BindingImageMemoryFailed(#[source] vk::Result),
    #[error("mapping failed")]
    MappingFailed(#[source] vk::Result),
    #[error("pointer not found")]
    PointerNotFound,
}

#[derive(Clone, Debug)]
pub struct VESingleAllocation {
    pub alloc_identifier: u64,
    pub chunk_identifier: u64,
    pub size: u64,
    pub offset: u64,
}

pub struct VEMemoryChunk {
    pub chunk_identifier: u64,
    device: Arc<VEDevice>,
    pub allocations: Vec<VESingleAllocation>,
    pub handle: DeviceMemory,
    identifier_counter: u64,
    ptr: Option<*mut core::ffi::c_void>,
}

unsafe impl Send for VEMemoryChunk {}
unsafe impl Sync for VEMemoryChunk {}

impl Debug for VEMemoryChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("VEMemoryChunk({})", self.chunk_identifier))
    }
}

impl VEMemoryChunk {
    pub fn new(
        device: Arc<VEDevice>,
        chunk_identifier: u64,
        memory_type_index: u32,
    ) -> Result<VEMemoryChunk, VEMemoryChunkError> {
        let handle = unsafe {
            device
                .device
                .allocate_memory(
                    &MemoryAllocateInfo::default()
                        .allocation_size(CHUNK_SIZE)
                        .memory_type_index(memory_type_index),
                    None,
                )
                .map_err(VEMemoryChunkError::AllocationFailed)?
        };

        Ok(VEMemoryChunk {
            device,
            chunk_identifier,
            allocations: vec![],
            handle,
            identifier_counter: 0,
            ptr: None,
        })
    }

    pub fn free_allocation(&mut self, alloc_identifier: u64) {
        for i in 0..self.allocations.len() {
            if self.allocations[i].alloc_identifier == alloc_identifier {
                event!(
                    Level::TRACE,
                    "FREEING offset {}!",
                    self.allocations[i].offset
                );
                self.allocations.remove(i);
                return;
            }
        }
    }

    pub fn bind_buffer_memory(
        &mut self,
        buffer: Buffer,
        size: u64,
        offset: u64,
    ) -> Result<VESingleAllocation, VEMemoryChunkError> {
        {
            self.identifier_counter += 1;
        }
        unsafe {
            self.device
                .device
                .bind_buffer_memory(buffer, self.handle, offset as DeviceSize)
                .map_err(VEMemoryChunkError::BindingBufferMemoryFailed)?
        }
        let allocation = VESingleAllocation {
            chunk_identifier: self.chunk_identifier,
            alloc_identifier: self.identifier_counter,
            size,
            offset,
        };
        self.allocations.push(allocation.clone());
        Ok(allocation)
    }

    pub fn bind_image_memory(
        &mut self,
        image: Image,
        size: u64,
        offset: u64,
    ) -> Result<VESingleAllocation, VEMemoryChunkError> {
        {
            self.identifier_counter += 1;
        }
        unsafe {
            self.device
                .device
                .bind_image_memory(image, self.handle, offset as DeviceSize)
                .map_err(VEMemoryChunkError::BindingImageMemoryFailed)?
        }
        let allocation = VESingleAllocation {
            chunk_identifier: self.chunk_identifier,
            alloc_identifier: self.identifier_counter,
            size,
            offset,
        };
        self.allocations.push(allocation.clone());
        Ok(allocation)
    }

    pub fn find_free_memory_offset(&self, size: u64) -> Option<u64> {
        if self.is_free_space(0, size) {
            event!(Level::TRACE, "Zero is free! Amazing");
            return Some(0);
        }
        for a in &self.allocations {
            if self.is_free_space(a.offset + a.size + 0x1000, size) {
                event!(Level::TRACE, "offset {} is free!", a.offset);
                return Some(a.offset + a.size + 0x1000);
            }
        }
        None
    }

    fn is_free_space(&self, offset: u64, size: u64) -> bool {
        // Check for overflow and bounds
        match offset.checked_add(size) {
            None => false,                          // Integer overflow
            Some(end) if end > CHUNK_SIZE => false, // Out of bounds
            Some(end) => {
                // Check for overlap with any existing allocation
                // Two ranges overlap if the start of one range is before the end of the other
                !self.allocations.iter().any(|alloc| {
                    let alloc_end = alloc.offset + alloc.size;
                    offset < alloc_end && alloc.offset < end
                })
            }
        }
    }

    pub fn map(&mut self, offset: u64) -> Result<*mut core::ffi::c_void, VEMemoryChunkError> {
        if self.ptr.is_none() {
            // once mapped, stays mapped
            self.ptr = Some(unsafe {
                self.device
                    .device
                    .map_memory(self.handle, 0, CHUNK_SIZE, MemoryMapFlags::default())
                    .map_err(VEMemoryChunkError::MappingFailed)?
            });
        }

        Ok(unsafe {
            let ptr = self
                .ptr
                .as_mut()
                .ok_or(VEMemoryChunkError::PointerNotFound)?;

            ptr.offset(offset as isize)
        })
    }

    pub fn unmap(&mut self) {
        self.ptr = None;
        unsafe {
            self.device.device.unmap_memory(self.handle);
        }
    }
}

impl Drop for VEMemoryChunk {
    fn drop(&mut self) {
        unsafe { self.device.device.free_memory(self.handle, None) };
    }
}
