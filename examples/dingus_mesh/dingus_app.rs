use std::sync::Arc;
use vengine_rs::buffer::buffer::{VEBuffer, VEBufferUsage};
use vengine_rs::core::descriptor_set::VEDescriptorSet;
use vengine_rs::core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayout,
    VEDescriptorSetLayoutField,
};
use vengine_rs::core::helpers::{clear_color_f32, clear_depth};
use vengine_rs::core::memory_properties::VEMemoryProperties;
use vengine_rs::core::scheduler::VEScheduler;
use vengine_rs::core::shader_module::VEShaderModuleType;
use vengine_rs::core::toolkit::{App, VEToolkit};
use vengine_rs::graphics::attachment::VEAttachment;
use vengine_rs::graphics::render_stage::{VECullMode, VEPrimitiveTopology, VERenderStage};
use vengine_rs::graphics::vertex_attributes::VertexAttribFormat;
use vengine_rs::graphics::vertex_buffer::VEVertexBuffer;
use vengine_rs::image::filtering::VEFiltering;
use vengine_rs::image::image::{VEImage, VEImageUsage, VEImageViewCreateInfo};
use vengine_rs::image::image_format::VEImageFormat;
use vengine_rs::image::sampler::{VESampler, VESamplerAddressMode};
use winit::event::{DeviceEvent, DeviceId, WindowEvent};

pub struct DingusApp {
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

#[allow(clippy::unwrap_used)]
impl DingusApp {
    pub fn new(toolkit: Arc<VEToolkit>) -> DingusApp {
        let mesh_stage = Self::create_mesh_stage(&toolkit);

        let mut scheduler = toolkit.create_scheduler(2);

        let render_item = scheduler
            .create_render_item(mesh_stage.render_stage.clone())
            .unwrap();
        let blit_item = scheduler
            .create_blit_item(mesh_stage.color_buffer.clone())
            .unwrap();

        scheduler.set_layer(0, vec![render_item]).unwrap();
        scheduler.set_layer(1, vec![blit_item]).unwrap();

        let mut app = DingusApp {
            mesh_stage,
            meshes: vec![],

            scheduler,
            elapsed: 0.0,
        };

        let dingus = app.create_mesh(
            &toolkit,
            "examples/dingus_mesh/dingus.jpg",
            "examples/dingus_mesh/dingus.raw",
        );

        app.meshes.push(dingus);

        app
    }

    fn create_mesh_stage(toolkit: &VEToolkit) -> MeshStage {
        let vertex_shader = toolkit
            .create_shader_module(
                "examples/dingus_mesh/vertex.spv",
                VEShaderModuleType::Vertex,
            )
            .unwrap();
        let fragment_shader = toolkit
            .create_shader_module(
                "examples/dingus_mesh/fragment.spv",
                VEShaderModuleType::Fragment,
            )
            .unwrap();

        let mut global_descriptor_set_layout = toolkit
            .create_descriptor_set_layout(&[VEDescriptorSetLayoutField {
                binding: 0,
                typ: VEDescriptorSetFieldType::UniformBuffer,
                stage: VEDescriptorSetFieldStage::AllGraphics,
            }])
            .unwrap();

        let mut mesh_descriptor_set_layout = toolkit
            .create_descriptor_set_layout(&[VEDescriptorSetLayoutField {
                binding: 0,
                typ: VEDescriptorSetFieldType::Sampler,
                stage: VEDescriptorSetFieldStage::Fragment,
            }])
            .unwrap();

        let global_descriptor_set = global_descriptor_set_layout
            .create_descriptor_set()
            .unwrap();

        let uniform_buffer = toolkit
            .create_buffer(
                &[VEBufferUsage::Uniform],
                128,
                Some(VEMemoryProperties::HostCoherent),
            )
            .unwrap();
        global_descriptor_set
            .bind_buffer(0, &uniform_buffer)
            .unwrap();

        let width = 640;
        let height = 480;

        let mut color_buffer = toolkit
            .create_image_full(
                width,
                height,
                1,
                VEImageFormat::RGBA32f,
                &[VEImageUsage::ColorAttachment, VEImageUsage::TransferSource],
            )
            .unwrap();

        let color_attachment_view = color_buffer
            .get_view(VEImageViewCreateInfo::simple_2d())
            .unwrap();

        let color_attachment = VEAttachment::from_image(
            &color_buffer,
            color_attachment_view,
            None,
            Some(clear_color_f32([0.0, 0.0, 1.0, 1.0])),
        )
        .unwrap();

        let color_buffer = Arc::from(color_buffer);

        let mut depth_buffer = toolkit
            .create_image_full(
                width,
                height,
                1,
                VEImageFormat::Depth32f,
                &[VEImageUsage::DepthAttachment],
            )
            .unwrap();

        let depth_attachment_view = depth_buffer
            .get_view(VEImageViewCreateInfo::simple_2d())
            .unwrap();

        let depth_attachment = VEAttachment::from_image(
            &depth_buffer,
            depth_attachment_view,
            None,
            Some(clear_depth(1.0)),
        )
        .unwrap();

        let vertex_attributes = [
            VertexAttribFormat::RGB32f,
            VertexAttribFormat::RGB32f,
            VertexAttribFormat::RG32f,
            VertexAttribFormat::RGBA32f,
        ];

        let render_stage = Arc::new(
            toolkit
                .create_render_stage(
                    width,
                    height,
                    &[&color_attachment, &depth_attachment],
                    &[&global_descriptor_set_layout, &mesh_descriptor_set_layout],
                    &vertex_shader,
                    &fragment_shader,
                    &vertex_attributes,
                    VEPrimitiveTopology::TriangleList,
                    VECullMode::Back,
                )
                .unwrap(),
        );

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

    pub fn create_mesh(&mut self, toolkit: &VEToolkit, texture: &str, model: &str) -> Mesh {
        let descriptor_set = self
            .mesh_stage
            .mesh_descriptor_set_layout
            .create_descriptor_set()
            .unwrap();

        let vertex_buffer = toolkit
            .create_vertex_buffer_from_file(model, &self.mesh_stage.vertex_attributes)
            .unwrap();

        let mut texture = toolkit
            .create_image_from_file(texture, &[VEImageUsage::Sampled])
            .unwrap();

        let sampler = toolkit
            .create_sampler(
                VESamplerAddressMode::Repeat,
                VEFiltering::Linear,
                VEFiltering::Linear,
                false,
            )
            .unwrap();

        let texture_view = texture
            .get_view(VEImageViewCreateInfo::simple_2d())
            .unwrap();

        descriptor_set
            .bind_image_sampler(0, &texture, texture_view, &sampler)
            .unwrap();

        Mesh {
            vertex_buffer,
            texture,
            sampler,

            descriptor_set,
        }
    }
}

#[allow(clippy::unwrap_used)]
impl App for DingusApp {
    fn draw(&mut self) {
        let pointer = self.mesh_stage.uniform_buffer.map().unwrap() as *mut f32;
        unsafe {
            pointer.write(self.elapsed);
        }
        self.mesh_stage.uniform_buffer.unmap().unwrap();

        self.mesh_stage.render_stage.begin_recording().unwrap();

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

        self.mesh_stage.render_stage.end_recording().unwrap();

        self.scheduler.run().unwrap();

        self.elapsed += 0.001;
    }

    fn on_window_event(&self, event: WindowEvent) {}

    fn on_device_event(&self, device_id: DeviceId, event: DeviceEvent) {}
}
