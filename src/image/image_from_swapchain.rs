use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::image::image::{VEImage, VEImageError};
use ash::vk;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

impl VEImage {
    pub fn from_swapchain_present_image(
        device: Arc<VEDevice>,
        queue: Arc<Mutex<VEMainDeviceQueue>>,
        command_pool: Arc<VECommandPool>,

        width: u32,
        height: u32,

        format: vk::Format,
        image_handle: vk::Image,
    ) -> Result<VEImage, VEImageError> {
        let mut image = VEImage {
            device: device.clone(),
            queue: queue.clone(),

            allocation: None,

            width,
            height,
            depth: 1,

            format,

            aspect: vk::ImageAspectFlags::COLOR,

            handle: image_handle,
            views: HashMap::new(),
            current_layout: vk::ImageLayout::UNDEFINED,
        };

        let command_buffer = VECommandBuffer::new(device, command_pool)?;
        //command_buffer.begin(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        command_buffer.begin()?;
        image.transition_layout(
            &command_buffer,
            image.current_layout,
            vk::ImageLayout::PRESENT_SRC_KHR,
        )?;
        command_buffer.end()?;

        let queue = queue.lock().map_err(|_| VEImageError::QueueLockingFailed)?;

        command_buffer.submit(&queue, vec![], vec![])?;
        queue.wait_idle()?;
        Ok(image)
    }
}
