mod buffer;
mod compute;
mod core;
mod graphics;
mod image;
mod memory;
mod window;

use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::graphics::attachment::VEAttachment;
use crate::graphics::vertex_buffer::VEVertexBuffer;
use crate::image::image::VEImage;
use crate::image::sampler::VESampler;
use crate::memory::memory_manager::VEMemoryManager;
use crate::window::swapchain::VESwapchain;
use crate::window::window::VEWindow;
use ash::vk;
use compute::compute_stage::VEComputeStage;
use core::command_buffer::VECommandBuffer;
use core::command_pool::VECommandPool;
use core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayout,
    VEDescriptorSetLayoutField,
};
use core::device::VEDevice;
use core::main_device_queue::VEMainDeviceQueue;
use core::shader_module::{VEShaderModule, VEShaderModuleType};
use graphics::render_stage::CullMode;
use graphics::vertex_attributes::VertexAttribFormat;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tokio::main;
use window::output_stage::VEOutputStage;

#[main]
async fn main() {
    env_logger::init();
    let window = VEWindow::new();
    let width = window.window.inner_size().width;
    let height = window.window.inner_size().height;
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

                let command_buffer = VECommandBuffer::new(device.clone(), command_pool.clone());

                //
                // let mut descriptor_set_layout = VEDescriptorSetLayout::new(
                //     device.clone(),
                //     &[VEDescriptorSetLayoutField {
                //         binding: 0,
                //         typ: VEDescriptorSetFieldType::StorageBuffer,
                //         stage: VEDescriptorSetFieldStage::Compute,
                //     }],
                // );
                //
                // let descriptor_set = Arc::new(descriptor_set_layout.create_descriptor_set());
                // descriptor_set.bind_buffer(0, &buffer);
                //
                // let compute_shader = VEShaderModule::new(
                //     device.clone(),
                //     &mut fs::File::open("compute.spv").unwrap(),
                //     VEShaderModuleType::Compute,
                // );
                //
                // let mut compute_stage =
                //     VEComputeStage::new(device.clone(), &[&descriptor_set_layout], &compute_shader);
                // // let compute_stage2 =
                // //     VEComputeStage::new(device.clone(), &[&descriptor_set_layout], &compute_shader);
                //
                //
                // compute_stage.begin_recording(&command_buffer);
                // compute_stage.set_descriptor_set(&command_buffer, 0, descriptor_set);
                // compute_stage.dispatch(&command_buffer, 4, 1, 1);
                // compute_stage.end_recording(&command_buffer);
                //
                // command_buffer.submit(&main_device_queue, &[], &[]);
                //
                // main_device_queue.wait_idle();
                //
                // let mem = buffer.map() as *mut f32;
                // let readback1 = unsafe { mem.offset(0).read() };
                // let readback2 = unsafe { mem.offset(1).read() };
                // let readback3 = unsafe { mem.offset(2).read() };
                // let readback4 = unsafe { mem.offset(3).read() };
                // buffer.unmap();
                //
                // println!("{:?}", device.device.handle());
                // println!("{readback1}, {readback2}, {readback3}, {readback4}");

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

                let mut descriptor_set_layout = VEDescriptorSetLayout::new(
                    device.clone(),
                    &[VEDescriptorSetLayoutField {
                        binding: 0,
                        typ: VEDescriptorSetFieldType::Sampler,
                        stage: VEDescriptorSetFieldStage::Fragment,
                    }],
                );

                let descriptor_set = Arc::new(descriptor_set_layout.create_descriptor_set());
                // descriptor_set.bind_buffer(0, &buffer);

                let depth_buffer = Arc::new(VEImage::from_full(
                    device.clone(),
                    main_device_queue.clone(),
                    command_pool.clone(),
                    memory_manager.clone(),
                    width,
                    height,
                    1,
                    vk::Format::D32_SFLOAT,
                    vk::ImageTiling::OPTIMAL,
                    vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                    vk::MemoryPropertyFlags::empty(),
                ));

                let depth_attachment = VEAttachment::from_image(
                    depth_buffer,
                    None,
                    Some(vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 1.0,
                            stencil: 0,
                        },
                    }),
                    false,
                );

                let mut output_stage = VEOutputStage::new(
                    device.clone(),
                    main_device_queue.clone(),
                    command_pool.clone(),
                    memory_manager.clone(),
                    swapchain.clone(),
                    Some(vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.0, 0.0, 1.0, 1.0],
                        },
                    }),
                    Some(&depth_attachment),
                    &[&descriptor_set_layout],
                    &vertex_shader,
                    &fragment_shader,
                    &[
                        VertexAttribFormat::RGB32f,
                        VertexAttribFormat::RGB32f,
                        VertexAttribFormat::RG32f,
                        VertexAttribFormat::RGBA32f,
                    ],
                    vk::PrimitiveTopology::TRIANGLE_LIST,
                    CullMode::None,
                );

                let vertex_buffer = VEVertexBuffer::from_file(
                    device.clone(),
                    memory_manager.clone(),
                    "dingus.raw",
                    3 * 4 + 3 * 4 + 2 * 4 + 4 * 4,
                );

                let texture = VEImage::from_file(
                    device.clone(),
                    main_device_queue.clone(),
                    command_pool.clone(),
                    memory_manager.clone(),
                    "test-normal-map.jpg",
                    vk::ImageUsageFlags::SAMPLED,
                );

                let sampler = VESampler::new(
                    device.clone(),
                    vk::SamplerAddressMode::REPEAT,
                    vk::Filter::LINEAR,
                    vk::Filter::LINEAR,
                    false,
                );

                descriptor_set.bind_image_sampler(0, &texture, &sampler);

                output_stage.next_image();
                output_stage.begin_recording(&command_buffer);

                output_stage.set_descriptor_set(&command_buffer, 0, descriptor_set);
                vertex_buffer.draw_instanced(&command_buffer, 1);

                output_stage.end_recording(&command_buffer);
                command_buffer.submit(
                    &main_device_queue,
                    &[&output_stage.image_ready_semaphore],
                    &[&output_stage.ready_for_present_semaphore],
                );

                output_stage.present();

                sleep(Duration::from_secs(2));
            }
        }
    }
}
