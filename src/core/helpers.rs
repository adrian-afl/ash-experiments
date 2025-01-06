use ash::vk;

pub fn make_clear_color_f32(values: [f32; 4]) -> vk::ClearValue {
    vk::ClearValue {
        color: vk::ClearColorValue { float32: values },
    }
}

pub fn make_clear_color_i32(values: [i32; 4]) -> vk::ClearValue {
    vk::ClearValue {
        color: vk::ClearColorValue { int32: values },
    }
}

pub fn make_clear_color_ui32(values: [u32; 4]) -> vk::ClearValue {
    vk::ClearValue {
        color: vk::ClearColorValue { uint32: values },
    }
}

pub fn make_clear_depth(value: f32) -> vk::ClearValue {
    vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
            depth: value,
            stencil: 0,
        },
    }
}
