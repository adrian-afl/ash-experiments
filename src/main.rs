mod buffer;
mod command_buffer;
mod command_pool;
mod device;
mod main_device_queue;
mod memory;
mod swapchain;
mod vertex_buffer;
mod window;

use crate::buffer::VEBuffer;
use crate::command_pool::VECommandPool;
use crate::device::VEDevice;
use crate::main_device_queue::VEMainDeviceQueue;
use crate::memory::memory_manager::VEMemoryManager;
use crate::swapchain::VESwapchain;
use crate::window::VEWindow;
use ash::vk::BufferUsageFlags;
use std::sync::{Arc, Mutex};
use tokio::main;

#[main]
async fn main() {
    env_logger::init();
    let window = VEWindow::new();
    let device = Arc::new(VEDevice::new(&window));
    {
        let command_pool = VECommandPool::new(device.clone());
        let main_device_queue = VEMainDeviceQueue::new(device.clone());
        let swapchain = VESwapchain::new(&window, device.clone());
        {
            let mut mem = Arc::new(Mutex::from(VEMemoryManager::new(device.clone())));
            {
                let mut buffer = VEBuffer::new(
                    device.clone(),
                    mem.clone(),
                    1024,
                    BufferUsageFlags::UNIFORM_BUFFER,
                );
                let mem = buffer.map() as *mut u8;
                unsafe {
                    mem.write(69);
                }
                buffer.unmap();

                let mem = buffer.map() as *mut u8;
                let readback = unsafe { mem.read() };
                buffer.unmap();

                println!("{:?}", device.device.handle());
                println!("{readback}");
            }
        }
    }
}
