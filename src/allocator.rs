use crate::device::VEDevice;
use vk_mem::{Allocator, AllocatorCreateInfo};

pub struct VEAllocator {
    pub allocator: Allocator,
}

impl VEAllocator {
    pub fn new(device: &VEDevice) -> VEAllocator {
        let create_info =
            AllocatorCreateInfo::new(&device.instance, &device.device, device.physical_device);
        let allocator = unsafe { Allocator::new(create_info).unwrap() };
        VEAllocator { allocator }
    }
}
