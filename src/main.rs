mod allocator;
mod buffer;
mod command_buffer;
mod command_pool;
mod device;
mod main_device_queue;
mod memory;
mod swapchain;
mod vertex_buffer;
mod window;

use crate::allocator::VEAllocator;
use crate::buffer::VEBuffer;
use crate::command_pool::VECommandPool;
use crate::device::VEDevice;
use crate::main_device_queue::VEMainDeviceQueue;
use crate::memory::memory_manager::VEMemoryManager;
use crate::swapchain::VESwapchain;
use crate::window::VEWindow;
use ash::vk::BufferUsageFlags;
use tokio::main;

struct Test<'a> {
    value: i32,
}

struct Test2<'a> {
    test: &'a mut Test<'a>,
}

fn test(test: &mut Test) {
    let mut test = Test { value: 1 };
    let test2 = Test2 { test: &mut test };

    tesXt(&mut test);

    println!("{}", test2.test.value);
}

fn tesXt(test: &mut Test) {
    println!("{}", test.value);
}

#[main]
async fn main() {
    env_logger::init();
    let window = VEWindow::new();
    let device = VEDevice::new(&window);
    {
        let command_pool = VECommandPool::new(&device);
        let main_device_queue = VEMainDeviceQueue::new(&device);
        let swapchain = VESwapchain::new(&window, &device);
        {
            let mut mem = VEMemoryManager::new(&device);
            {
                let mut buffer =
                    VEBuffer::new(&device, &mut mem, 1024, BufferUsageFlags::UNIFORM_BUFFER);
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
