use crate::core::device::VEDevice;
use crate::graphics::attachment::VEAttachment;
use crate::graphics::renderpass::VERenderPass;
use ash::vk;
use std::sync::Arc;

pub struct VEFrameBuffer {
    device: Arc<VEDevice>,
    pub handle: vk::Framebuffer,
}

impl VEFrameBuffer {
    pub fn new(
        device: Arc<VEDevice>,
        width: u32,
        height: u32,
        render_pass: &VERenderPass,
        attachments: &[&VEAttachment],
    ) -> VEFrameBuffer {
        let image_views: Vec<vk::ImageView> = attachments.iter().map(|a| a.image.view).collect();

        let create_info = vk::FramebufferCreateInfo::default()
            .attachments(&image_views)
            .render_pass(render_pass.handle)
            .width(width)
            .height(height)
            .layers(1);

        let handle = unsafe {
            device
                .device
                .create_framebuffer(&create_info, None)
                .unwrap()
        };

        VEFrameBuffer { device, handle }
    }
}
