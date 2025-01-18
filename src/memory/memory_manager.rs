use crate::core::device::VEDevice;
use crate::memory::memory_chunk::{VEMemoryChunk, VEMemoryChunkError, VESingleAllocation};
use ash::vk::{Buffer, Image};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEMemoryManagerError {
    #[error("no allocation found to map")]
    NoAllocationFoundToMap,

    #[error("no allocation found to map")]
    NoAllocationFoundToUnmap,

    #[error("no allocation found to free")]
    NoAllocationFoundToFree,

    #[error("memory already mapped")]
    MemoryAlreadyMapped,

    #[error("mapping failed")]
    MappingFailed(#[from] VEMemoryChunkError),
}

pub struct VEMemoryManager {
    device: Arc<VEDevice>,
    chunks: HashMap<u32, Vec<VEMemoryChunk>>,
    identifier_counter: u64,
    mapped: bool,
}

impl Debug for VEMemoryManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("VEMemoryManager")
    }
}

impl VEMemoryManager {
    pub fn new(device: Arc<VEDevice>) -> VEMemoryManager {
        VEMemoryManager {
            device,
            chunks: HashMap::new(),
            identifier_counter: 0,
            mapped: false,
        }
    }

    pub fn bind_buffer_memory(
        &mut self,
        memory_type_index: u32,
        buffer: Buffer,
        size: u64,
    ) -> Result<VESingleAllocation, VEMemoryChunkError> {
        let free = self.find_free(memory_type_index, size)?;
        free.0.bind_buffer_memory(buffer, size, free.1)
    }

    pub fn bind_image_memory(
        &mut self,
        memory_type_index: u32,
        image: Image,
        size: u64,
    ) -> Result<VESingleAllocation, VEMemoryChunkError> {
        let free = self.find_free(memory_type_index, size)?;
        free.0.bind_image_memory(image, size, free.1)
    }

    fn find_free(
        &mut self,
        memory_type_index: u32,
        size: u64,
    ) -> Result<(&mut VEMemoryChunk, u64), VEMemoryChunkError> {
        let chunks_for_type = self.chunks.entry(memory_type_index).or_default();

        for i in 0..chunks_for_type.len() {
            if let Some(offset) = chunks_for_type[i].find_free_memory_offset(size) {
                println!("Free memory found!");
                return Ok((&mut chunks_for_type[i], offset));
            }
        }
        println!("Free memory NOT found!");

        // no suitable chunk found, allocate
        self.identifier_counter += 1;
        let chunk = VEMemoryChunk::new(
            self.device.clone(),
            self.identifier_counter,
            memory_type_index,
        )?;
        chunks_for_type.push(chunk);
        let last_index = chunks_for_type.len() - 1;
        // honestly, i dont know why rust allows this
        Ok((&mut chunks_for_type[last_index], 0))
    }

    pub fn map(
        &mut self,
        allocation: &VESingleAllocation,
    ) -> Result<*mut core::ffi::c_void, VEMemoryManagerError> {
        if self.mapped {
            // this is to work around the limitation of memory chunks
            return Err(VEMemoryManagerError::MemoryAlreadyMapped);
        }
        for chunks_for_type in self.chunks.values() {
            for chunk in chunks_for_type {
                if chunk.chunk_identifier == allocation.chunk_identifier {
                    self.mapped = true;
                    return chunk
                        .map(allocation.offset, allocation.size)
                        .map_err(VEMemoryManagerError::MappingFailed);
                }
            }
        }
        Err(VEMemoryManagerError::NoAllocationFoundToMap)
    }

    pub fn unmap(&mut self, allocation: &VESingleAllocation) -> Result<(), VEMemoryManagerError> {
        for chunks_for_type in self.chunks.values() {
            for chunk in chunks_for_type {
                if chunk.chunk_identifier == allocation.chunk_identifier {
                    self.mapped = false;
                    chunk.unmap();
                    return Ok(());
                }
            }
        }
        Err(VEMemoryManagerError::NoAllocationFoundToUnmap)
    }

    pub fn free_allocation(
        &mut self,
        allocation: &VESingleAllocation,
    ) -> Result<(), VEMemoryManagerError> {
        for chunks_for_type in self.chunks.values_mut() {
            for i in 0..chunks_for_type.len() {
                if chunks_for_type[i].chunk_identifier == allocation.chunk_identifier {
                    chunks_for_type[i].free_allocation(allocation.alloc_identifier);
                    return Ok(());
                }
            }
        }
        Err(VEMemoryManagerError::NoAllocationFoundToFree)
    }
}
