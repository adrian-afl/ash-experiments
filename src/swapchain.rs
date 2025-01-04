use crate::device::VEDevice;
use crate::main_device_queue::VEMainDeviceQueue;
use crate::window::VEWindow;
use ash::khr::swapchain;
use ash::vk::{
    ComponentMapping, ComponentSwizzle, Extent2D, ImageSubresourceRange, ImageViewCreateInfo,
    ImageViewType, PresentInfoKHR, PresentModeKHR, Semaphore, SurfaceCapabilitiesKHR,
    SurfaceFormatKHR, SwapchainKHR,
};
use ash::{vk, Device};
use std::sync::Arc;

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
    present_images: Vec<vk::Image>,
    present_image_views: Vec<vk::ImageView>,
    main_device_queue: Arc<VEMainDeviceQueue>,
}

impl VESwapchain {
    pub fn new(
        window: &VEWindow,
        device: Arc<VEDevice>,
        main_device_queue: Arc<VEMainDeviceQueue>,
    ) -> VESwapchain {
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
                width: window.window.inner_size().width,
                height: window.window.inner_size().height,
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
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
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

        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let present_image_views: Vec<vk::ImageView> = present_images
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

        VESwapchain {
            device,
            swapchain,
            swapchain_loader,
            present_images,
            present_image_views,
            main_device_queue,
        }
    }

    pub fn present(&self, wait_semaphores: &Vec<Semaphore>, image_index: u32) {
        let swapchains = [self.swapchain];
        let images = [image_index];
        let info = PresentInfoKHR::default()
            .wait_semaphores(wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&images);
        unsafe {
            self.swapchain_loader
                .queue_present(self.main_device_queue.main_queue, &info)
                .unwrap();
        }
    }
}
