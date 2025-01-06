use crate::buffer::buffer::{VEBuffer, VEBufferType};
use crate::compute::compute_stage::VEComputeStage;
use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::descriptor_set_layout::{VEDescriptorSetLayout, VEDescriptorSetLayoutField};
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::scheduler::VEScheduler;
use crate::core::semaphore::VESemaphore;
use crate::core::shader_module::{VEShaderModule, VEShaderModuleType};
use crate::graphics::attachment::VEAttachment;
use crate::graphics::render_stage::{CullMode, VERenderStage};
use crate::graphics::vertex_attributes::VertexAttribFormat;
use crate::graphics::vertex_buffer::VEVertexBuffer;
use crate::image::image::VEImage;
use crate::image::sampler::VESampler;
use crate::memory::memory_manager::VEMemoryManager;
use crate::window::swapchain::VESwapchain;
use crate::window::window::{AppCallback, VEWindow};
use ash::vk;
use std::sync::{Arc, Mutex};
use std::{fs, io};
use winit::dpi::PhysicalSize;
use winit::window::WindowAttributes;

pub trait App {
    fn draw(&mut self, toolkit: &VEToolkit);
}

pub struct VEToolkit {
    device: Arc<VEDevice>,
    pub swapchain: Arc<Mutex<VESwapchain>>,
    pub queue: Arc<VEMainDeviceQueue>, // TODO maybe this could be made private
    command_pool: Arc<VECommandPool>,
    memory_manager: Arc<Mutex<VEMemoryManager>>,
}

pub struct VEToolkitCallbacks {
    toolkit: Option<VEToolkit>,
    pub window: Option<Arc<VEWindow>>,
    create_app: Box<dyn Fn(&VEToolkit) -> Arc<Mutex<dyn App>>>,
    app: Option<Arc<Mutex<dyn App>>>,
}

impl AppCallback for VEToolkitCallbacks {
    fn on_window_ready(&mut self, toolkit: VEToolkit) {
        self.toolkit = Some(toolkit);
        let constructor = &self.create_app;
        let app = constructor(self.toolkit.as_ref().unwrap());
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

    fn on_window_resize(&self, new_size: PhysicalSize<u32>) {
        println!("new size {:?}", new_size);
        self.toolkit.as_ref().unwrap().device.wait_idle();
        println!("Is window {:?}", self.window);
        self.toolkit
            .as_ref()
            .unwrap()
            .swapchain
            .lock()
            .unwrap()
            .recreate(new_size);
    }
}

impl VEToolkit {
    pub fn start(
        create_app: Box<dyn Fn(&VEToolkit) -> Arc<Mutex<dyn App>>>,
        initial_window_attributes: WindowAttributes,
    ) {
        let callbacks = Arc::new(Mutex::from(VEToolkitCallbacks {
            toolkit: None,
            window: None,
            app: None,
            create_app,
        }));
        println!("Setting window");
        callbacks.lock().unwrap().window = Some(Arc::new(VEWindow::new(
            callbacks.clone(),
            initial_window_attributes,
        ))); //oh god i hope this won't race condition with event loop
    }

    pub fn new(window: &VEWindow) -> VEToolkit {
        let device = Arc::new(VEDevice::new(&window));

        let mut memory_manager = Arc::new(Mutex::from(VEMemoryManager::new(device.clone())));

        let command_pool = Arc::new(VECommandPool::new(device.clone()));

        let queue = Arc::new(VEMainDeviceQueue::new(device.clone()));

        let swapchain = Arc::new(Mutex::from(VESwapchain::new(
            &window,
            device.clone(),
            queue.clone(),
            command_pool.clone(),
            memory_manager.clone(),
        )));

        VEToolkit {
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

    pub fn make_sampler(
        &self,
        sampler_address_mode: vk::SamplerAddressMode,

        min_filter: vk::Filter,
        mag_filter: vk::Filter,

        anisotropy: bool,
    ) -> VESampler {
        VESampler::new(
            self.device.clone(),
            sampler_address_mode,
            min_filter,
            mag_filter,
            anisotropy,
        )
    }

    pub fn make_semaphore(&self) -> VESemaphore {
        VESemaphore::new(self.device.clone())
    }

    pub fn make_buffer(
        &self,
        typ: VEBufferType,
        size: vk::DeviceSize,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> VEBuffer {
        VEBuffer::new(
            self.device.clone(),
            self.memory_manager.clone(),
            typ,
            size,
            memory_properties,
        )
    }

    pub fn make_vertex_buffer(&self, buffer: VEBuffer, vertex_count: u32) -> VEVertexBuffer {
        VEVertexBuffer::new(self.device.clone(), buffer, vertex_count)
    }

    pub fn make_vertex_buffer_from_file(
        &self,
        path: &str,
        vertex_attributes: &[VertexAttribFormat],
    ) -> VEVertexBuffer {
        VEVertexBuffer::from_file(
            self.device.clone(),
            self.memory_manager.clone(),
            path,
            vertex_attributes,
        )
    }

    pub fn make_compute_stage(
        &self,
        set_layouts: &[&VEDescriptorSetLayout],
        shader: &VEShaderModule,
    ) -> VEComputeStage {
        VEComputeStage::new(
            self.device.clone(),
            self.command_pool.clone(),
            set_layouts,
            shader,
        )
    }

    pub fn make_render_stage(
        &self,
        viewport_width: u32,
        viewport_height: u32,
        attachments: &[&VEAttachment],
        set_layouts: &[&VEDescriptorSetLayout],
        vertex_shader: &VEShaderModule,
        fragment_shader: &VEShaderModule,
        vertex_attributes: &[VertexAttribFormat],
        primitive_topology: vk::PrimitiveTopology,
        cull_mode: CullMode,
    ) -> VERenderStage {
        VERenderStage::new(
            self.device.clone(),
            self.command_pool.clone(),
            viewport_width,
            viewport_height,
            attachments,
            set_layouts,
            vertex_shader,
            fragment_shader,
            vertex_attributes,
            primitive_topology,
            cull_mode,
        )
    }

    pub fn make_scheduler(&self, layers_count: u8) -> VEScheduler {
        VEScheduler::new(
            self.device.clone(),
            self.swapchain.clone(),
            self.queue.clone(),
            layers_count,
        )
    }
}
