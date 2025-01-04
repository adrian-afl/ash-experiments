use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::core::command_buffer::VECommandBuffer;
use crate::core::device::VEDevice;
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};

pub struct VEVertexBuffer {
    device: Arc<VEDevice>,
    pub buffer: VEBuffer,
    pub vertex_count: u32,
}

impl VEVertexBuffer {
    pub fn new(device: Arc<VEDevice>, buffer: VEBuffer, vertex_count: u32) -> VEVertexBuffer {
        VEVertexBuffer {
            device,
            buffer,
            vertex_count,
        }
    }

    pub fn draw_instanced(&self, command_buffer: &VECommandBuffer, instances: u32) {
        unsafe {
            self.device.device.cmd_bind_vertex_buffers(
                command_buffer.handle,
                0,
                &[self.buffer.buffer],
                &[0],
            );
            self.device
                .device
                .cmd_draw(command_buffer.handle, self.vertex_count, instances, 0, 0);
        }
    }

    pub fn from_file(
        device: Arc<VEDevice>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        path: &str,
        vertex_size_bytes: usize,
    ) -> VEVertexBuffer {
        let mut file = File::open(path).unwrap();
        let metadata = file.metadata().unwrap();
        let file_size = metadata.len() as usize;

        if file_size % vertex_size_bytes != 0 {
            panic!("Mismatch of vertex size in the file, not aligned")
        }

        let vertex_count = file_size / vertex_size_bytes;

        let mut buffer = VEBuffer::new(
            device.clone(),
            VEBufferType::Vertex,
            memory_manager.clone(),
            file_size as vk::DeviceSize,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            let mem = buffer.map() as *mut u8;
            let mut slice = std::slice::from_raw_parts_mut(mem, file_size);
            file.read_exact(&mut slice).unwrap();
            buffer.unmap();
        }

        VEVertexBuffer::new(device.clone(), buffer, vertex_count as u32)
    }
}
