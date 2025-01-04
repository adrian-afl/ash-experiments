mod attachment;
mod buffer;
mod command_buffer;
mod command_pool;
mod compute_pipeline;
mod compute_stage;
mod descriptor_set;
mod descriptor_set_layout;
mod device;
mod framebuffer;
mod graphics_pipeline;
mod image;
mod main_device_queue;
mod memory;
mod output_stage;
mod render_stage;
mod renderpass;
mod semaphore;
mod shader_module;
mod subpass;
mod swapchain;
mod vertex_attributes;
mod vertex_buffer;
mod window;

use crate::buffer::{VEBuffer, VEBufferType};
use crate::command_buffer::VECommandBuffer;
use crate::command_pool::VECommandPool;
use crate::compute_stage::VEComputeStage;
use crate::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayout,
    VEDescriptorSetLayoutField,
};
use crate::device::VEDevice;
use crate::main_device_queue::VEMainDeviceQueue;
use crate::memory::memory_manager::VEMemoryManager;
use crate::output_stage::VEOutputStage;
use crate::render_stage::CullMode;
use crate::shader_module::{VEShaderModule, VEShaderModuleType};
use crate::swapchain::VESwapchain;
use crate::vertex_attributes::VertexAttribFormat;
use crate::window::VEWindow;
use ash::vk;
use ash::vk::BufferUsageFlags;
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::main;

#[main]
async fn main() {
    env_logger::init();
    let window = VEWindow::new();
    let device = Arc::new(VEDevice::new(&window));
    {
        let command_pool = Arc::new(VECommandPool::new(device.clone()));
        let main_device_queue = Arc::new(VEMainDeviceQueue::new(device.clone()));
        let swapchain = Arc::new(Mutex::from(VESwapchain::new(
            &window,
            device.clone(),
            main_device_queue.clone(),
        )));
        {
            let mut memory_manager = Arc::new(Mutex::from(VEMemoryManager::new(device.clone())));
            {
                let mut buffer = VEBuffer::new(
                    device.clone(),
                    VEBufferType::Storage,
                    memory_manager.clone(),
                    1024,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                );
                let mem = buffer.map() as *mut f32;
                unsafe {
                    mem.offset(0).write(1.0_f32);
                    mem.offset(1).write(10.0_f32);
                    mem.offset(2).write(100.0_f32);
                    mem.offset(3).write(1000.0_f32);
                }
                buffer.unmap();

                let mut descriptor_set_layout = VEDescriptorSetLayout::new(
                    device.clone(),
                    &[VEDescriptorSetLayoutField {
                        binding: 0,
                        typ: VEDescriptorSetFieldType::StorageBuffer,
                        stage: VEDescriptorSetFieldStage::Compute,
                    }],
                );

                let descriptor_set = Arc::new(descriptor_set_layout.create_descriptor_set());
                descriptor_set.bind_buffer(0, &buffer);

                let compute_shader = VEShaderModule::new(
                    device.clone(),
                    &mut fs::File::open("compute.spv").unwrap(),
                    VEShaderModuleType::Compute,
                );

                let mut compute_stage =
                    VEComputeStage::new(device.clone(), &[&descriptor_set_layout], &compute_shader);
                // let compute_stage2 =
                //     VEComputeStage::new(device.clone(), &[&descriptor_set_layout], &compute_shader);

                let command_buffer = VECommandBuffer::new(device.clone(), command_pool.clone());
                compute_stage.begin_recording(&command_buffer);
                compute_stage.set_descriptor_set(&command_buffer, 0, descriptor_set);
                compute_stage.dispatch(&command_buffer, 4, 1, 1);
                compute_stage.end_recording(&command_buffer);

                command_buffer.submit(&main_device_queue, &[], &[]);

                main_device_queue.wait_idle();

                let mem = buffer.map() as *mut f32;
                let readback1 = unsafe { mem.offset(0).read() };
                let readback2 = unsafe { mem.offset(1).read() };
                let readback3 = unsafe { mem.offset(2).read() };
                let readback4 = unsafe { mem.offset(3).read() };
                buffer.unmap();

                println!("{:?}", device.device.handle());
                println!("{readback1}, {readback2}, {readback3}, {readback4}");

                let vertex_shader = VEShaderModule::new(
                    device.clone(),
                    &mut fs::File::open("vertex.spv").unwrap(),
                    VEShaderModuleType::Compute,
                );

                let fragment_shader = VEShaderModule::new(
                    device.clone(),
                    &mut fs::File::open("fragment.spv").unwrap(),
                    VEShaderModuleType::Compute,
                );

                let mut output_stage = VEOutputStage::new(
                    device.clone(),
                    main_device_queue.clone(),
                    command_pool.clone(),
                    memory_manager.clone(),
                    swapchain.clone(),
                    None,
                    &[],
                    &vertex_shader,
                    &fragment_shader,
                    &[VertexAttribFormat::RGB32f],
                    vk::PrimitiveTopology::TRIANGLE_LIST,
                    CullMode::None,
                );

                output_stage.next_image();
                output_stage.begin_recording(&command_buffer);
                output_stage.end_recording(&command_buffer);
                command_buffer.submit(
                    &main_device_queue,
                    &[&output_stage.image_ready_semaphore],
                    &[&output_stage.ready_for_present_semaphore],
                );

                output_stage.present();
            }
        }
    }
}
