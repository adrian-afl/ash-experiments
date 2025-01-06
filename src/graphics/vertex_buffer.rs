use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::core::command_buffer::VECommandBuffer;
use crate::core::device::VEDevice;
use crate::graphics::vertex_attributes::{get_vertex_attribute_type_byte_size, VertexAttribFormat};
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

    pub fn from_file(
        device: Arc<VEDevice>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        path: &str,
        vertex_attributes: &[VertexAttribFormat],
    ) -> VEVertexBuffer {
        let vertex_size_bytes: u32 = vertex_attributes
            .iter()
            .map(|a| get_vertex_attribute_type_byte_size(a))
            .sum();

        let mut file = File::open(path).unwrap();
        let metadata = file.metadata().unwrap();
        let file_size = metadata.len() as u32;

        if file_size % vertex_size_bytes != 0 {
            panic!("Mismatch of vertex size in the file, not aligned")
        }

        let vertex_count = file_size / vertex_size_bytes;

        let mut buffer = VEBuffer::new(
            device.clone(),
            memory_manager.clone(),
            VEBufferType::Vertex,
            file_size as vk::DeviceSize,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            let mem = buffer.map() as *mut u8;
            let mut slice = std::slice::from_raw_parts_mut(mem, file_size as usize);
            file.read_exact(&mut slice).unwrap();
            buffer.unmap();
        }

        VEVertexBuffer::new(device.clone(), buffer, vertex_count as u32)
    }
}
