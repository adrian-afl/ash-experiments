use crate::device::VEDevice;
use crate::memory::memory_chunk::{VEMemoryChunk, VESingleAllocation};
use ash::vk::{Buffer, Image};
use std::collections::HashMap;
use std::sync::Arc;

pub struct VEMemoryManager {
    device: Arc<VEDevice>,
    chunks: HashMap<u32, Vec<VEMemoryChunk>>,
    identifier_counter: u64,
}

impl VEMemoryManager {
    pub fn new(device: Arc<VEDevice>) -> VEMemoryManager {
        VEMemoryManager {
            device,
            chunks: HashMap::new(),
            identifier_counter: 0,
        }
    }

    pub fn bind_buffer_memory(
        &mut self,
        memory_type_index: u32,
        buffer: Buffer,
        size: u64,
    ) -> (VESingleAllocation) {
        let free = self.find_free(memory_type_index, size);
        free.0.bind_buffer_memory(buffer, size, free.1)
    }

    pub fn bind_image_memory(
        &mut self,
        memory_type_index: u32,
        image: Image,
        size: u64,
    ) -> VESingleAllocation {
        let free = self.find_free(memory_type_index, size);
        free.0.bind_image_memory(image, size, free.1)
    }

    fn find_free(&mut self, memory_type_index: u32, size: u64) -> (&mut VEMemoryChunk, u64) {
        if (!self.chunks.contains_key(&memory_type_index)) {
            self.chunks.insert(memory_type_index, vec![]);
        }
        let chunks_for_type = self.chunks.get_mut(&memory_type_index).unwrap();

        for i in 0..chunks_for_type.len() {
            match chunks_for_type[i].find_free_memory_offset(size) {
                Some(offset) => return (&mut chunks_for_type[i], offset),
                None => (),
            }
        }

        // no suitable chunk found, allocate
        self.identifier_counter += 1;
        let chunk = VEMemoryChunk::new(
            self.device.clone(),
            self.identifier_counter,
            memory_type_index,
        );
        chunks_for_type.push(chunk);
        (chunks_for_type.last_mut().unwrap(), 0)
    }

    pub fn map(&mut self, allocation: &VESingleAllocation) -> *mut core::ffi::c_void {
        for chunks_for_type in self.chunks.values() {
            for chunk in chunks_for_type {
                if chunk.chunk_identifier == allocation.chunk_identifier {
                    return chunk.map(allocation.offset, allocation.size);
                }
            }
        }
        panic!("No allocation found")
    }

    pub fn unmap(&mut self, allocation: &VESingleAllocation) {
        for chunks_for_type in self.chunks.values() {
            for chunk in chunks_for_type {
                if chunk.chunk_identifier == allocation.chunk_identifier {
                    return chunk.unmap();
                }
            }
        }
        panic!("No allocation found")
    }

    pub fn free_allocation(&mut self, allocation: &VESingleAllocation) {
        for chunks_for_type in self.chunks.values_mut() {
            for i in 0..chunks_for_type.len() {
                if chunks_for_type[i].chunk_identifier == allocation.chunk_identifier {
                    return chunks_for_type[i].free_allocation(allocation.alloc_identifier);
                }
            }
        }
        panic!("No allocation found")
    }
}
