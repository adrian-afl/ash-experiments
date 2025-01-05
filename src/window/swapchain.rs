use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::semaphore::VESemaphore;
use crate::image::image::VEImage;
use crate::memory::memory_manager::VEMemoryManager;
use crate::window::window::VEWindow;
use ash::khr::swapchain;
use ash::vk;
use ash::vk::{
    CommandBufferUsageFlags, PresentInfoKHR, PresentModeKHR, SurfaceCapabilitiesKHR,
    SurfaceFormatKHR, SwapchainKHR,
};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use tracing::instrument;

struct SwapChainSupportDetails {
    surface_capabilities: SurfaceCapabilitiesKHR,
    formats: Vec<SurfaceFormatKHR>,
    present_modes: Vec<PresentModeKHR>,
}

impl SwapChainSupportDetails {
    fn default() -> Self {
        Self {
            surface_capabilities: SurfaceCapabilitiesKHR::default(),
            formats: Vec::new(),
            present_modes: Vec::new(),
        }
    }
}

pub struct VESwapchain {
    device: Arc<VEDevice>,
    swapchain: SwapchainKHR,
    swapchain_loader: swapchain::Device,
    main_device_queue: Arc<VEMainDeviceQueue>,

    pub present_images: Vec<VEImage>,
    pub width: u32,
    pub height: u32,

    acquire_ready_semaphore: VESemaphore,
    pub blit_done_semaphore: VESemaphore,
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
                width: winit_window.inner_size().width,
                height: winit_window.inner_size().height,
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
        let swapchain_loader = swapchain::Device::new(&device.instance, &device.device);

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
        let present_image_views_raw: Vec<vk::ImageView> = present_images_raw
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::default()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);
                unsafe {
                    device
                        .device
                        .create_image_view(&create_view_info, None)
                        .unwrap()
                }
            })
            .collect();

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
                present_image_views_raw[i],
            ))
        }

        VESwapchain {
            device: device.clone(),
            swapchain,
            swapchain_loader,
            present_images,
            main_device_queue,

            width: surface_resolution.width,
            height: surface_resolution.height,

            acquire_ready_semaphore: VESemaphore::new(device.clone()),
            blit_done_semaphore: VESemaphore::new(device.clone()),
            present_command_buffer: VECommandBuffer::new(device.clone(), command_pool),
        }
    }

    #[instrument]
    pub fn blit(&mut self, source: &VEImage, wait_for_semaphores: &[&VESemaphore]) {
        let acquired = self.acquire_next_image(self.acquire_ready_semaphore.handle);

        let ack_semaphore = &self.acquire_ready_semaphore;
        let blit_semaphore = &self.blit_done_semaphore;
        let mut wait_handles: Vec<&VESemaphore> = Vec::from(wait_for_semaphores);
        wait_handles.push(ack_semaphore);

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
                source.current_layout,
                self.present_images[acquired as usize].handle,
                vk::ImageLayout::GENERAL,
                &[region],
                vk::Filter::LINEAR,
            )
        }

        self.present_command_buffer.end();
        self.present_command_buffer.submit(
            &self.main_device_queue,
            &wait_handles,
            &[blit_semaphore],
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
            self.swapchain_loader
                .queue_present(self.main_device_queue.main_queue, &info)
                .unwrap();
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
        result.unwrap().0
    }
}
