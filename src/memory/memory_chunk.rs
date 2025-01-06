use std::fmt::{Debug, Formatter, Write};
use crate::core::device::VEDevice;
use ash::vk::{Buffer, DeviceMemory, DeviceSize, Image, MemoryAllocateInfo, MemoryMapFlags};
use std::sync::Arc;
use tracing::instrument;
use crate::image::image::VEImage;

static CHUNK_SIZE: u64 = 256 * 1024 * 1024;

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
}

impl Debug for VEMemoryChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("VEMemoryChunk({})", self.chunk_identifier))
    }
}

impl VEMemoryChunk {
    #[instrument]
    pub fn new(
        device: Arc<VEDevice>,
        chunk_identifier: u64,
        memory_type_index: u32,
    ) -> VEMemoryChunk {
        let handle = unsafe {
            device
                .device
                .allocate_memory(
                    &MemoryAllocateInfo::default()
                        .allocation_size(CHUNK_SIZE)
                        .memory_type_index(memory_type_index),
                    None,
                )
                .unwrap()
        };
        VEMemoryChunk {
            device,
            chunk_identifier,
            allocations: vec![],
            handle,
            identifier_counter: 0,
        }
    }

    #[instrument]
    pub fn free_allocation(&mut self, alloc_identifier: u64) {
        for i in 0..self.allocations.len() {
            if self.allocations[i].alloc_identifier == alloc_identifier {
                self.allocations.remove(i);
                return;
            }
        }
    }

    #[instrument]
    pub fn bind_buffer_memory(
        &mut self,
        buffer: Buffer,
        size: u64,
        offset: u64,
    ) -> VESingleAllocation {
        {
            self.identifier_counter += 1;
        }
        unsafe {
            self.device
                .device
                .bind_buffer_memory(buffer, self.handle, offset as DeviceSize)
                .unwrap()
        }
        self.allocations.push(VESingleAllocation {
            chunk_identifier: self.chunk_identifier,
            alloc_identifier: self.identifier_counter,
            size,
            offset,
        });
        self.allocations.last().unwrap().clone()
    }

    #[instrument]
    pub fn bind_image_memory(
        &mut self,
        image: Image,
        size: u64,
        offset: u64,
    ) -> VESingleAllocation {
        {
            self.identifier_counter += 1;
        }
        unsafe {
            self.device
                .device
                .bind_image_memory(image, self.handle, offset as DeviceSize)
                .unwrap()
        }
        self.allocations.push(VESingleAllocation {
            chunk_identifier: self.chunk_identifier,
            alloc_identifier: self.identifier_counter,
            size,
            offset,
        });
        self.allocations.last().unwrap().clone()
    }

    #[instrument]
    pub fn find_free_memory_offset(&self, size: u64) -> Option<u64> {
        if self.is_free_space(0, size) {
            return Some(0);
        }
        for a in &self.allocations {
            if self.is_free_space(a.offset, a.offset + size) {
                return Some(a.offset + size);
            }
        }
        None
    }

    #[instrument]
    fn is_free_space(&self, offset: u64, size: u64) -> bool {
        let end = offset + size;
        if (end >= CHUNK_SIZE) {
            return false;
        }
        for a in &self.allocations {
            let aend = a.offset + a.size;
            if offset >= a.offset && offset < aend {
                // if start of alloc collides
                return false;
            }
            if end >= a.offset && end < aend {
                // if end of alloc collides
                return false;
            }
            if offset <= a.offset && end > aend {
                // if alloc contains element
                return false;
            }
            if offset >= a.offset && end < aend {
                // if elements contains alloc
                return false;
            }
        }
        true
    }

    #[instrument]
    pub fn map(&self, offset: u64, size: u64) -> *mut core::ffi::c_void {
        unsafe {
            self.device
                .device
                .map_memory(self.handle, offset, size, MemoryMapFlags::default())
                .unwrap()
        }
    }

    #[instrument]
    pub fn unmap(&self) {
        unsafe {
            self.device.device.unmap_memory(self.handle);
        }
    }
}

impl Drop for VEMemoryChunk {
    #[instrument]
    fn drop(&mut self) {
        unsafe { self.device.device.free_memory(self.handle, None) };
    }
}
