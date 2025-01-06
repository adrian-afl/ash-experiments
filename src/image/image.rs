use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::image::transition_image_layout::transition_image_layout;
use crate::memory::memory_chunk::VESingleAllocation;
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use tracing::instrument;

#[path = "./image_from_data.rs"]
mod image_from_data;
#[path = "./image_from_file.rs"]
mod image_from_file;
#[path = "./image_from_full.rs"]
mod image_from_full;
#[path = "./image_from_swapchain.rs"]
mod image_from_swapchain;

#[derive(Clone)]
pub struct VEImage {
    device: Arc<VEDevice>,
    queue: Arc<VEMainDeviceQueue>,
    command_pool: Arc<VECommandPool>,
    memory_manager: Arc<Mutex<VEMemoryManager>>,

    pub width: u32,
    pub height: u32,
    pub depth: u32,

    pub format: vk::Format,
    tiling: vk::ImageTiling,

    usage: vk::ImageUsageFlags,
    aspect: vk::ImageAspectFlags,

    pub current_layout: vk::ImageLayout,

    allocation: VESingleAllocation,
    pub handle: vk::Image,
    pub view: Option<vk::ImageView>,
}

impl Debug for VEImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("VEImage")
    }
}

impl VEImage {
    pub fn is_depth(&self) -> bool {
        self.format == vk::Format::D16_UNORM || self.format == vk::Format::D32_SFLOAT
    }

    pub fn transition_layout(&mut self, from_layout: vk::ImageLayout, to_layout: vk::ImageLayout) {
        transition_image_layout(
            self.device.clone(),
            self.command_pool.clone(),
            self.queue.clone(),
            self.handle,
            self.aspect,
            from_layout,
            to_layout,
        );

        self.current_layout = to_layout;
    }
}

// impl Drop for VEImage {
//     fn drop(&mut self) {
//         unsafe {
//             if let Some(view) = self.view {
//                 self.device.device.destroy_image_view(view, None);
//             }
//             self.device.device.destroy_image(self.handle, None);
//         }
//     }
// }
