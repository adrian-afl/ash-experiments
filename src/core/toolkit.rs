use crate::buffer::buffer::{VEBuffer, VEBufferError, VEBufferType};
use crate::compute::compute_stage::{VEComputeStage, VEComputeStageError};
use crate::core::command_buffer::{VECommandBuffer, VECommandBufferError};
use crate::core::command_pool::{VECommandPool, VECommandPoolError};
use crate::core::descriptor_set_layout::{
    VEDescriptorSetLayout, VEDescriptorSetLayoutError, VEDescriptorSetLayoutField,
};
use crate::core::device::{VEDevice, VEDeviceError};
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::memory_properties::VEMemoryProperties;
use crate::core::scheduler::VEScheduler;
use crate::core::semaphore::{VESemaphore, VESemaphoreError};
use crate::core::shader_module::{VEShaderModule, VEShaderModuleError, VEShaderModuleType};
use crate::graphics::attachment::VEAttachment;
use crate::graphics::render_stage::{
    VECullMode, VEPrimitiveTopology, VERenderStage, VERenderStageError,
};
use crate::graphics::vertex_attributes::VertexAttribFormat;
use crate::graphics::vertex_buffer::{VEVertexBuffer, VEVertexBufferError};
use crate::image::filtering::VEFiltering;
use crate::image::image::{VEImage, VEImageError, VEImageUsage};
use crate::image::image_format::VEImageFormat;
use crate::image::sampler::{VESampler, VESamplerAddressMode, VESamplerError};
use crate::memory::memory_manager::VEMemoryManager;
use crate::window::swapchain::{VESwapchain, VESwapchainError};
use crate::window::window::{AppCallback, VEWindow, VEWindowError};
use ash::vk;
use std::sync::{Arc, LockResult, Mutex};
use std::{fs, io};
use thiserror::Error;
use winit::dpi::PhysicalSize;
use winit::window::WindowAttributes;

