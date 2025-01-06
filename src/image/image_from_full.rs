use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::image::aspect_from_format::aspect_from_format;
use crate::image::image::VEImage;
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use std::sync::{Arc, Mutex};

impl VEImage {
    pub fn from_full(
        device: Arc<VEDevice>,
        queue: Arc<VEMainDeviceQueue>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,

        width: u32,
        height: u32,
        depth: u32,

        format: vk::Format,
        tiling: vk::ImageTiling,

        usage: vk::ImageUsageFlags,

        memory_properties: vk::MemoryPropertyFlags,
    ) -> VEImage {
        let aspect = aspect_from_format(format);

        let queue_family_indices = [device.queue_family_index];

        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(if depth == 1 {
                vk::ImageType::TYPE_2D
            } else {
                vk::ImageType::TYPE_3D
            })
            .extent(
                vk::Extent3D::default()
                    .width(width)
                    .height(height)
                    .depth(depth),
            )
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family_indices)
            .initial_layout(vk::ImageLayout::PREINITIALIZED);

        let image_handle = unsafe {
            device
                .device
                .create_image(&image_create_info, None)
                .unwrap()
        };

        let mem_reqs = unsafe { device.device.get_image_memory_requirements(image_handle) };
        let mem_index = device.find_memory_type(mem_reqs.memory_type_bits, memory_properties);

        let allocation = {
            memory_manager
                .lock()
                .unwrap()
                .bind_image_memory(mem_index, image_handle, mem_reqs.size)
        };

        let image_view_create_info = vk::ImageViewCreateInfo::default()
            .image(image_handle)
            .view_type(if depth == 1 {
                vk::ImageViewType::TYPE_2D
            } else {
                vk::ImageViewType::TYPE_3D
            })
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect)
                    .base_mip_level(0)
                    .level_count(1) // TODO MIPMAPPING
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY, // TODO identity?
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            });
        let image_view_handle = unsafe {
            device
                .device
                .create_image_view(&image_view_create_info, None)
                .unwrap()
        };

        let mut image = VEImage {
            device,
            queue,
            command_pool,
            memory_manager,

            allocation: Some(allocation),

            width,
            height,
            depth,

            format,
            tiling,

            usage,
            aspect,

            handle: image_handle,
            view: Some(image_view_handle),
            current_layout: vk::ImageLayout::PREINITIALIZED,
        };
        image.transition_layout(image.current_layout, vk::ImageLayout::GENERAL);

        image
    }
}
