use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::memory_properties::VEMemoryProperties;
use crate::image::image::{VEImage, VEImageError, VEImageUsage};
use crate::image::image_format::VEImageFormat;
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use ash::vk::CommandBufferUsageFlags;
use std::sync::{Arc, Mutex};

impl VEImage {
    pub fn from_data(
        device: Arc<VEDevice>,
        queue: Arc<VEMainDeviceQueue>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,

        data: Vec<u8>,

        width: u32,
        height: u32,
        depth: u32,

        format: VEImageFormat,

        usages: &[VEImageUsage],

        memory_properties: Option<VEMemoryProperties>,
    ) -> Result<VEImage, VEImageError> {
        let mut usages = usages.to_vec();
        usages.push(VEImageUsage::TransferDestination);
        let mut result = VEImage::from_full(
            device.clone(),
            queue.clone(),
            command_pool.clone(),
            memory_manager.clone(),
            width,
            height,
            depth,
            format,
            usages.as_slice(),
            memory_properties,
        )?;

        let mut staging_buffer = VEBuffer::new(
            device.clone(),
            memory_manager.clone(),
            VEBufferType::TransferSource,
            data.len() as vk::DeviceSize,
            Some(VEMemoryProperties::HostCoherent),
        )?;

        unsafe {
            let mem = staging_buffer.map()? as *mut u8;
            std::ptr::copy(data.as_ptr(), mem, data.len());
            staging_buffer.unmap()?;
        }

        result.transition_layout(result.current_layout, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

        let command_buffer = VECommandBuffer::new(device.clone(), command_pool.clone())?;
        //command_buffer.begin(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        command_buffer.begin(CommandBufferUsageFlags::empty())?;

        let region = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(result.aspect)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_offset(vk::Offset3D::default())
            .image_extent(
                vk::Extent3D::default()
                    .width(width)
                    .height(height)
                    .depth(depth),
            );

        unsafe {
            device.device.cmd_copy_buffer_to_image(
                command_buffer.handle,
                staging_buffer.buffer,
                result.handle,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        command_buffer.end()?;

        command_buffer.submit(&queue, vec![], vec![])?;
        queue.wait_idle()?;

        result.transition_layout(
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::GENERAL,
        );

        Ok(result)
    }
}
