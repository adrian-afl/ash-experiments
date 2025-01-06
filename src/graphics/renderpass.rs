use crate::core::device::VEDevice;
use crate::graphics::attachment::VEAttachment;
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
    ) -> VERenderPass {
        let color_attas: Vec<&&VEAttachment> =
            attachments.iter().filter(|x| !x.is_depth).collect();
        let depth_atta = attachments.iter().filter(|x| x.is_depth).last();

        let color_references: Vec<vk::AttachmentReference> = (0..color_attas.len())
            .map(|i| Self::create_subpass_attachment_reference(i as i32, false))
            .collect();

        let depth_reference_maybe =
            Self::create_subpass_attachment_reference(color_attas.len() as i32, true);
        let depth_reference = match depth_atta {
            None => None,
            Some(_) => Some(&depth_reference_maybe), // depth last
        };

        let subpass = Self::create_subpass(&color_references, depth_reference);
        let subpasses = [subpass];

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

    fn create_subpass<'a>(
        color_references: &'a [vk::AttachmentReference],
        depth_reference: Option<&'a vk::AttachmentReference>,
    ) -> vk::SubpassDescription<'a> {
        let mut description = vk::SubpassDescription::default().color_attachments(&color_references);
        match depth_reference {
            None => (),
            Some(reference) => description = description.depth_stencil_attachment(reference),
        }
        description
    }

    fn create_subpass_attachment_reference(index: i32, depth: bool) -> vk::AttachmentReference {
        vk::AttachmentReference::default()
            .attachment(index as u32)
            .layout(if depth {
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            } else {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            })
    }
}
