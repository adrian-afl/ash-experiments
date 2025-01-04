use ash::vk;

#[derive(Clone, Copy, Debug)]
pub enum VertexAttribFormat {
    R8inorm,
    RG8inorm,
    RGB8inorm,
    RGBA8inorm,

    R8unorm,
    RG8unorm,
    RGB8unorm,
    RGBA8unorm,

    R16i,
    RG16i,
    RGB16i,
    RGBA16i,

    R16u,
    RG16u,
    RGB16u,
    RGBA16u,

    R16f,
    RG16f,
    RGB16f,
    RGBA16f,

    R32i,
    RG32i,
    RGB32i,
    RGBA32i,

    R32u,
    RG32u,
    RGB32u,
    RGBA32u,

    R32f,
    RG32f,
    RGB32f,
    RGBA32f,

    Padding8,
    Padding16,
    Padding24,
    Padding32,
}

fn get_vertex_attribute_type_byte_size(attrib: &VertexAttribFormat) -> u32 {
    match attrib {
        VertexAttribFormat::R8inorm => 1,
        VertexAttribFormat::RG8inorm => 2,
        VertexAttribFormat::RGB8inorm => 3,
        VertexAttribFormat::RGBA8inorm => 4,
        VertexAttribFormat::R8unorm => 1,
        VertexAttribFormat::RG8unorm => 2,
        VertexAttribFormat::RGB8unorm => 3,
        VertexAttribFormat::RGBA8unorm => 4,
        VertexAttribFormat::R16i => 2,
        VertexAttribFormat::RG16i => 4,
        VertexAttribFormat::RGB16i => 6,
        VertexAttribFormat::RGBA16i => 8,
        VertexAttribFormat::R16u => 2,
        VertexAttribFormat::RG16u => 4,
        VertexAttribFormat::RGB16u => 6,
        VertexAttribFormat::RGBA16u => 8,
        VertexAttribFormat::R16f => 2,
        VertexAttribFormat::RG16f => 4,
        VertexAttribFormat::RGB16f => 6,
        VertexAttribFormat::RGBA16f => 8,
        VertexAttribFormat::R32i => 4,
        VertexAttribFormat::RG32i => 8,
        VertexAttribFormat::RGB32i => 12,
        VertexAttribFormat::RGBA32i => 16,
        VertexAttribFormat::R32u => 4,
        VertexAttribFormat::RG32u => 8,
        VertexAttribFormat::RGB32u => 12,
        VertexAttribFormat::RGBA32u => 16,
        VertexAttribFormat::R32f => 4,
        VertexAttribFormat::RG32f => 8,
        VertexAttribFormat::RGB32f => 12,
        VertexAttribFormat::RGBA32f => 16,
        VertexAttribFormat::Padding8 => 1,
        VertexAttribFormat::Padding16 => 2,
        VertexAttribFormat::Padding24 => 4,
        VertexAttribFormat::Padding32 => 8,
    }
}

fn resolve_vertex_attribute_format(attrib: &VertexAttribFormat) -> vk::Format {
    match attrib {
        VertexAttribFormat::R8inorm => vk::Format::R8_SNORM,
        VertexAttribFormat::RG8inorm => vk::Format::R8G8_SNORM,
        VertexAttribFormat::RGB8inorm => vk::Format::R8G8B8_SNORM,
        VertexAttribFormat::RGBA8inorm => vk::Format::R8G8B8A8_SNORM,
        VertexAttribFormat::R8unorm => vk::Format::R8_UNORM,
        VertexAttribFormat::RG8unorm => vk::Format::R8G8_UNORM,
        VertexAttribFormat::RGB8unorm => vk::Format::R8G8B8_UNORM,
        VertexAttribFormat::RGBA8unorm => vk::Format::R8G8B8A8_UNORM,
        VertexAttribFormat::R16i => vk::Format::R16_SINT,
        VertexAttribFormat::RG16i => vk::Format::R16G16_SINT,
        VertexAttribFormat::RGB16i => vk::Format::R16G16B16_SINT,
        VertexAttribFormat::RGBA16i => vk::Format::R16G16B16A16_SINT,
        VertexAttribFormat::R16u => vk::Format::R16_UINT,
        VertexAttribFormat::RG16u => vk::Format::R16G16_UINT,
        VertexAttribFormat::RGB16u => vk::Format::R16G16B16_UINT,
        VertexAttribFormat::RGBA16u => vk::Format::R16G16B16A16_UINT,
        VertexAttribFormat::R16f => vk::Format::R16_SFLOAT,
        VertexAttribFormat::RG16f => vk::Format::R16G16_SFLOAT,
        VertexAttribFormat::RGB16f => vk::Format::R16G16B16_SFLOAT,
        VertexAttribFormat::RGBA16f => vk::Format::R16G16B16A16_SFLOAT,
        VertexAttribFormat::R32i => vk::Format::R32_SINT,
        VertexAttribFormat::RG32i => vk::Format::R32G32_SINT,
        VertexAttribFormat::RGBA32i => vk::Format::R32G32B32A32_SINT,
        VertexAttribFormat::RGB32i => vk::Format::R32G32B32_SINT,
        VertexAttribFormat::R32u => vk::Format::R32_UINT,
        VertexAttribFormat::RG32u => vk::Format::R32G32_UINT,
        VertexAttribFormat::RGB32u => vk::Format::R32G32B32_UINT,
        VertexAttribFormat::RGBA32u => vk::Format::R32G32B32A32_UINT,
        VertexAttribFormat::R32f => vk::Format::R32_SFLOAT,
        VertexAttribFormat::RG32f => vk::Format::R32G32_SFLOAT,
        VertexAttribFormat::RGB32f => vk::Format::R32G32B32_SFLOAT,
        VertexAttribFormat::RGBA32f => vk::Format::R32G32B32A32_SFLOAT,
        _ => vk::Format::R8_SNORM,
    }
}

fn is_offset(attrib: &VertexAttribFormat) -> bool {
    match attrib {
        VertexAttribFormat::Padding8 => true,
        VertexAttribFormat::Padding16 => true,
        VertexAttribFormat::Padding24 => true,
        VertexAttribFormat::Padding32 => true,
        _ => false,
    }
}

pub fn create_vertex_input_state_descriptions(
    attribs: &[VertexAttribFormat],
) -> (
    vk::VertexInputBindingDescription,
    Vec<vk::VertexInputAttributeDescription>,
) {
    let stride: u32 = attribs
        .iter()
        .map(|a| get_vertex_attribute_type_byte_size(a))
        .sum();

    println!("stride: {stride}");

    if (stride % 4 != 0) {
        panic!("Stride not dividable by 4");
    }

    let binding_desc = vk::VertexInputBindingDescription::default()
        .stride(stride)
        .binding(0)
        .input_rate(vk::VertexInputRate::VERTEX);

    let mut offset = 0;
    let mut location = 0;
    let mut descriptions: Vec<vk::VertexInputAttributeDescription> = vec![];

    for attrib in attribs {
        if !is_offset(attrib) {
            let desc = vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(location)
                .format(resolve_vertex_attribute_format(attrib))
                .offset(offset);
            println!(
                "location: {location} offset {offset} format {:?}",
                resolve_vertex_attribute_format(attrib)
            );
            descriptions.push(desc);
            location += 1;
        }
        offset += get_vertex_attribute_type_byte_size(attrib);
    }
    (binding_desc, descriptions)
}
