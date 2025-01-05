use crate::core::command_buffer::VECommandBuffer;
use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayoutField,
};
use crate::core::semaphore::VESemaphore;
use crate::core::shader_module::VEShaderModuleType;
use crate::core::toolkit::{App, VEToolkit};
use crate::graphics::attachment::VEAttachment;
use crate::graphics::render_stage::{CullMode, VERenderStage};
use crate::graphics::vertex_attributes::VertexAttribFormat;
use crate::graphics::vertex_buffer::VEVertexBuffer;
use crate::image::image::VEImage;
use ash::vk;
use std::sync::Arc;

pub struct MyApp {
    vertex_buffer: VEVertexBuffer,
    descriptor_set: Arc<VEDescriptorSet>,
    command_buffer: VECommandBuffer,
    render_stage: VERenderStage,
    render_done_semaphore: VESemaphore,
    result_image: Arc<VEImage>,
}

impl MyApp {
    pub fn new(toolkit: Arc<VEToolkit>) -> MyApp {
        let command_buffer = toolkit.make_command_buffer();

        let vertex_shader = toolkit.make_shader_module("vertex.spv", VEShaderModuleType::Vertex);
        let fragment_shader =
            toolkit.make_shader_module("fragment.spv", VEShaderModuleType::Fragment);

        let mut descriptor_set_layout =
            toolkit.make_descriptor_set_layout(&[VEDescriptorSetLayoutField {
                binding: 0,
                typ: VEDescriptorSetFieldType::Sampler,
                stage: VEDescriptorSetFieldStage::Fragment,
            }]);

        let descriptor_set = Arc::new(descriptor_set_layout.create_descriptor_set());
        // descriptor_set.bind_buffer(0, &buffer);

        let width = 640;
        let height = 480;

        let color_buffer = Arc::new(toolkit.make_image_full(
            width,
            height,
            1,
            vk::Format::R32G32B32A32_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::empty(),
        ));

        let color_attachment = VEAttachment::from_image(
            color_buffer.clone(),
            None,
            Some(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 1.0, 1.0],
                },
            }),
            false,
        );

        let depth_buffer = Arc::new(toolkit.make_image_full(
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

        let vertex_attributes = [
            VertexAttribFormat::RGB32f,
            VertexAttribFormat::RGB32f,
            VertexAttribFormat::RG32f,
            VertexAttribFormat::RGBA32f,
        ];

        let render_stage = toolkit.make_render_stage(
            width,
            height,
            &[&color_attachment, &depth_attachment],
            &[&descriptor_set_layout],
            &vertex_shader,
            &fragment_shader,
            &vertex_attributes,
            vk::PrimitiveTopology::TRIANGLE_LIST,
            CullMode::None,
        );

        let vertex_buffer = toolkit.make_vertex_buffer_from_file("dingus.raw", &vertex_attributes);

        let texture =
            toolkit.make_image_from_file("test-normal-map.jpg", vk::ImageUsageFlags::SAMPLED);

        let sampler = toolkit.make_sampler(
            vk::SamplerAddressMode::REPEAT,
            vk::Filter::LINEAR,
            vk::Filter::LINEAR,
            false,
        );

        descriptor_set.bind_image_sampler(0, &texture, &sampler);

        MyApp {
            result_image: color_buffer.clone(),
            render_done_semaphore: toolkit.make_semaphore(),
            render_stage,
            vertex_buffer,
            command_buffer,
            descriptor_set,
        }
    }
}

impl App for MyApp {
    fn draw(&mut self, toolkit: &VEToolkit) {
        self.render_stage.begin_recording(&self.command_buffer);

        self.render_stage
            .set_descriptor_set(&self.command_buffer, 0, self.descriptor_set.clone());

        self.vertex_buffer.draw_instanced(&self.command_buffer, 1);

        self.render_stage.end_recording(&self.command_buffer);

        let mut swapchain = toolkit.swapchain.lock().unwrap();

        self.command_buffer.submit(
            &toolkit.queue,
            &[&swapchain.blit_done_semaphore],
            &[&self.render_done_semaphore],
        );

        swapchain.blit(&self.result_image, &[&self.render_done_semaphore])
        // winit_window.pre_present_notify();
    }
}
