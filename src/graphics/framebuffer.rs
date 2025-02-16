use crate::core::device::VEDevice;
use crate::graphics::attachment::VEAttachment;
use crate::graphics::renderpass::VERenderPass;
use ash::vk;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEFrameBufferError {
    #[error("creation failed")]
    CreationFailed(#[from] vk::Result),
}

pub struct VEFrameBuffer {
    pub handle: vk::Framebuffer,
}

impl VEFrameBuffer {
    pub fn new(
        device: Arc<VEDevice>,
        width: u32,
        height: u32,
        render_pass: &VERenderPass,
        attachments: &[&VEAttachment],
    ) -> Result<VEFrameBuffer, VEFrameBufferError> {
        let image_views: Vec<vk::ImageView> = attachments.iter().map(|a| a.image_view).collect();

        let create_info = vk::FramebufferCreateInfo::default()
            .attachments(&image_views)
            .render_pass(render_pass.handle)
            .width(width)
            .height(height)
            .layers(1);

        let handle = unsafe { device.device.create_framebuffer(&create_info, None)? };

        Ok(VEFrameBuffer { handle })
    }
}
