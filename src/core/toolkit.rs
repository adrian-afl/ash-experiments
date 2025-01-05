use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::descriptor_set_layout::{VEDescriptorSetLayout, VEDescriptorSetLayoutField};
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::shader_module::{VEShaderModule, VEShaderModuleType};
use crate::image::image::VEImage;
use crate::memory::memory_manager::VEMemoryManager;
use crate::window::swapchain::VESwapchain;
use crate::window::window::{AppCallback, VEWindow};
use ash::vk;
use ash::vk::ShaderModule;
use std::sync::{Arc, Mutex};
use std::{fs, io};
use winit::window::WindowAttributes;

pub trait App {
    fn draw(&mut self, toolkit: &VEToolkit);
}

pub struct VEToolkit {
    window: Arc<VEWindow>,
    device: Arc<VEDevice>,
    swapchain: Arc<Mutex<VESwapchain>>,
    queue: Arc<VEMainDeviceQueue>,
    command_pool: Arc<VECommandPool>,
    memory_manager: Arc<Mutex<VEMemoryManager>>,
}

pub struct VEToolkitCallbacks {
    toolkit: Option<Arc<VEToolkit>>,
    pub window: Option<Arc<VEWindow>>,
    create_app: Box<dyn Fn(Arc<VEToolkit>) -> Arc<Mutex<dyn App>>>,
    app: Option<Arc<Mutex<dyn App>>>,
}

impl AppCallback for VEToolkitCallbacks {
    fn on_window_ready(&mut self) {
        let toolkit = Arc::from(VEToolkit::new(self.window.as_ref().unwrap().clone()));
        self.toolkit = Some(toolkit.clone());
        let constructor = &self.create_app;
        let app = constructor(toolkit.clone());
        self.app = Some(app);
    }

    fn on_window_draw(&self) {
        self.app
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .draw(self.toolkit.as_ref().unwrap());
    }
}

impl VEToolkit {
    pub fn start(
        create_app: Box<dyn Fn(Arc<VEToolkit>) -> Arc<Mutex<dyn App>>>,
        initial_window_attributes: WindowAttributes,
    ) {
        let callbacks = Arc::new(Mutex::from(VEToolkitCallbacks {
            toolkit: None,
            window: None,
            app: None,
            create_app,
        }));
        callbacks.lock().unwrap().window = Some(Arc::new(VEWindow::new(
            callbacks.clone(),
            initial_window_attributes,
        ))); //oh god i hope this won't race condition with event loop
    }

    pub fn new(window: Arc<VEWindow>) -> VEToolkit {
        let device = Arc::new(VEDevice::new(&window));

        let command_pool = Arc::new(VECommandPool::new(device.clone()));
        let queue = Arc::new(VEMainDeviceQueue::new(device.clone()));
        let swapchain = Arc::new(Mutex::from(VESwapchain::new(
            &window,
            device.clone(),
            queue.clone(),
        )));

        let mut memory_manager = Arc::new(Mutex::from(VEMemoryManager::new(device.clone())));

        VEToolkit {
            window,
            device,
            swapchain,
            queue,
            command_pool,
            memory_manager,
        }
    }

    pub fn make_command_buffer(&self) -> VECommandBuffer {
        VECommandBuffer::new(self.device.clone(), self.command_pool.clone())
    }

    pub fn make_shader_module(&self, path: &str, typ: VEShaderModuleType) -> VEShaderModule {
        VEShaderModule::new(self.device.clone(), &mut fs::File::open(path).unwrap(), typ)
    }

    pub fn make_descriptor_set_layout(
        &self,
        fields: &[VEDescriptorSetLayoutField],
    ) -> VEDescriptorSetLayout {
        VEDescriptorSetLayout::new(self.device.clone(), fields)
    }

    pub fn make_image_full(
        &self,
        width: u32,
        height: u32,
        depth: u32,

        format: vk::Format,
        tiling: vk::ImageTiling,

        usage: vk::ImageUsageFlags,

        memory_properties: vk::MemoryPropertyFlags,
    ) -> VEImage {
        VEImage::from_full(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            width,
            height,
            depth,
            format,
            tiling,
            usage,
            memory_properties,
        )
    }

    pub fn make_image_from_data(
        &self,
        data: Vec<u8>,

        width: u32,
        height: u32,
        depth: u32,

        format: vk::Format,
        tiling: vk::ImageTiling,

        usage: vk::ImageUsageFlags,

        memory_properties: vk::MemoryPropertyFlags,
    ) -> VEImage {
        VEImage::from_data(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            data,
            width,
            height,
            depth,
            format,
            tiling,
            usage,
            memory_properties,
        )
    }

    pub fn make_image_from_file(&self, path: &str, usage: vk::ImageUsageFlags) -> VEImage {
        VEImage::from_file(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            path,
            usage,
        )
    }
}
