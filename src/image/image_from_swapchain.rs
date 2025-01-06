use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::image::image::VEImage;
use crate::memory::memory_chunk::VESingleAllocation;
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use std::sync::{Arc, Mutex};

impl VEImage {
    pub fn from_swapchain_present_image(
        device: Arc<VEDevice>,
        queue: Arc<VEMainDeviceQueue>,
        command_pool: Arc<VECommandPool>,

        width: u32,
        height: u32,

        format: vk::Format,
        image_handle: vk::Image,
    ) -> VEImage {
        let mut image = VEImage {
            device,
            queue,
            command_pool,

            allocation: None,

            width,
            height,
            depth: 1,

            format,

            aspect: vk::ImageAspectFlags::COLOR,

            handle: image_handle,
            view: None,
            current_layout: vk::ImageLayout::UNDEFINED,
        };

        image.transition_layout(image.current_layout, vk::ImageLayout::PRESENT_SRC_KHR);

        image
    }
}
