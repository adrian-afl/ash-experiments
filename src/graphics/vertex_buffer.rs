use crate::buffer::buffer::{VEBuffer, VEBufferError, VEBufferUsage};
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
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

    pub fn from_data(
        device: Arc<VEDevice>,
        queue: Arc<Mutex<VEMainDeviceQueue>>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        data: Vec<u8>,
        vertex_attributes: &[VertexAttribFormat],
    ) -> Result<VEVertexBuffer, VEVertexBufferError> {
        let vertex_size_bytes: u32 = vertex_attributes
            .iter()
            .map(|a| get_vertex_attribute_type_byte_size(a))
            .sum();

        let input_size = data.len() as u32;

        if input_size % vertex_size_bytes != 0 {
            return Err(VEVertexBufferError::VertexSizeMismatch);
        }

        let vertex_count = input_size / vertex_size_bytes;

        let mut staging_buffer = VEBuffer::new(
            device.clone(),
            queue.clone(),
            command_pool.clone(),
            memory_manager.clone(),
            &[VEBufferUsage::TransferSource],
            input_size as vk::DeviceSize,
            Some(VEMemoryProperties::HostCoherent),
        )?;

        let final_buffer = VEBuffer::new(
            device.clone(),
            queue.clone(),
            command_pool.clone(),
            memory_manager.clone(),
            &[VEBufferUsage::Vertex, VEBufferUsage::TransferDestination],
            input_size as vk::DeviceSize,
            Some(VEMemoryProperties::DeviceLocal),
        )?;

        unsafe {
            let mem = staging_buffer.map()? as *mut u8;
            let slice = std::slice::from_raw_parts_mut(mem, input_size as usize);
            slice.copy_from_slice(&data);
            // staging_buffer.unmap()?;
        }

        staging_buffer.copy_to(&final_buffer, 0, 0, staging_buffer.size)?;

        Ok(VEVertexBuffer::new(
            device.clone(),
            final_buffer,
            vertex_count,
        ))
    }

    pub fn from_file(
        device: Arc<VEDevice>,
        queue: Arc<Mutex<VEMainDeviceQueue>>,
        command_pool: Arc<VECommandPool>,
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

        let mut staging_buffer = VEBuffer::new(
            device.clone(),
            queue.clone(),
            command_pool.clone(),
            memory_manager.clone(),
            &[VEBufferUsage::TransferSource],
            file_size as vk::DeviceSize,
            Some(VEMemoryProperties::HostCoherent),
        )?;

        let final_buffer = VEBuffer::new(
            device.clone(),
            queue.clone(),
            command_pool.clone(),
            memory_manager.clone(),
            &[VEBufferUsage::Vertex, VEBufferUsage::TransferDestination],
            file_size as vk::DeviceSize,
            Some(VEMemoryProperties::DeviceLocal),
        )?;

        unsafe {
            let mem = staging_buffer.map()? as *mut u8;
            let mut slice = std::slice::from_raw_parts_mut(mem, file_size as usize);
            file.read_exact(&mut slice)
                .map_err(VEVertexBufferError::ReadingFileFailed)?;
            // staging_buffer.unmap()?;
        }

        staging_buffer.copy_to(&final_buffer, 0, 0, staging_buffer.size)?;

        Ok(VEVertexBuffer::new(
            device.clone(),
            final_buffer,
            vertex_count,
        ))
    }
}
