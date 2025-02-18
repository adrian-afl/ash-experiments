use crate::core::command_buffer::{VECommandBuffer, VECommandBufferError};
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::semaphore::{SemaphoreState, VESemaphore, VESemaphoreError};
use crate::image::image::{VEImage, VEImageError};
use crate::window::window::VEWindow;
use ash::khr::swapchain;
use ash::vk;
use ash::vk::{CommandBufferUsageFlags, PresentInfoKHR, SwapchainKHR};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use winit::dpi::PhysicalSize;

#[derive(Error, Debug)]
pub enum VESwapchainError {
    #[error("no winit window found")]
    NoWinitWindowFound,

    #[error("acquire semaphore locking failed")]
    AcquireSemaphoreLockingFailed,

    #[error("blit semaphore locking failed")]
    BlitSemaphoreLockingFailed,

    #[error("window locking failed")]
    WindowLockingFailed,

    #[error("queue locking failed")]
    QueueLockingFailed,

    #[error("queue wait idle failed")]
    QueueWaitIdleFailed,

    #[error("semaphore error")]
    SemaphoreError(#[from] VESemaphoreError),

    #[error("image error")]
    ImageError(#[from] VEImageError),

    #[error("command buffer error")]
    CommandBufferError(#[from] VECommandBufferError),

    #[error("swapchain creation failed")]
    SwapchainCreationFailed(#[source] vk::Result),

    #[error("present failed")]
    PresentFailed(#[source] vk::Result),

    #[error("acquire failed")]
    AcquireFailed(#[source] vk::Result),

    #[error("cannot get physical device surface formats")]
    CannotGetPhysicalDeviceSurfaceFormats(#[source] vk::Result),

    #[error("cannot get physical device surface capabilities")]
    CannotGetPhysicalDeviceSurfaceCapabilities(#[source] vk::Result),

    #[error("cannot get physical device surface present modes")]
    CannotGetPhysicalDeviceSurfacePresentModes(#[source] vk::Result),

    #[error("cannot get swapchain images")]
    CannotGetSwapchainImages(#[source] vk::Result),
}

pub struct VESwapchain {
    device: Arc<VEDevice>,
    queue: Arc<Mutex<VEMainDeviceQueue>>,
    command_pool: Arc<VECommandPool>,

    swapchain: SwapchainKHR,
    swapchain_loader: swapchain::Device,
    pub present_images: Vec<VEImage>,
    pub width: u32,
    pub height: u32,

    acquire_ready_semaphore: Arc<Mutex<VESemaphore>>,
    pub blit_done_semaphore: Arc<Mutex<VESemaphore>>,
    present_command_buffer: VECommandBuffer,
}

impl Debug for VESwapchain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("VESwapchain")
    }
}

impl VESwapchain {
    pub fn new(
        window: &VEWindow,
        device: Arc<VEDevice>,
        queue: Arc<Mutex<VEMainDeviceQueue>>,
        command_pool: Arc<VECommandPool>,
    ) -> Result<VESwapchain, VESwapchainError> {
        let winit_window = window
            .window
            .as_ref()
            .ok_or(VESwapchainError::NoWinitWindowFound)?
            .lock()
            .map_err(|_| VESwapchainError::WindowLockingFailed)?;

        let (swapchain, swapchain_loader, present_images) = Self::create_swapchain_images(
            device.clone(),
            queue.clone(),
            command_pool.clone(),
            winit_window.inner_size(),
        )?;

        let acquire_ready_semaphore = VESemaphore::new(device.clone())?;
        let blit_done_semaphore = VESemaphore::new(device.clone())?;
        let present_command_buffer = VECommandBuffer::new(device.clone(), command_pool.clone())?;

        Ok(VESwapchain {
            device: device.clone(),
            swapchain,
            swapchain_loader,
            present_images,
            queue,
            command_pool,

            width: winit_window.inner_size().width,
            height: winit_window.inner_size().height,

            acquire_ready_semaphore: Arc::new(Mutex::from(acquire_ready_semaphore)),
            blit_done_semaphore: Arc::new(Mutex::from(blit_done_semaphore)),
            present_command_buffer,
        })
    }

    fn create_swapchain_images(
        device: Arc<VEDevice>,
        main_device_queue: Arc<Mutex<VEMainDeviceQueue>>,
        command_pool: Arc<VECommandPool>,
        new_size: PhysicalSize<u32>,
    ) -> Result<(SwapchainKHR, swapchain::Device, Vec<VEImage>), VESwapchainError> {
        let swapchain_loader = swapchain::Device::new(&device.instance, &device.device);

        let surface_format = unsafe {
            device
                .surface_loader
                .get_physical_device_surface_formats(device.physical_device, device.surface)
                .map_err(VESwapchainError::CannotGetPhysicalDeviceSurfaceFormats)?[0]
        };

        let surface_capabilities = unsafe {
            device
                .surface_loader
                .get_physical_device_surface_capabilities(device.physical_device, device.surface)
                .map_err(VESwapchainError::CannotGetPhysicalDeviceSurfaceCapabilities)?
        };
        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }
        let surface_resolution = match surface_capabilities.current_extent.width {
            u32::MAX => vk::Extent2D {
                width: new_size.width,
                height: new_size.height,
            },
            _ => surface_capabilities.current_extent,
        };
        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };
        let present_modes = unsafe {
            device
                .surface_loader
                .get_physical_device_surface_present_modes(device.physical_device, device.surface)
                .map_err(VESwapchainError::CannotGetPhysicalDeviceSurfacePresentModes)?
        };
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(device.surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .map_err(VESwapchainError::SwapchainCreationFailed)?
        };

        let present_images_raw = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .map_err(VESwapchainError::CannotGetSwapchainImages)?
        };

        let mut present_images = vec![];
        for i in 0..present_images_raw.len() {
            present_images.push(VEImage::from_swapchain_present_image(
                device.clone(),
                main_device_queue.clone(),
                command_pool.clone(),
                surface_resolution.width,
                surface_resolution.height,
                surface_format.format,
                present_images_raw[i],
            )?);
        }
        Ok((swapchain, swapchain_loader, present_images))
    }

    pub fn recreate(&mut self, new_size: PhysicalSize<u32>) -> Result<(), VESwapchainError> {
        self.queue
            .lock()
            .map_err(|_| VEImageError::QueueLockingFailed)?
            .wait_idle()
            .map_err(|_| VESwapchainError::QueueWaitIdleFailed)?;

        self.present_images.clear();

        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }

        let (swapchain, swapchain_loader, present_images) = Self::create_swapchain_images(
            self.device.clone(),
            self.queue.clone(),
            self.command_pool.clone(),
            new_size,
        )?;

        self.present_images = present_images;

        self.swapchain_loader = swapchain_loader;
        self.swapchain = swapchain;

        self.width = new_size.width;
        self.height = new_size.height;

        self.blit_done_semaphore
            .lock()
            .map_err(|_| VESwapchainError::BlitSemaphoreLockingFailed)?
            .recreate()?;
        self.acquire_ready_semaphore
            .lock()
            .map_err(|_| VESwapchainError::AcquireSemaphoreLockingFailed)?
            .recreate()?;

        self.queue
            .lock()
            .map_err(|_| VEImageError::QueueLockingFailed)?
            .wait_idle()
            .map_err(|_| VESwapchainError::QueueWaitIdleFailed)?;
        Ok(())
    }

    pub fn blit(
        &mut self,
        source: &VEImage,
        wait_for_semaphores: Vec<Arc<Mutex<VESemaphore>>>,
    ) -> Result<(), VESwapchainError> {
        self.acquire_ready_semaphore
            .lock()
            .map_err(|_| VESwapchainError::AcquireSemaphoreLockingFailed)?
            .state = SemaphoreState::Pending;
        let ack_semaphore = self.acquire_ready_semaphore.clone();
        let acquired = self.acquire_next_image(
            ack_semaphore
                .lock()
                .map_err(|_| VESwapchainError::AcquireSemaphoreLockingFailed)?
                .handle,
        )?;

        let blit_semaphore = &self.blit_done_semaphore;
        let mut wait_handles: Vec<Arc<Mutex<VESemaphore>>> = vec![];
        for item in wait_for_semaphores.iter() {
            wait_handles.push(item.clone());
        }

        wait_handles.push(ack_semaphore.clone());

        self.present_command_buffer.begin()?; // TODO try to remove this flag

        self.present_images[acquired as usize].transition_layout(
            &self.present_command_buffer,
            vk::ImageLayout::PRESENT_SRC_KHR,
            vk::ImageLayout::GENERAL,
        )?;

        let region = vk::ImageBlit::default()
            .src_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .src_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::default()
                    .x(source.width as i32)
                    .y(source.height as i32)
                    .z(1),
            ])
            .dst_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .dst_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::default()
                    .x(self.width as i32)
                    .y(self.height as i32)
                    .z(1),
            ]);

        unsafe {
            self.device.device.cmd_blit_image(
                self.present_command_buffer.handle,
                source.handle,
                vk::ImageLayout::GENERAL,
                self.present_images[acquired as usize].handle,
                vk::ImageLayout::GENERAL,
                &[region],
                vk::Filter::LINEAR,
            )
        }

        self.present_images[acquired as usize].transition_layout(
            &self.present_command_buffer,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::PRESENT_SRC_KHR,
        )?;

        self.present_command_buffer.end()?;

        {
            let queue = &self
                .queue
                .lock()
                .map_err(|_| VESwapchainError::QueueLockingFailed)?;

            self.present_command_buffer.submit(
                queue,
                wait_handles,
                vec![blit_semaphore.clone()],
            )?;
        }

        self.present(&[], acquired)?;

        Ok(())
    }

    fn present(
        &self,
        wait_handles: &[vk::Semaphore],
        image_index: u32,
    ) -> Result<(), VESwapchainError> {
        let swapchains = [self.swapchain];
        let images = [image_index];
        let info = PresentInfoKHR::default()
            .wait_semaphores(&wait_handles)
            .swapchains(&swapchains)
            .image_indices(&images);

        let queue = &self
            .queue
            .lock()
            .map_err(|_| VESwapchainError::QueueLockingFailed)?;

        unsafe {
            self.swapchain_loader
                .queue_present(queue.main_queue, &info)
                .map_err(VESwapchainError::PresentFailed)?;
        }
        Ok(())
    }

    fn acquire_next_image(&mut self, semaphore: vk::Semaphore) -> Result<u32, VESwapchainError> {
        let result = unsafe {
            self.swapchain_loader
                .acquire_next_image(self.swapchain, 2000, semaphore, vk::Fence::null())
                .map_err(VESwapchainError::AcquireFailed)?
        };
        Ok(result.0)
    }
}
