use ash::vk;

pub enum VEImageFormat {
    R8inorm,
    RG8inorm,
    RGBA8inorm,

    R8unorm,
    RG8unorm,
    RGBA8unorm,

    R16i,
    RG16i,
    RGBA16i,

    R16u,
    RG16u,
    RGBA16u,

    R16f,
    RG16f,
    RGBA16f,

    R32i,
    RG32i,
    RGBA32i,

    R32u,
    RG32u,
    RGBA32u,

    R32f,
    RG32f,
    RGBA32f,

    Depth16u,
    Depth32f,
}

pub fn get_image_format(format: VEImageFormat) -> vk::Format {
    match format {
        VEImageFormat::R8inorm => vk::Format::R8_SNORM,
        VEImageFormat::RG8inorm => vk::Format::R8G8_SNORM,
        VEImageFormat::RGBA8inorm => vk::Format::R8G8B8A8_SNORM,

        VEImageFormat::R8unorm => vk::Format::R8_UNORM,
        VEImageFormat::RG8unorm => vk::Format::R8G8_UNORM,
        VEImageFormat::RGBA8unorm => vk::Format::R8G8B8A8_UNORM,

        VEImageFormat::R16i => vk::Format::R16_SINT,
        VEImageFormat::RG16i => vk::Format::R16G16_SINT,
        VEImageFormat::RGBA16i => vk::Format::R16G16B16A16_SINT,

        VEImageFormat::R16u => vk::Format::R16_UINT,
        VEImageFormat::RG16u => vk::Format::R16G16_UINT,
        VEImageFormat::RGBA16u => vk::Format::R16G16B16A16_UINT,

        VEImageFormat::R16f => vk::Format::R16_SFLOAT,
        VEImageFormat::RG16f => vk::Format::R16G16_SFLOAT,
        VEImageFormat::RGBA16f => vk::Format::R16G16B16A16_SFLOAT,

        VEImageFormat::R32i => vk::Format::R32_SINT,
        VEImageFormat::RG32i => vk::Format::R32G32_SINT,
        VEImageFormat::RGBA32i => vk::Format::R32G32B32A32_SINT,

        VEImageFormat::R32u => vk::Format::R32_UINT,
        VEImageFormat::RG32u => vk::Format::R32G32_UINT,
        VEImageFormat::RGBA32u => vk::Format::R32G32B32A32_UINT,

        VEImageFormat::R32f => vk::Format::R32_SFLOAT,
        VEImageFormat::RG32f => vk::Format::R32G32_SFLOAT,
        VEImageFormat::RGBA32f => vk::Format::R32G32B32A32_SFLOAT,

        VEImageFormat::Depth16u => vk::Format::D16_UNORM,
        VEImageFormat::Depth32f => vk::Format::D32_SFLOAT,
    }
}
