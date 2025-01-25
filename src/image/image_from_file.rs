use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::image::image::{VEImage, VEImageError, VEImageUsage};
use crate::image::image_format::VEImageFormat;
use crate::memory::memory_manager::VEMemoryManager;
use image::{EncodableLayout, ImageReader};
use std::sync::{Arc, Mutex};

impl VEImage {
    pub fn from_file(
        device: Arc<VEDevice>,
        queue: Arc<Mutex<VEMainDeviceQueue>>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        path: &str,
        usages: &[VEImageUsage],
    ) -> Result<VEImage, VEImageError> {
        let img = ImageReader::open(path)
            .map_err(VEImageError::OpeningFileFailed)?
            .decode()
            .map_err(VEImageError::ImageDecodingFailed)?;
        let img = img.to_rgba8(); // error handling... TODO
                                  // let format = match img.color() {
                                  //     ColorType::L8 => vk::Format::R8_UNORM,
                                  //     ColorType::La8 => vk::Format::R8G8_UNORM,
                                  //     ColorType::L16 => vk::Format::R16_UNORM,
                                  //     ColorType::La16 => vk::Format::R16G16_UNORM,
                                  //     ColorType::Rgb8 => vk::Format::R8G8B8_UNORM,
                                  //     ColorType::Rgba8 => vk::Format::R8G8B8A8_UNORM,
                                  //     ColorType::Rgb16 => vk::Format::R16G16B16_UNORM,
                                  //     ColorType::Rgba16 => vk::Format::R16G16B16A16_UNORM,
                                  //     ColorType::Rgb32F => vk::Format::R32G32B32_SFLOAT,
                                  //     ColorType::Rgba32F => vk::Format::R32G32B32A32_SFLOAT,
                                  //     _ => panic!("Unknown format"),
                                  // };
        let format = VEImageFormat::RGBA8unorm;
        VEImage::from_data(
            device,
            queue,
            command_pool,
            memory_manager,
            Vec::from(img.as_bytes()),
            img.width(),
            img.height(),
            1,
            format,
            usages,
        )
    }
}
