use crate::device::VEDevice;
use ash::vk;

pub struct VEMainDeviceQueue<'a> {
    device: &'a VEDevice,
    pub main_queue: vk::Queue,
}

impl<'a> VEMainDeviceQueue<'a> {
    pub fn new(device: &'a VEDevice) -> VEMainDeviceQueue {
        VEMainDeviceQueue {
            device,
            main_queue: unsafe { device.device.get_device_queue(device.queue_family_index, 0) },
        }
    }

    pub fn wait_idle(&self) {
        unsafe { self.device.device.queue_wait_idle(self.main_queue).unwrap() }
    }
}
