use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayoutField,
};
use crate::core::helpers::{make_clear_color_f32, make_clear_depth};
use crate::core::scheduler::VEScheduler;
use crate::core::semaphore::VESemaphore;
use crate::core::shader_module::VEShaderModuleType;
use crate::core::toolkit::{App, VEToolkit};
use crate::graphics::attachment::VEAttachment;
use crate::graphics::render_stage::{CullMode, VERenderStage};
use crate::graphics::vertex_attributes::VertexAttribFormat;
use crate::graphics::vertex_buffer::VEVertexBuffer;
use crate::image::image::VEImage;
use crate::image::sampler::VESampler;
use ash::vk;
use std::sync::Arc;

pub struct MyApp {
    texture: VEImage,
    sampler: VESampler,
    vertex_buffer: VEVertexBuffer,
    uniform_buffer: VEBuffer,
    descriptor_set: VEDescriptorSet,
    render_stage: Arc<VERenderStage>,
    render_done_semaphore: VESemaphore,
    scheduler: VEScheduler,
    depth_buffer: VEImage,
    color_buffer: Arc<VEImage>,
    elapsed: f32,
}

impl MyApp {
    pub fn new(toolkit: &VEToolkit) -> MyApp {
        let vertex_shader = toolkit.make_shader_module("vertex.spv", VEShaderModuleType::Vertex);
        let fragment_shader =
            toolkit.make_shader_module("fragment.spv", VEShaderModuleType::Fragment);

        let mut descriptor_set_layout = toolkit.make_descriptor_set_layout(&[
            VEDescriptorSetLayoutField {
                binding: 0,
                typ: VEDescriptorSetFieldType::Sampler,
                stage: VEDescriptorSetFieldStage::Fragment,
            },
            VEDescriptorSetLayoutField {
                binding: 1,
                typ: VEDescriptorSetFieldType::UniformBuffer,
                stage: VEDescriptorSetFieldStage::AllGraphics,
            },
        ]);

        let descriptor_set = descriptor_set_layout.create_descriptor_set();

        let width = 640;
        let height = 480;

        let color_buffer = Arc::from(toolkit.make_image_full(
            width,
            height,
            1,
            vk::Format::R32G32B32A32_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::empty(),
        ));

        let color_attachment = VEAttachment::from_image(
            &color_buffer,
            None,
            Some(make_clear_color_f32([0.0, 0.0, 1.0, 1.0])),
            false,
        );

        let depth_buffer = toolkit.make_image_full(
            width,
            height,
            1,
            vk::Format::D32_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::empty(),
        );

        let depth_attachment =
            VEAttachment::from_image(&depth_buffer, None, Some(make_clear_depth(1.0)), false);

        let vertex_attributes = [
            VertexAttribFormat::RGB32f,
            VertexAttribFormat::RGB32f,
            VertexAttribFormat::RG32f,
            VertexAttribFormat::RGBA32f,
        ];

        let render_stage = Arc::new(toolkit.make_render_stage(
            width,
            height,
            &[&color_attachment, &depth_attachment],
            &[&descriptor_set_layout],
            &vertex_shader,
            &fragment_shader,
            &vertex_attributes,
            vk::PrimitiveTopology::TRIANGLE_LIST,
            CullMode::None,
        ));

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

        let mut scheduler = toolkit.make_scheduler(2);

        let render_item = scheduler.make_render_item(render_stage.clone());
        let blit_item = scheduler.make_blit_item(color_buffer.clone());

        scheduler.set_layer(0, vec![render_item]);
        scheduler.set_layer(1, vec![blit_item]);

        let uniform_buffer = toolkit.make_buffer(
            VEBufferType::Uniform,
            128,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        );
        descriptor_set.bind_buffer(1, &uniform_buffer);

        MyApp {
            render_done_semaphore: toolkit.make_semaphore(),
            render_stage: render_stage,
            vertex_buffer,
            descriptor_set,
            texture,
            sampler,
            scheduler,
            uniform_buffer,
            depth_buffer,
            color_buffer,
            elapsed: 0.0,
        }
    }
}

impl App for MyApp {
    fn draw(&mut self, toolkit: &VEToolkit) {
        let pointer = self.uniform_buffer.map() as *mut f32;

        unsafe {
            pointer.write(self.elapsed);
        }

        self.uniform_buffer.unmap();

        self.render_stage.begin_recording();

        self.render_stage
            .set_descriptor_set(0, &self.descriptor_set);

        self.render_stage.draw_instanced(&self.vertex_buffer, 1);

        self.render_stage.end_recording();

        self.scheduler.run();

        self.elapsed += 0.01;
    }
}
