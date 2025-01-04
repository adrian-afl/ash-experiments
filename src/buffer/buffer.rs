use crate::core::device::VEDevice;
use crate::memory::memory_chunk::VESingleAllocation;
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use ash::vk::{Buffer, BufferCreateInfo, DeviceSize, MemoryPropertyFlags, SharingMode};
use std::sync::{Arc, Mutex};

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
        typ: VEBufferType,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        size: DeviceSize,
        memory_properties: MemoryPropertyFlags,
    ) -> VEBuffer {
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
                .unwrap();

            let mem_reqs = device.device.get_buffer_memory_requirements(buffer);
            let mem_index = device.find_memory_type(mem_reqs.memory_type_bits, memory_properties);

            let allocation = {
                memory_manager
                    .lock()
                    .unwrap()
                    .bind_buffer_memory(mem_index, buffer, mem_reqs.size)
            };

            VEBuffer {
                device,
                memory_manager,
                buffer,
                allocation,
                size,
                typ,
            }
        }
    }

    pub fn map(&mut self) -> *mut core::ffi::c_void {
        self.memory_manager.lock().unwrap().map(&self.allocation)
    }

    pub fn unmap(&mut self) {
        self.memory_manager.lock().unwrap().unmap(&self.allocation)
    }
}

impl Drop for VEBuffer {
    fn drop(&mut self) {
        self.memory_manager
            .lock()
            .unwrap()
            .free_allocation(&self.allocation);
    }
}
