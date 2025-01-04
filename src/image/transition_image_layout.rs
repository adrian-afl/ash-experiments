use crate::core::command_buffer::VECommandBuffer;
use crate::core::command_pool::VECommandPool;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use ash::vk;
use ash::vk::CommandBufferUsageFlags;
use std::sync::Arc;

pub fn transition_image_layout(
    device: Arc<VEDevice>,
    command_pool: Arc<VECommandPool>,
    queue: Arc<VEMainDeviceQueue>,
    image_handle: vk::Image,
    aspect: vk::ImageAspectFlags,
    current_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    let mut src_access = vk::AccessFlags::empty();
    let mut dst_access = vk::AccessFlags::empty();
    let mut source_stage = vk::PipelineStageFlags::empty();
    let mut destination_stage = vk::PipelineStageFlags::empty();

    if (current_layout == vk::ImageLayout::UNDEFINED
        && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL)
    {
        dst_access = vk::AccessFlags::TRANSFER_WRITE;

        source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        destination_stage = vk::PipelineStageFlags::TRANSFER;
    } else if (current_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
    {
        src_access = vk::AccessFlags::TRANSFER_WRITE;
        dst_access = vk::AccessFlags::SHADER_READ;

        source_stage = vk::PipelineStageFlags::TRANSFER;
        destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    } else {
        source_stage = vk::PipelineStageFlags::ALL_COMMANDS;
        destination_stage = vk::PipelineStageFlags::ALL_COMMANDS;
        match (current_layout) {
            vk::ImageLayout::PREINITIALIZED => src_access = vk::AccessFlags::HOST_WRITE,

            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => {
                src_access = vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            }

            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => {
                src_access = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            }

            vk::ImageLayout::TRANSFER_SRC_OPTIMAL => src_access = vk::AccessFlags::TRANSFER_READ,

            vk::ImageLayout::TRANSFER_DST_OPTIMAL => src_access = vk::AccessFlags::TRANSFER_WRITE,

            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => src_access = vk::AccessFlags::SHADER_READ,

            _ => (),
        }
        match (new_layout) {
            vk::ImageLayout::TRANSFER_DST_OPTIMAL => dst_access = vk::AccessFlags::TRANSFER_WRITE,

            vk::ImageLayout::TRANSFER_SRC_OPTIMAL => dst_access = vk::AccessFlags::TRANSFER_READ,

            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => {
                dst_access = vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            }

            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => {
                dst_access = dst_access | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            }

            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => {
                if src_access == vk::AccessFlags::empty() {
                    src_access = vk::AccessFlags::HOST_WRITE | vk::AccessFlags::TRANSFER_WRITE;
                }
                if (current_layout == vk::ImageLayout::TRANSFER_SRC_OPTIMAL) {
                    src_access = vk::AccessFlags::TRANSFER_READ;
                }
                dst_access = vk::AccessFlags::SHADER_READ;
            }
            _ => (),
        }
    }

    let command_buffer = VECommandBuffer::new(device.clone(), command_pool.clone());
    //command_buffer.begin(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    command_buffer.begin(CommandBufferUsageFlags::empty());

    image_memory_barrier(
        device,
        &command_buffer,
        image_handle,
        aspect,
        current_layout,
        new_layout,
        src_access,
        dst_access,
        source_stage,
        destination_stage,
    );

    command_buffer.end();

    command_buffer.submit(&queue, &[], &[]);
    queue.wait_idle();
}

fn image_memory_barrier(
    device: Arc<VEDevice>,
    command_buffer: &VECommandBuffer,
    image_handle: vk::Image,
    aspect: vk::ImageAspectFlags,
    current_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_access: vk::AccessFlags,
    dst_access: vk::AccessFlags,
    source_stage: vk::PipelineStageFlags,
    destination_stage: vk::PipelineStageFlags,
) {
    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(current_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image_handle)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(aspect)
                .base_mip_level(0)
                .level_count(1) // TODO MIPMAPPING
                .base_array_layer(0)
                .layer_count(1),
        )
        .src_access_mask(src_access)
        .dst_access_mask(dst_access);

    unsafe {
        device.device.cmd_pipeline_barrier(
            command_buffer.handle,
            source_stage,
            destination_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    }
}
