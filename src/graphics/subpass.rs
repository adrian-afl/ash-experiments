use ash::vk;

pub fn create_subpass<'a>(
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

pub fn create_subpass_attachment_reference(index: i32, depth: bool) -> vk::AttachmentReference {
    vk::AttachmentReference::default()
        .attachment(index as u32)
        .layout(if depth {
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        } else {
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        })
}
