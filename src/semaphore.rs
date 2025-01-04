use crate::device::VEDevice;
use ash::vk;
use std::sync::Arc;

pub struct VESemaphore {
    device: Arc<VEDevice>,
    pub handle: vk::Semaphore,
}

impl VESemaphore {
    pub fn new(device: Arc<VEDevice>) -> VESemaphore {
        let info = vk::SemaphoreCreateInfo::default();
        let handle = unsafe { device.device.create_semaphore(&info, None).unwrap() };

        VESemaphore { device, handle }
    }
}

impl Drop for VESemaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_semaphore(self.handle, None);
        }
    }
}
