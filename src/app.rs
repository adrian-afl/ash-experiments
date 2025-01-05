use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayout,
    VEDescriptorSetLayoutField,
};
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::shader_module::{VEShaderModule, VEShaderModuleType};
use crate::core::toolkit::{App, VEToolkit};
use crate::graphics::attachment::VEAttachment;
use crate::graphics::render_stage::CullMode;
use crate::graphics::vertex_attributes::VertexAttribFormat;
use crate::graphics::vertex_buffer::VEVertexBuffer;
use crate::image::image::VEImage;
use crate::image::sampler::VESampler;
use crate::memory::memory_manager::VEMemoryManager;
use crate::window::output_stage::VEOutputStage;
use crate::window::swapchain::VESwapchain;
use crate::window::window::VEWindow;
use ash::vk;
use std::fs;
use std::sync::{Arc, Mutex};

pub struct MyApp {
    vertex_buffer: Option<VEVertexBuffer>,
    descriptor_set: Option<Arc<VEDescriptorSet>>,
    output_command_buffer: Option<VECommandBuffer>,
    output_stage: Option<VEOutputStage>,
    queue: Option<Arc<VEMainDeviceQueue>>,
}

impl MyApp {
    pub fn new(toolkit: Arc<VEToolkit>) -> MyApp {
        let command_buffer = toolkit.make_command_buffer();

        let fragment_shader = toolkit.make_shader_module("vertex.spv", VEShaderModuleType::Vertex);
        let vertex_shader =
            toolkit.make_shader_module("fragment.spv", VEShaderModuleType::Fragment);

        let mut descriptor_set_layout =
            toolkit.make_descriptor_set_layout(&[VEDescriptorSetLayoutField {
                binding: 0,
                typ: VEDescriptorSetFieldType::Sampler,
                stage: VEDescriptorSetFieldStage::Fragment,
            }]);

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

        self.vertex_buffer = Some(VEVertexBuffer::from_file(
            device.clone(),
            memory_manager.clone(),
            "dingus.raw",
            3 * 4 + 3 * 4 + 2 * 4 + 4 * 4,
        ));

        self.output_stage = Some(output_stage);

        self.output_command_buffer = Some(command_buffer);

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

        self.queue = Some(main_device_queue.clone());
        self.descriptor_set = Some(descriptor_set);
    }
}

impl App for MyApp {
    fn draw(&mut self, toolkit: Arc<VEToolkit>) {
        let mut output_stage = self.output_stage.as_mut().unwrap();
        let command_buffer = self.output_command_buffer.as_ref().unwrap();
        let vertex_buffer = self.vertex_buffer.as_ref().unwrap();
        let descriptor_set = self.descriptor_set.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();

        output_stage.next_image();
        output_stage.begin_recording(&command_buffer);

        output_stage.set_descriptor_set(&command_buffer, 0, descriptor_set.clone());

        vertex_buffer.draw_instanced(&command_buffer, 1);

        output_stage.end_recording(&command_buffer);
        command_buffer.submit(
            &queue,
            &[&output_stage.image_ready_semaphore],
            &[&output_stage.ready_for_present_semaphore],
        );

        output_stage.present();
        // winit_window.pre_present_notify();
    }
}
