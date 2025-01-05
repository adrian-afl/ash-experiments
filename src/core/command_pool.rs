use crate::core::device::VEDevice;
use ash::vk;
use std::sync::Arc;

#[derive(Debug)]
pub struct VECommandPool {
    device: Arc<VEDevice>,
    pub handle: vk::CommandPool,
}

impl VECommandPool {
    pub fn new(device: Arc<VEDevice>) -> VECommandPool {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.queue_family_index);
        let pool = unsafe { device.device.create_command_pool(&pool_info, None).unwrap() };

        VECommandPool {
            device,
            handle: pool,
        }
    }
}
