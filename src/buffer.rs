use crate::device::VEDevice;
use crate::memory::memory_chunk::VESingleAllocation;
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk::{
    Buffer, BufferCreateInfo, BufferUsageFlags, DeviceSize, MemoryPropertyFlags, SharingMode,
};
use std::sync::{Arc, Mutex};

pub struct VEBuffer {
    device: Arc<VEDevice>,
    memory_manager: Arc<Mutex<VEMemoryManager>>,
    allocation: VESingleAllocation,
    pub buffer: Buffer,
}

impl VEBuffer {
    pub fn new(
        device: Arc<VEDevice>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        size: DeviceSize,
        usage: BufferUsageFlags,
    ) -> VEBuffer {
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
            let mem_index = device.find_memory_type(
                mem_reqs.memory_type_bits,
                MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            );

            // allocate the memory, in a sub-scope so borrow is dropped
            let allocation = {
                memory_manager
                    .lock()
                    .unwrap()
                    .bind_buffer_memory(mem_index, buffer, size)
            };

            // return the VEBuffer
            VEBuffer {
                device,
                memory_manager, // Problem here: cannot borrow `*memory_manager` as mutable more than once at a time
                buffer,
                allocation,
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
