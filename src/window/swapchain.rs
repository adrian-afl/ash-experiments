use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::semaphore::{SemaphoreState, VESemaphore};
use crate::image::image::VEImage;
use crate::memory::memory_manager::VEMemoryManager;
use crate::window::window::VEWindow;
use ash::khr::swapchain;
use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{
    CommandBufferUsageFlags, PresentInfoKHR, PresentModeKHR, SurfaceCapabilitiesKHR,
    SurfaceFormatKHR, SwapchainKHR,
};
use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use tracing::{event, instrument, Level};
use winit::dpi::PhysicalSize;

pub struct VESwapchain {
    device: Arc<VEDevice>,
    main_device_queue: Arc<VEMainDeviceQueue>,
    command_pool: Arc<VECommandPool>,
    memory_manager: Arc<Mutex<VEMemoryManager>>,

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
    #[instrument]
    pub fn new(
        window: &VEWindow,
        device: Arc<VEDevice>,
        main_device_queue: Arc<VEMainDeviceQueue>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
    ) -> VESwapchain {
        let winit_window = window.window.as_ref().unwrap();

        let (swapchain, swapchain_loader, present_images) = Self::create_swapchain_images(
            device.clone(),
            main_device_queue.clone(),
            command_pool.clone(),
            memory_manager.clone(),
            winit_window.inner_size(),
        );

        VESwapchain {
            device: device.clone(),
            swapchain,
            swapchain_loader,
            present_images,
            main_device_queue,
            command_pool: command_pool.clone(),
            memory_manager,

            width: winit_window.inner_size().width,
            height: winit_window.inner_size().height,

            acquire_ready_semaphore: Arc::new(Mutex::from(VESemaphore::new(device.clone()))),
            blit_done_semaphore: Arc::new(Mutex::from(VESemaphore::new(device.clone()))),
            present_command_buffer: VECommandBuffer::new(device.clone(), command_pool),
        }
    }

    fn create_swapchain_images(
        device: Arc<VEDevice>,
        main_device_queue: Arc<VEMainDeviceQueue>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        new_size: PhysicalSize<u32>,
    ) -> (SwapchainKHR, swapchain::Device, Vec<VEImage>) {
        let swapchain_loader = swapchain::Device::new(&device.instance, &device.device);
        // let winit_window = window.window.as_ref().unwrap();
        let surface_format = unsafe {
            device
                .surface_loader
                .get_physical_device_surface_formats(device.physical_device, device.surface)
                .unwrap()[0]
        };

        let surface_capabilities = unsafe {
            device
                .surface_loader
                .get_physical_device_surface_capabilities(device.physical_device, device.surface)
                .unwrap()
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
                .unwrap()
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
                .unwrap()
        };

        let present_images_raw =
            unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };

        let mut present_images = vec![];
        for i in 0..present_images_raw.len() {
            present_images.push(VEImage::from_swapchain_present_image(
                device.clone(),
                main_device_queue.clone(),
                command_pool.clone(),
                memory_manager.clone(),
                surface_resolution.width,
                surface_resolution.height,
                surface_format.format,
                present_images_raw[i],
            ))
        }
        (swapchain, swapchain_loader, present_images)
    }

    #[instrument]
    pub fn recreate(&mut self, new_size: PhysicalSize<u32>) {
        self.present_images.clear();

        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }

        let (swapchain, swapchain_loader, present_images) = Self::create_swapchain_images(
            self.device.clone(),
            self.main_device_queue.clone(),
            self.command_pool.clone(),
            self.memory_manager.clone(),
            new_size,
        );

        self.present_images = present_images;

        self.swapchain_loader = swapchain_loader;
        self.swapchain = swapchain;
        println!("new size 2 {:?}", new_size);
        self.width = new_size.width;
        self.height = new_size.height;

        self.blit_done_semaphore.lock().unwrap().recreate();
        self.acquire_ready_semaphore.lock().unwrap().recreate();
    }

    #[instrument]
    pub fn blit(&mut self, source: &VEImage, wait_for_semaphores: Vec<Arc<Mutex<VESemaphore>>>) {
        self.acquire_ready_semaphore.lock().unwrap().state = SemaphoreState::Pending;
        event!(
            Level::TRACE,
            "Setting semaphore acquire_ready_semaphore to Pending"
        );
        let ack_semaphore = self.acquire_ready_semaphore.clone();
        let acquired = self.acquire_next_image(ack_semaphore.lock().unwrap().handle);

        let blit_semaphore = &self.blit_done_semaphore;
        let mut wait_handles: Vec<Arc<Mutex<VESemaphore>>> = vec![];
        for item in wait_for_semaphores.iter() {
            wait_handles.push(item.clone());
        }

        wait_handles.push(ack_semaphore.clone());

        self.present_images[acquired as usize]
            .transition_layout(vk::ImageLayout::PRESENT_SRC_KHR, vk::ImageLayout::GENERAL);

        self.present_command_buffer
            .begin(CommandBufferUsageFlags::SIMULTANEOUS_USE); // TODO try to remove this flag

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

        self.present_command_buffer.end();
        self.present_command_buffer.submit(
            &self.main_device_queue,
            wait_handles,
            vec![blit_semaphore.clone()],
        );

        self.present_images[acquired as usize]
            .transition_layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::PRESENT_SRC_KHR);

        self.present(&[], acquired);
    }

    #[instrument]
    fn present(&self, wait_handles: &[vk::Semaphore], image_index: u32) {
        let swapchains = [self.swapchain];
        let images = [image_index];
        let info = PresentInfoKHR::default()
            .wait_semaphores(&wait_handles)
            .swapchains(&swapchains)
            .image_indices(&images);
        unsafe {
            let result = self
                .swapchain_loader
                .queue_present(self.main_device_queue.main_queue, &info);
            match result {
                Ok(_) => (),
                Err(e) => (), //println!("Swapchain lost at present, {:?}", e),
            }
        }
    }

    #[instrument]
    fn acquire_next_image(&mut self, semaphore: vk::Semaphore) -> u32 {
        let result = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                2000,
                semaphore,
                vk::Fence::null(),
            )
        };
        match result {
            Ok(res) => res.0,
            Err(e) => {
                // println!("Swapchain lost at acquire, {:?}, width {}", e, self.width);
                // if let Some(window) = self.window.as_ref() {
                //     self.recreate(window.clone());
                // }
                0
            }
        }
    }
}
