use crate::core::device::VEDevice;
use ash::vk;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum SemaphoreState {
    Fresh,
    Pending,
    Awaited,
}

#[derive(Debug)]
pub struct VESemaphore {
    device: Arc<VEDevice>,
    pub handle: vk::Semaphore,
    pub state: SemaphoreState,
}

impl VESemaphore {
    pub fn new(device: Arc<VEDevice>) -> VESemaphore {
        let info = vk::SemaphoreCreateInfo::default();
        let handle = unsafe { device.device.create_semaphore(&info, None).unwrap() };

        VESemaphore {
            device,
            handle,
            state: SemaphoreState::Fresh,
        }
    }

    pub fn recreate(&mut self) {
        unsafe {
            self.device.device.destroy_semaphore(self.handle, None);
            let info = vk::SemaphoreCreateInfo::default();
            self.handle = self.device.device.create_semaphore(&info, None).unwrap();
            self.state = SemaphoreState::Fresh;
        }
    }
}

impl Drop for VESemaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_semaphore(self.handle, None);
        }
    }
}
