use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::core::descriptor_set::VEDescriptorSet;
use crate::core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayout,
    VEDescriptorSetLayoutField,
};
use crate::core::helpers::{make_clear_color_f32, make_clear_depth};
use crate::core::memory_properties::VEMemoryProperties;
use crate::core::scheduler::VEScheduler;
use crate::core::semaphore::VESemaphore;
use crate::core::shader_module::VEShaderModuleType;
use crate::core::toolkit::{App, VEToolkit};
use crate::graphics::attachment::VEAttachment;
use crate::graphics::render_stage::{VECullMode, VEPrimitiveTopology, VERenderStage};
use crate::graphics::vertex_attributes::VertexAttribFormat;
use crate::graphics::vertex_buffer::VEVertexBuffer;
use crate::image::filtering::VEFiltering;
use crate::image::image::{VEImage, VEImageUsage};
use crate::image::image_format::VEImageFormat;
use crate::image::sampler::{VESampler, VESamplerAddressMode};
use std::sync::Arc;

pub struct MyApp {
    scheduler: VEScheduler,
    elapsed: f32,

    mesh_stage: MeshStage,

    meshes: Vec<Mesh>,
}

struct MeshStage {
    uniform_buffer: VEBuffer,

    depth_buffer: VEImage,
    color_buffer: Arc<VEImage>,

    mesh_descriptor_set_layout: VEDescriptorSetLayout,
    global_descriptor_set_layout: VEDescriptorSetLayout,
    global_descriptor_set: VEDescriptorSet,

    vertex_attributes: Vec<VertexAttribFormat>,
    render_stage: Arc<VERenderStage>,
}

struct Mesh {
    vertex_buffer: VEVertexBuffer,
    texture: VEImage,
    sampler: VESampler,

    descriptor_set: VEDescriptorSet,
}

impl MyApp {
    pub fn new(toolkit: &VEToolkit) -> MyApp {
        let mesh_stage = Self::make_mesh_stage(toolkit);

        let mut scheduler = toolkit.make_scheduler(2);

        let render_item = scheduler.make_render_item(mesh_stage.render_stage.clone());
        let blit_item = scheduler.make_blit_item(mesh_stage.color_buffer.clone());

        scheduler.set_layer(0, vec![render_item]);
        scheduler.set_layer(1, vec![blit_item]);

        let mut app = MyApp {
            mesh_stage,
            meshes: vec![],

            scheduler,
            elapsed: 0.0,
        };

        let dingus = app.make_mesh(toolkit, "dingus.jpg", "dingus.raw");

        app.meshes.push(dingus);

        app
    }

    fn make_mesh_stage(toolkit: &VEToolkit) -> MeshStage {
        let vertex_shader = toolkit.make_shader_module("vertex.spv", VEShaderModuleType::Vertex);
        let fragment_shader =
            toolkit.make_shader_module("fragment.spv", VEShaderModuleType::Fragment);

        let mut global_descriptor_set_layout =
            toolkit.make_descriptor_set_layout(&[VEDescriptorSetLayoutField {
                binding: 0,
                typ: VEDescriptorSetFieldType::UniformBuffer,
                stage: VEDescriptorSetFieldStage::AllGraphics,
            }]);

        let mut mesh_descriptor_set_layout =
            toolkit.make_descriptor_set_layout(&[VEDescriptorSetLayoutField {
                binding: 0,
                typ: VEDescriptorSetFieldType::Sampler,
                stage: VEDescriptorSetFieldStage::Fragment,
            }]);

        let global_descriptor_set = global_descriptor_set_layout.create_descriptor_set();

        let uniform_buffer = toolkit.make_buffer(
            VEBufferType::Uniform,
            128,
            Some(VEMemoryProperties::HostCoherent),
        );
        global_descriptor_set.bind_buffer(0, &uniform_buffer);

        let width = 640;
        let height = 480;

        let color_buffer = Arc::from(toolkit.make_image_full(
            width,
            height,
            1,
            VEImageFormat::RGBA32f,
            &[VEImageUsage::ColorAttachment, VEImageUsage::TransferSource],
            None,
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
            VEImageFormat::Depth32f,
            &[VEImageUsage::DepthAttachment],
            None,
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
            &[&global_descriptor_set_layout, &mesh_descriptor_set_layout],
            &vertex_shader,
            &fragment_shader,
            &vertex_attributes,
            VEPrimitiveTopology::TriangleList,
            VECullMode::None,
        ));

        MeshStage {
            uniform_buffer,
            depth_buffer,
            color_buffer,

            vertex_attributes: vertex_attributes.to_vec(),

            mesh_descriptor_set_layout,

            global_descriptor_set_layout,
            global_descriptor_set,

            render_stage,
        }
    }

    pub fn make_mesh(&mut self, toolkit: &VEToolkit, texture: &str, model: &str) -> Mesh {
        let descriptor_set = self
            .mesh_stage
            .mesh_descriptor_set_layout
            .create_descriptor_set();

        let vertex_buffer =
            toolkit.make_vertex_buffer_from_file(model, &self.mesh_stage.vertex_attributes);

        let texture = toolkit.make_image_from_file(texture, &[VEImageUsage::Sampled]);

        let sampler = toolkit.make_sampler(
            VESamplerAddressMode::Repeat,
            VEFiltering::Linear,
            VEFiltering::Linear,
            false,
        );

        descriptor_set.bind_image_sampler(0, &texture, &sampler);

        Mesh {
            vertex_buffer,
            texture,
            sampler,

            descriptor_set,
        }
    }
}

impl App for MyApp {
    fn draw(&mut self, toolkit: &VEToolkit) {
        let pointer = self.mesh_stage.uniform_buffer.map() as *mut f32;
        unsafe {
            pointer.write(self.elapsed);
        }
        self.mesh_stage.uniform_buffer.unmap();

        self.mesh_stage.render_stage.begin_recording();

        self.mesh_stage
            .render_stage
            .set_descriptor_set(0, &self.mesh_stage.global_descriptor_set);

        for mesh in &self.meshes {
            self.mesh_stage
                .render_stage
                .set_descriptor_set(1, &mesh.descriptor_set);

            self.mesh_stage
                .render_stage
                .draw_instanced(&mesh.vertex_buffer, 1);
        }

        self.mesh_stage.render_stage.end_recording();

        self.scheduler.run();

        self.elapsed += 0.01;
    }
}
