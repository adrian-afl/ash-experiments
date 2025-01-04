use crate::buffer::{VEBuffer, VEBufferType};
use crate::device::VEDevice;
use crate::memory::memory_manager::VEMemoryManager;
use crate::vertex_buffer::VEVertexBuffer;
use ash::vk;
use ash::vk::DeviceSize;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};

// TODO if this works then it could be in vertex buffer impl
pub fn load_vertex_buffer_from_file(
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
        file_size as DeviceSize,
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
