use crate::device::VEDevice;
use crate::window::VEWindow;
use ash::khr::swapchain;
use ash::vk;
use ash::vk::SwapchainKHR;

pub struct VESwapchain<'a> {
    device: &'a VEDevice,
    pub swapchain: SwapchainKHR,
}

impl<'a> VESwapchain<'a> {
    pub fn new(window: &VEWindow, device: &'a VEDevice) -> VESwapchain<'a> {
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

        VESwapchain { device, swapchain }
    }
}
