use crate::image::image::VEImage;
use ash::vk;
use std::sync::Arc;

pub enum AttachmentBlending {
    Additive,
    Alpha,
}

pub struct VEAttachment {
    pub image_view: vk::ImageView,
    pub is_depth: bool,
    pub description: vk::AttachmentDescription,
    pub blending: Option<AttachmentBlending>,
    pub clear: Option<vk::ClearValue>,
}

impl VEAttachment {
    pub fn from_image(
        image: &VEImage,
        blending: Option<AttachmentBlending>,
        clear: Option<vk::ClearValue>,
        for_present: bool,
    ) -> VEAttachment {
        let description = vk::AttachmentDescription::default()
            .format(image.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(if clear.is_some() {
                vk::AttachmentLoadOp::CLEAR
            } else {
                vk::AttachmentLoadOp::LOAD
            })
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(image.current_layout)
            .final_layout(if for_present {
                vk::ImageLayout::PRESENT_SRC_KHR
            } else if image.is_depth() {
                vk::ImageLayout::GENERAL // TODO verify, its the final layout
            } else {
                vk::ImageLayout::GENERAL
            });

        VEAttachment {
            image_view: image.view.unwrap(),
            is_depth: image.is_depth(),
            description,
            blending,
            clear,
        }
    }
}
