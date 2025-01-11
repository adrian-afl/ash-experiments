use crate::core::device::VEDevice;
use ash::vk;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEMainDeviceQueueError {
    #[error("queue wait idle failed")]
    QueueWaitIdleFailed(#[source] vk::Result),
}

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

    pub fn wait_idle(&self) -> Result<(), VEMainDeviceQueueError> {
        unsafe {
            self.device
                .device
                .queue_wait_idle(self.main_queue)
                .map_err(VEMainDeviceQueueError::QueueWaitIdleFailed)?
        }
        Ok(())
    }
}
