use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::image::image::VEImage;
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

        format: vk::Format,
        tiling: vk::ImageTiling,

        usage: vk::ImageUsageFlags,

        memory_properties: vk::MemoryPropertyFlags,
    ) -> VEImage {
        println!("format is {:?}", format);
        let mut empty = VEImage::from_full(
            device.clone(),
            queue.clone(),
            command_pool.clone(),
            memory_manager.clone(),
            width,
            height,
            depth,
            format,
            tiling,
            usage | vk::ImageUsageFlags::TRANSFER_DST,
            memory_properties,
        );

        let mut staging_buffer = VEBuffer::new(
            device.clone(),
            VEBufferType::TransferSource,
            memory_manager.clone(),
            data.len() as vk::DeviceSize,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            let mem = staging_buffer.map() as *mut u8;
            std::ptr::copy(data.as_ptr(), mem, data.len());
            staging_buffer.unmap();
        }

        empty.transition_layout(empty.current_layout, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

        let command_buffer = VECommandBuffer::new(device.clone(), command_pool.clone());
        //command_buffer.begin(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        command_buffer.begin(CommandBufferUsageFlags::empty());

        let region = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(empty.aspect)
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
                empty.handle,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        command_buffer.end();

        command_buffer.submit(&queue, &[], &[]);
        queue.wait_idle();

        empty.transition_layout(
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::GENERAL,
        );

        empty
    }
}
