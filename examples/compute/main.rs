use std::fs::File;
use std::process;
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::FmtSubscriber;
use vengine_rs::buffer::buffer::VEBufferType;
use vengine_rs::core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayoutField,
};
use vengine_rs::core::memory_properties::VEMemoryProperties;
use vengine_rs::core::shader_module::VEShaderModuleType;
use vengine_rs::core::toolkit::{App, VEToolkit};
use winit::dpi::PhysicalSize;
use winit::window::WindowAttributes;

struct ComputeApp {}

#[allow(clippy::unwrap_used)]
impl ComputeApp {
    pub fn calculate(toolkit: &VEToolkit) -> ComputeApp {
        let mut buffer = toolkit
            .create_buffer(
                VEBufferType::Storage,
                128,
                Some(VEMemoryProperties::HostCoherent),
            )
            .unwrap();
        let pointer = buffer.map().unwrap() as *mut f32;
        unsafe {
            pointer.offset(0).write(1.0);
            pointer.offset(1).write(10.0);
            pointer.offset(2).write(100.0);
            pointer.offset(3).write(1000.0);
        }
        buffer.unmap().unwrap();

        let mut set_layout = toolkit
            .create_descriptor_set_layout(&[VEDescriptorSetLayoutField {
                binding: 0,
                typ: VEDescriptorSetFieldType::StorageBuffer,
                stage: VEDescriptorSetFieldStage::Compute,
            }])
            .unwrap();

        let shader = toolkit
            .create_shader_module("examples/compute/compute.spv", VEShaderModuleType::Compute)
            .unwrap();

        let compute_stage = toolkit
            .create_compute_stage(&[&set_layout], &shader)
            .unwrap();

        let set = set_layout.create_descriptor_set().unwrap();
        set.bind_buffer(0, &buffer).unwrap();

        compute_stage.begin_recording().unwrap();
        compute_stage.set_descriptor_set(0, &set);
        compute_stage.dispatch(4, 1, 1);
        compute_stage.end_recording().unwrap();
        compute_stage
            .command_buffer
            .submit(&toolkit.queue, vec![], vec![])
            .unwrap();

        toolkit.queue.wait_idle().unwrap();

        let pointer = buffer.map().unwrap() as *mut f32;
        unsafe {
            println!("{}", pointer.offset(0).read());
            println!("{}", pointer.offset(1).read());
            println!("{}", pointer.offset(2).read());
            println!("{}", pointer.offset(3).read());
        }
        buffer.unmap().unwrap();

        process::exit(0);
    }
}

impl App for ComputeApp {
    fn draw(&mut self, toolkit: &VEToolkit) {}
}

#[allow(clippy::unwrap_used)]
fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_ansi(false)
        .with_writer(File::create("../log.txt").unwrap())
        .with_span_events(FmtSpan::FULL)
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let window_attributes = WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(1, 1))
        .with_visible(false)
        .with_title("compute");

    VEToolkit::start(
        Box::from(|toolkit: &VEToolkit| {
            let app = ComputeApp::calculate(&toolkit);
            Arc::new(Mutex::from(app)) as Arc<Mutex<dyn App>>
        }),
        window_attributes,
    )
    .unwrap()
}
