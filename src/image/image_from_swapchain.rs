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
        memory_manager: Arc<Mutex<VEMemoryManager>>,

        width: u32,
        height: u32,

        format: vk::Format,
        image_handle: vk::Image,
        image_view_handle: vk::ImageView,
    ) -> VEImage {
        let mut image = VEImage {
            device,
            queue,
            command_pool,
            memory_manager,

            allocation: VESingleAllocation {
                alloc_identifier: u64::MAX,
                chunk_identifier: u64::MAX,
                size: 0,
                offset: 0,
            },

            width,
            height,
            depth: 1,

            format,
            tiling: vk::ImageTiling::OPTIMAL,

            usage: vk::ImageUsageFlags::TRANSFER_DST,
            aspect: vk::ImageAspectFlags::COLOR,

            handle: image_handle,
            view: image_view_handle,
            current_layout: vk::ImageLayout::UNDEFINED,
        };

        image
    }
}
