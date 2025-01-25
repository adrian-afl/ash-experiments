use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::memory_properties::{get_memory_properties_flags, VEMemoryProperties};
use crate::image::aspect_from_format::aspect_from_format;
use crate::image::image::{VEImage, VEImageError, VEImageUsage};
use crate::image::image_format::{get_image_format, VEImageFormat};
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn get_image_usage_flags(usages: &[VEImageUsage]) -> vk::ImageUsageFlags {
    let mut flags = vk::ImageUsageFlags::empty();
    for usage in usages {
        match usage {
            VEImageUsage::ColorAttachment => flags = flags | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            VEImageUsage::DepthAttachment => {
                flags = flags | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
            }
            VEImageUsage::Sampled => flags = flags | vk::ImageUsageFlags::SAMPLED,
            VEImageUsage::Storage => flags = flags | vk::ImageUsageFlags::STORAGE,
            VEImageUsage::TransferDestination => flags = flags | vk::ImageUsageFlags::TRANSFER_DST,
            VEImageUsage::TransferSource => flags = flags | vk::ImageUsageFlags::TRANSFER_SRC,
        }
    }
    flags
}

impl VEImage {
    pub fn from_full(
        device: Arc<VEDevice>,
        queue: Arc<Mutex<VEMainDeviceQueue>>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,

        width: u32,
        height: u32,
        depth: u32,

        format: VEImageFormat,

        usages: &[VEImageUsage],
    ) -> Result<VEImage, VEImageError> {
        let format = get_image_format(format);
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
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(get_image_usage_flags(usages))
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family_indices)
            .initial_layout(vk::ImageLayout::PREINITIALIZED);

        let image_handle = unsafe {
            device
                .device
                .create_image(&image_create_info, None)
                .map_err(VEImageError::ImageCreationFailed)?
        };

        let mem_reqs = unsafe { device.device.get_image_memory_requirements(image_handle) };
        let mem_index = device.find_memory_type(
            mem_reqs.memory_type_bits,
            get_memory_properties_flags(Some(VEMemoryProperties::DeviceLocal)),
        );

        let allocation = match mem_index {
            None => return Err(VEImageError::NoSuitableMemoryTypeFound),
            Some(mem_index) => memory_manager
                .lock()
                .map_err(|_| VEImageError::MemoryManagerLockingFailed)?
                .bind_image_memory(mem_index, image_handle, mem_reqs.size)?,
        };

        let mut image = VEImage {
            device,
            queue,
            command_pool,

            allocation: Some(allocation),

            width,
            height,
            depth,

            format,

            aspect,

            handle: image_handle,
            views: HashMap::new(),
            current_layout: vk::ImageLayout::PREINITIALIZED,
        };
        image.transition_layout(image.current_layout, vk::ImageLayout::GENERAL)?;

        Ok(image)
    }
}