#[derive(Error, Debug)]
pub enum VEToolkitError {
    #[error("device error")]
    DeviceError(#[from] VEDeviceError),

    #[error("window error")]
    WindowError(#[from] VEWindowError),

    #[error("swapchain error")]
    SwapchainError(#[from] VESwapchainError),

    #[error("command pool error")]
    CommandPoolError(#[from] VECommandPoolError),

    #[error("callbacks locking failed")]
    CallbacksLockingFailed,
}

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
        let toolkit = self.toolkit.as_ref();
        match toolkit {
            None => println!("Cannot get self.toolkit in Toolkit AppCallback!"),
            Some(toolkit) => {
                let app = constructor(toolkit);
                self.app = Some(app);
            }
        }
    }

    fn on_window_draw(&self) {
        let app = self.app.as_ref();
        match app {
            None => println!("Cannot get self.app in Toolkit AppCallback!"),
            Some(app) => {
                let app = app.lock();
                match app {
                    Ok(mut app) => {
                        let toolkit = self.toolkit.as_ref();
                        match toolkit {
                            None => println!("Cannot get self.toolkit in Toolkit AppCallback!"),
                            Some(toolkit) => app.draw(toolkit),
                        }
                    }
                    Err(error) => println!("Could not lock app mutex! Reason: {:?}", error),
                }
            }
        }
    }

    fn on_window_resize(&self, new_size: PhysicalSize<u32>) {
        let toolkit = self.toolkit.as_ref();
        match toolkit {
            None => println!("Cannot get self.toolkit in Toolkit AppCallback!"),
            Some(toolkit) => match toolkit.device.wait_idle() {
                Ok(_) => {
                    let swapchain = toolkit.swapchain.lock();
                    match swapchain {
                        Ok(mut swapchain) => match swapchain.recreate(new_size) {
                            Ok(_) => (),
                            Err(error) => {
                                println!("Cannot recreate Swapchain! Reason: {:?}", error)
                            }
                        },
                        Err(error) => println!("Cannot lock Swapchain! Reason: {:?}", error),
                    }
                }
                Err(error) => println!("Cannot wait idle on Device! Reason: {:?}", error),
            },
        }
    }
}

impl VEToolkit {
    pub fn start(
        create_app: Box<dyn Fn(&VEToolkit) -> Arc<Mutex<dyn App>>>,
        initial_window_attributes: WindowAttributes,
    ) -> Result<(), VEToolkitError> {
        let callbacks = Arc::new(Mutex::from(VEToolkitCallbacks {
            toolkit: None,
            window: None,
            app: None,
            create_app,
        }));
        callbacks
            .lock()
            .map_err(|_| VEToolkitError::CallbacksLockingFailed)?
            .window = Some(Arc::new(VEWindow::new(
            callbacks.clone(),
            initial_window_attributes,
        )?));
        Ok(())
    }

    pub fn new(window: &VEWindow) -> Result<VEToolkit, VEToolkitError> {
        let device = Arc::new(VEDevice::new(&window)?);

        let mut memory_manager = Arc::new(Mutex::from(VEMemoryManager::new(device.clone())));

        let command_pool = Arc::new(VECommandPool::new(device.clone())?);

        let queue = Arc::new(VEMainDeviceQueue::new(device.clone()));

        let swapchain = Arc::new(Mutex::from(VESwapchain::new(
            &window,
            device.clone(),
            queue.clone(),
            command_pool.clone(),
        )?));

        Ok(VEToolkit {
            device,
            swapchain,
            queue,
            command_pool,
            memory_manager,
        })
    }

    pub fn create_command_buffer(&self) -> Result<VECommandBuffer, VECommandBufferError> {
        VECommandBuffer::new(self.device.clone(), self.command_pool.clone())
    }

    pub fn create_shader_module(
        &self,
        path: &str,
        typ: VEShaderModuleType,
    ) -> Result<VEShaderModule, VEShaderModuleError> {
        VEShaderModule::from_file(self.device.clone(), path, typ)
    }

    pub fn create_descriptor_set_layout(
        &self,
        fields: &[VEDescriptorSetLayoutField],
    ) -> Result<VEDescriptorSetLayout, VEDescriptorSetLayoutError> {
        VEDescriptorSetLayout::new(self.device.clone(), fields)
    }

    pub fn create_image_full(
        &self,
        width: u32,
        height: u32,
        depth: u32,

        format: VEImageFormat,

        usages: &[VEImageUsage],
    ) -> Result<VEImage, VEImageError> {
        VEImage::from_full(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            width,
            height,
            depth,
            format,
            usages,
        )
    }

    pub fn create_image_from_data(
        &self,
        data: Vec<u8>,

        width: u32,
        height: u32,
        depth: u32,

        format: VEImageFormat,

        usages: &[VEImageUsage],
    ) -> Result<VEImage, VEImageError> {
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
            usages,
        )
    }

    pub fn create_image_from_file(
        &self,
        path: &str,
        usages: &[VEImageUsage],
    ) -> Result<VEImage, VEImageError> {
        VEImage::from_file(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            path,
            usages,
        )
    }

    pub fn create_sampler(
        &self,
        sampler_address_mode: VESamplerAddressMode,

        min_filter: VEFiltering,
        mag_filter: VEFiltering,

        anisotropy: bool,
    ) -> Result<VESampler, VESamplerError> {
        VESampler::new(
            self.device.clone(),
            sampler_address_mode,
            min_filter,
            mag_filter,
            anisotropy,
        )
    }

    pub fn create_semaphore(&self) -> Result<VESemaphore, VESemaphoreError> {
        VESemaphore::new(self.device.clone())
    }

    pub fn create_buffer(
        &self,
        typ: VEBufferType,
        size: vk::DeviceSize,
        memory_properties: Option<VEMemoryProperties>,
    ) -> Result<VEBuffer, VEBufferError> {
        VEBuffer::new(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            typ,
            size,
            memory_properties,
        )
    }

    pub fn create_vertex_buffer(&self, buffer: VEBuffer, vertex_count: u32) -> VEVertexBuffer {
        VEVertexBuffer::new(self.device.clone(), buffer, vertex_count)
    }

    pub fn create_vertex_buffer_from_file(
        &self,
        path: &str,
        vertex_attributes: &[VertexAttribFormat],
    ) -> Result<VEVertexBuffer, VEVertexBufferError> {
        VEVertexBuffer::from_file(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            path,
            vertex_attributes,
        )
    }

    pub fn create_vertex_buffer_from_data(
        &self,
        data: Vec<u8>,
        vertex_attributes: &[VertexAttribFormat],
    ) -> Result<VEVertexBuffer, VEVertexBufferError> {
        VEVertexBuffer::from_data(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            data,
            vertex_attributes,
        )
    }

    pub fn create_compute_stage(
        &self,
        set_layouts: &[&VEDescriptorSetLayout],
        shader: &VEShaderModule,
    ) -> Result<VEComputeStage, VEComputeStageError> {
        VEComputeStage::new(
            self.device.clone(),
            self.command_pool.clone(),
            set_layouts,
            shader,
        )
    }

    pub fn create_render_stage(
        &self,
        viewport_width: u32,
        viewport_height: u32,
        attachments: &[&VEAttachment],
        set_layouts: &[&VEDescriptorSetLayout],
        vertex_shader: &VEShaderModule,
        fragment_shader: &VEShaderModule,
        vertex_attributes: &[VertexAttribFormat],
        primitive_topology: VEPrimitiveTopology,
        cull_mode: VECullMode,
    ) -> Result<VERenderStage, VERenderStageError> {
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

    pub fn create_scheduler(&self, layers_count: u8) -> VEScheduler {
        VEScheduler::new(
            self.device.clone(),
            self.swapchain.clone(),
            self.queue.clone(),
            layers_count,
        )
    }
}
