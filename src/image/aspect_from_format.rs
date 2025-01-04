use ash::vk;

pub fn aspect_from_format(format: vk::Format) -> vk::ImageAspectFlags {
    if format == vk::Format::D16_UNORM || format == vk::Format::D32_SFLOAT {
        vk::ImageAspectFlags::DEPTH
    } else {
        vk::ImageAspectFlags::COLOR
    }
}
