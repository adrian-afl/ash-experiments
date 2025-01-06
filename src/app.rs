use crate::core::command_buffer::VECommandBuffer;
use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayoutField,
};
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
use tracing::{event, Level};

pub struct MyApp {
    texture: VEImage,
    sampler: VESampler,
    vertex_buffer: VEVertexBuffer,
    descriptor_set: VEDescriptorSet,
    render_stage: Arc<VERenderStage>,
    render_done_semaphore: VESemaphore,
    scheduler: VEScheduler,
}

impl MyApp {
    pub fn new(toolkit: &VEToolkit) -> MyApp {
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

        let descriptor_set = descriptor_set_layout.create_descriptor_set();
        // descriptor_set.bind_buffer(0, &buffer);

        let width = 640;
        let height = 480;

        let mut color_buffer = Arc::from(toolkit.make_image_full(
            width,
            height,
            1,
            vk::Format::R32G32B32A32_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::empty(),
        ));

        // color_buffer.transition_layout(vk::ImageLayout::PREINITIALIZED, vk::ImageLayout::GENERAL);

        let color_attachment = VEAttachment::from_image(
            &color_buffer,
            None,
            Some(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 1.0, 1.0],
                },
            }),
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

        let depth_attachment = VEAttachment::from_image(
            &depth_buffer,
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

        let mut scheduler = VEScheduler::new(2);

        let render_item = scheduler.make_render_item(toolkit, "render", render_stage.clone());
        let blit_item = scheduler.make_blit_item(toolkit, "blit", color_buffer);

        scheduler.set_layer(0, vec![render_item]);
        scheduler.set_layer(1, vec![blit_item]);

        MyApp {
            render_done_semaphore: toolkit.make_semaphore(),
            render_stage: render_stage,
            vertex_buffer,
            descriptor_set,
            texture,
            sampler,
            scheduler,
        }
    }
}

impl App for MyApp {
    fn draw(&mut self, toolkit: &VEToolkit) {
        self.render_stage.begin_recording();

        self.render_stage
            .set_descriptor_set(0, &self.descriptor_set);

        self.render_stage.draw_instanced(&self.vertex_buffer, 1);

        self.render_stage.end_recording();

        self.scheduler.run(toolkit);
    }
}
