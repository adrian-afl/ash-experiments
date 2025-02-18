use crate::buffer::buffer::VEBufferError;
use crate::core::command_buffer::{VECommandBuffer, VECommandBufferError};
use crate::core::device::VEDevice;
use crate::core::main_device_queue::{VEMainDeviceQueue, VEMainDeviceQueueError};
use crate::image::transition_image_layout::transition_image_layout;
use crate::memory::memory_chunk::{VEMemoryChunkError, VESingleAllocation};
use crate::memory::memory_manager::VEMemoryManagerError;
use ash::vk;
use image::ImageError;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[path = "./image_from_data.rs"]
mod image_from_data;
#[path = "./image_from_file.rs"]
mod image_from_file;
#[path = "./image_from_full.rs"]
mod image_from_full;
#[path = "./image_from_swapchain.rs"]
mod image_from_swapchain;

#[derive(Error, Debug)]
pub enum VEImageError {
    #[error("image creation failed")]
    ImageCreationFailed(#[source] vk::Result),

    #[error("image view creation failed")]
    ImageViewCreationFailed(#[source] vk::Result),

    #[error("buffer error")]
    BufferError(#[from] VEBufferError),

    #[error("memory manager error")]
    MemoryManagerError(#[from] VEMemoryManagerError),

    #[error("memory chunk error")]
    MemoryChunkError(#[from] VEMemoryChunkError),

    #[error("command buffer error")]
    CommandBufferError(#[from] VECommandBufferError),

    #[error("main device query error")]
    MainDeviceQueueError(#[from] VEMainDeviceQueueError),

    #[error("opening file failed")]
    OpeningFileFailed(#[source] io::Error),

    #[error("image decoding failed")]
    ImageDecodingFailed(#[source] ImageError),

    #[error("memory manager locking failed")]
    MemoryManagerLockingFailed,

    #[error("no suitable memory type found")]
    NoSuitableMemoryTypeFound,

    #[error("queue locking failed")]
    QueueLockingFailed,
}

#[derive(Debug, Clone)]
pub enum VEImageUsage {
    ColorAttachment,
    DepthAttachment,
    Sampled,
    Storage,
    TransferDestination,
    TransferSource,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum VEImageViewType {
    View1D,
    View2D,
    View3D,
    ViewCube,
    View1DArray,
    View2DArray,
    ViewCubeArray,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct VEImageViewCreateInfo {
    pub typ: VEImageViewType,
    pub base_mipmap: u32,
    pub mipmap_count: u32,
    pub base_layer: u32,
    pub layer_count: u32,
}

impl VEImageViewCreateInfo {
    pub fn simple_2d() -> VEImageViewCreateInfo {
        VEImageViewCreateInfo {
            typ: VEImageViewType::View2D,
            base_layer: 0,
            layer_count: 1,
            base_mipmap: 0,
            mipmap_count: 1,
        }
    }

    pub fn simple_3d() -> VEImageViewCreateInfo {
        VEImageViewCreateInfo {
            typ: VEImageViewType::View3D,
            base_layer: 0,
            layer_count: 1,
            base_mipmap: 0,
            mipmap_count: 1,
        }
    }
}

#[derive(Clone)]
pub struct VEImage {
    device: Arc<VEDevice>,
    queue: Arc<Mutex<VEMainDeviceQueue>>,

    pub width: u32,
    pub height: u32,
    pub depth: u32,

    pub format: vk::Format,

    aspect: vk::ImageAspectFlags,

    pub current_layout: vk::ImageLayout,

    allocation: Option<VESingleAllocation>,
    pub handle: vk::Image,
    views: HashMap<VEImageViewCreateInfo, vk::ImageView>,
}

impl Debug for VEImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("VEImage")
    }
}

impl VEImage {
    pub fn is_depth(&self) -> bool {
        self.format == vk::Format::D16_UNORM || self.format == vk::Format::D32_SFLOAT
    }

    pub fn transition_layout(
        &mut self,
        command_buffer: &VECommandBuffer,
        from_layout: vk::ImageLayout,
        to_layout: vk::ImageLayout,
    ) -> Result<(), VEImageError> {
        transition_image_layout(
            self.device.clone(),
            command_buffer,
            self.handle,
            self.aspect,
            from_layout,
            to_layout,
        )?;

        self.current_layout = to_layout;

        Ok(())
    }

    pub fn get_view(&mut self, info: VEImageViewCreateInfo) -> Result<vk::ImageView, VEImageError> {
        let existing = self.views.get(&info);
        match existing {
            Some(view) => Ok(view.clone()),
            None => {
                let image_view_create_info = vk::ImageViewCreateInfo::default()
                    .image(self.handle)
                    .view_type(match info.typ {
                        VEImageViewType::View1D => vk::ImageViewType::TYPE_1D,
                        VEImageViewType::View2D => vk::ImageViewType::TYPE_2D,
                        VEImageViewType::View3D => vk::ImageViewType::TYPE_3D,
                        VEImageViewType::ViewCube => vk::ImageViewType::CUBE,
                        VEImageViewType::View1DArray => vk::ImageViewType::TYPE_1D_ARRAY,
                        VEImageViewType::View2DArray => vk::ImageViewType::TYPE_2D_ARRAY,
                        VEImageViewType::ViewCubeArray => vk::ImageViewType::CUBE_ARRAY,
                    })
                    .format(self.format)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(self.aspect)
                            .base_mip_level(info.base_mipmap)
                            .level_count(info.mipmap_count)
                            .base_array_layer(info.base_layer)
                            .layer_count(info.layer_count),
                    )
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    });

                let image_view_handle = unsafe {
                    self.device
                        .device
                        .create_image_view(&image_view_create_info, None)
                        .map_err(VEImageError::ImageViewCreationFailed)?
                };

                self.views.insert(info, image_view_handle.clone());
                Ok(image_view_handle)
            }
        }
    }
}

impl Drop for VEImage {
    fn drop(&mut self) {
        if let Some(_) = self.allocation {
            // only free the ones that app allocated, not swapchain, for example
            // probably this should be handled differently
            unsafe {
                for view in self.views.iter() {
                    self.device.device.destroy_image_view(*view.1, None);
                }
                self.device.device.destroy_image(self.handle, None);
            }
        }
    }
}
