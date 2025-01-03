use crate::device::VEDevice;
use ash::vk;

pub struct VECommandPool<'a> {
    device: &'a VEDevice,
    pub handle: vk::CommandPool,
}

impl<'a> VECommandPool<'a> {
    pub fn new(device: &'a VEDevice) -> VECommandPool<'a> {
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
