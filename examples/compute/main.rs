use std::process;
use std::sync::{Arc, Mutex};
use vengine_rs::buffer::buffer::VEBufferUsage;
use vengine_rs::core::descriptor_set_layout::{
    VEDescriptorSetFieldStage, VEDescriptorSetFieldType, VEDescriptorSetLayoutField,
};
use vengine_rs::core::memory_properties::VEMemoryProperties;
use vengine_rs::core::shader_module::VEShaderModuleType;
use vengine_rs::core::toolkit::{App, VEToolkit};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::window::{Window, WindowAttributes};

struct ComputeApp {}

#[allow(clippy::unwrap_used)]
impl ComputeApp {
    pub fn calculate(toolkit: Arc<VEToolkit>) -> ComputeApp {
        let mut buffer = toolkit
            .create_buffer(
                &[VEBufferUsage::Storage],
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
        // buffer.unmap().unwrap();

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

        let command_buffer = toolkit.create_command_buffer().unwrap();

        command_buffer.begin().unwrap();

        compute_stage.bind(&command_buffer);
        compute_stage.set_descriptor_set(&command_buffer, 0, &set);
        compute_stage.dispatch(&command_buffer, 4, 1, 1);

        command_buffer.end().unwrap();

        command_buffer
            .submit(&toolkit.queue.lock().unwrap(), vec![], vec![])
            .unwrap();

        toolkit.queue.lock().unwrap().wait_idle().unwrap();

        let pointer = buffer.map().unwrap() as *mut f32;
        unsafe {
            println!("{}", pointer.offset(0).read());
            println!("{}", pointer.offset(1).read());
            println!("{}", pointer.offset(2).read());
            println!("{}", pointer.offset(3).read());
        }
        // buffer.unmap().unwrap();

        process::exit(0);
    }
}

impl App for ComputeApp {
    fn draw(&mut self) {}
    fn on_window_event(&mut self, event: WindowEvent) {}
    fn on_device_event(&mut self, device_id: DeviceId, event: DeviceEvent) {}
}

#[allow(clippy::unwrap_used)]
fn main() {
    let window_attributes = WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(1, 1))
        .with_visible(false)
        .with_title("compute");

    VEToolkit::start(
        Box::from(|toolkit: Arc<VEToolkit>, window: Arc<Mutex<Window>>| {
            let app = ComputeApp::calculate(toolkit);
            Arc::new(Mutex::from(app)) as Arc<Mutex<dyn App>>
        }),
        window_attributes,
    )
    .unwrap()
}
