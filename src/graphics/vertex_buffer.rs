use crate::buffer::buffer::{VEBuffer, VEBufferError, VEBufferType};
use crate::core::device::VEDevice;
use crate::core::memory_properties::VEMemoryProperties;
use crate::graphics::vertex_attributes::{get_vertex_attribute_type_byte_size, VertexAttribFormat};
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use std::fs::File;
use std::io;
use std::io::Read;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEVertexBufferError {
    #[error("opening file failed")]
    OpeningFileFailed(#[source] io::Error),

    #[error("getting file metadata failed")]
    GettingFileMetadataFailed(#[source] io::Error),

    #[error("reading file failed")]
    ReadingFileFailed(#[source] io::Error),

    #[error("vertex size mismatch in the file")]
    VertexSizeMismatch,

    #[error("buffer error")]
    BufferError(#[from] VEBufferError),
}

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
    ) -> Result<VEVertexBuffer, VEVertexBufferError> {
        let vertex_size_bytes: u32 = vertex_attributes
            .iter()
            .map(|a| get_vertex_attribute_type_byte_size(a))
            .sum();

        let mut file = File::open(path).map_err(VEVertexBufferError::OpeningFileFailed)?;
        let metadata = file
            .metadata()
            .map_err(VEVertexBufferError::GettingFileMetadataFailed)?;
        let file_size = metadata.len() as u32;

        if file_size % vertex_size_bytes != 0 {
            return Err(VEVertexBufferError::VertexSizeMismatch);
        }

        let vertex_count = file_size / vertex_size_bytes;

        let mut buffer = VEBuffer::new(
            device.clone(),
            memory_manager.clone(),
            VEBufferType::Vertex,
            file_size as vk::DeviceSize,
            Some(VEMemoryProperties::HostCoherent),
        )?;

        unsafe {
            let mem = buffer.map()? as *mut u8;
            let mut slice = std::slice::from_raw_parts_mut(mem, file_size as usize);
            file.read_exact(&mut slice)
                .map_err(VEVertexBufferError::ReadingFileFailed)?;
            buffer.unmap()?;
        }

        Ok(VEVertexBuffer::new(device.clone(), buffer, vertex_count))
    }
}
