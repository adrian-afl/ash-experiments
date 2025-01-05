use crate::core::device::VEDevice;
use ash::vk;
use std::sync::Arc;

#[derive(Debug)]
pub struct VEMainDeviceQueue {
    device: Arc<VEDevice>,
    pub main_queue: vk::Queue,
}

impl VEMainDeviceQueue {
    pub fn new(device: Arc<VEDevice>) -> VEMainDeviceQueue {
        VEMainDeviceQueue {
            device: device.clone(),
            main_queue: unsafe { device.device.get_device_queue(device.queue_family_index, 0) },
        }
    }

    pub fn wait_idle(&self) {
        unsafe { self.device.device.queue_wait_idle(self.main_queue).unwrap() }
    }
}
