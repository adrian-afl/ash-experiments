use crate::attachment::VEAttachment;
use crate::device::VEDevice;
use ash::vk;
use std::sync::Arc;

pub struct VERenderPass {
    device: Arc<VEDevice>,
    pub handle: vk::RenderPass,
}

impl VERenderPass {
    pub fn new(
        device: Arc<VEDevice>,
        attachments: &[&VEAttachment],
        subpasses: &[vk::SubpassDescription],
    ) -> VERenderPass {
        let atta_descs: Vec<vk::AttachmentDescription> =
            attachments.iter().map(|a| a.description).collect();

        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&atta_descs)
            .subpasses(&subpasses);

        let handle = unsafe {
            device
                .device
                .create_render_pass(&create_info, None)
                .unwrap()
        };

        VERenderPass { device, handle }
    }
}
