use crate::attachment::{AttachmentBlending, VEAttachment};
use crate::command_buffer::VECommandBuffer;
use crate::command_pool::VECommandPool;
use crate::device::VEDevice;
use crate::main_device_queue::VEMainDeviceQueue;
use crate::memory::memory_chunk::VESingleAllocation;
use crate::memory::memory_manager::VEMemoryManager;
use ash::vk;
use ash::vk::CommandBufferUsageFlags;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VEImage {
    device: Arc<VEDevice>,
    queue: Arc<VEMainDeviceQueue>,
    command_pool: Arc<VECommandPool>,
    memory_manager: Arc<Mutex<VEMemoryManager>>,

    pub width: u32,
    pub height: u32,
    pub depth: u32,

    pub format: vk::Format,
    tiling: vk::ImageTiling,

    usage: vk::ImageUsageFlags,
    aspect: vk::ImageAspectFlags,

    pub current_layout: vk::ImageLayout,

    allocation: VESingleAllocation,
    handle: vk::Image,
    pub view: vk::ImageView,
    // mipmap view but thats for later
    sampler: Option<vk::Sampler>, // TODO maybe this can be different, separate for example
    sampler_address_mode: vk::SamplerAddressMode,

    min_filter: vk::Filter,
    mag_filter: vk::Filter,
}

impl VEImage {
    pub fn from_full(
        device: Arc<VEDevice>,
        queue: Arc<VEMainDeviceQueue>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,

        width: u32,
        height: u32,
        depth: u32,

        format: vk::Format,
        tiling: vk::ImageTiling,

        usage: vk::ImageUsageFlags,
        aspect: vk::ImageAspectFlags,

        memory_properties: vk::MemoryPropertyFlags,
    ) -> VEImage {
        let queue_family_indices = [device.queue_family_index];

        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(if depth == 1 {
                vk::ImageType::TYPE_2D
            } else {
                vk::ImageType::TYPE_3D
            })
            .extent(
                vk::Extent3D::default()
                    .width(width)
                    .height(height)
                    .depth(depth),
            )
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family_indices)
            .initial_layout(vk::ImageLayout::PREINITIALIZED);

        let image_handle = unsafe {
            device
                .device
                .create_image(&image_create_info, None)
                .unwrap()
        };

        let mem_reqs = unsafe { device.device.get_image_memory_requirements(image_handle) };
        let mem_index = device.find_memory_type(mem_reqs.memory_type_bits, memory_properties);

        let allocation = {
            memory_manager
                .lock()
                .unwrap()
                .bind_image_memory(mem_index, image_handle, mem_reqs.size)
        };

        let image_view_create_info = vk::ImageViewCreateInfo::default()
            .image(image_handle)
            .view_type(if depth == 1 {
                vk::ImageViewType::TYPE_2D
            } else {
                vk::ImageViewType::TYPE_3D
            })
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect)
                    .base_mip_level(0)
                    .level_count(1) // TODO MIPMAPPING
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY, // TODO identity?
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            });
        let image_view_handle = unsafe {
            device
                .device
                .create_image_view(&image_view_create_info, None)
                .unwrap()
        };

        let mut image = VEImage {
            device,
            queue,
            command_pool,
            memory_manager,

            allocation,

            width,
            height,
            depth,

            format,
            tiling,

            usage,
            aspect,

            handle: image_handle,
            view: image_view_handle,
            current_layout: vk::ImageLayout::PREINITIALIZED,

            sampler: None,
            sampler_address_mode: vk::SamplerAddressMode::REPEAT,

            min_filter: vk::Filter::LINEAR,
            mag_filter: vk::Filter::LINEAR,
        };
        image.transition_layout(vk::ImageLayout::GENERAL);

        image
    }

    pub fn from_swapchain_present_image(
        device: Arc<VEDevice>,
        queue: Arc<VEMainDeviceQueue>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,

        width: u32,
        height: u32,

        format: vk::Format,
        image_handle: vk::Image,
        image_view_handle: vk::ImageView,
    ) -> VEImage {
        let mut image = VEImage {
            device,
            queue,
            command_pool,
            memory_manager,

            allocation: VESingleAllocation {
                alloc_identifier: u64::MAX,
                chunk_identifier: u64::MAX,
                size: 0,
                offset: 0,
            },

            width,
            height,
            depth: 1,

            format,
            tiling: vk::ImageTiling::OPTIMAL,

            usage: vk::ImageUsageFlags::empty(),
            aspect: vk::ImageAspectFlags::empty(),

            handle: image_handle,
            view: image_view_handle,
            current_layout: vk::ImageLayout::UNDEFINED,

            sampler: None,
            sampler_address_mode: vk::SamplerAddressMode::REPEAT,

            min_filter: vk::Filter::LINEAR,
            mag_filter: vk::Filter::LINEAR,
        };
        // image.transition_layout(vk::ImageLayout::GENERAL);

        image
    }

    pub fn create_attachment(
        &self,
        blending: Option<AttachmentBlending>,
        clear: Option<vk::ClearValue>,
        for_present: bool,
    ) -> VEAttachment {
        VEAttachment::new(Arc::new(self.clone()), blending, clear, for_present)
    }

    pub fn is_depth(&self) -> bool {
        self.format == vk::Format::D16_UNORM || self.format == vk::Format::D32_SFLOAT
    }

    fn transition_layout(&mut self, new_layout: vk::ImageLayout) {
        let mut src_access = vk::AccessFlags::empty();
        let mut dst_access = vk::AccessFlags::empty();
        let mut source_stage = vk::PipelineStageFlags::empty();
        let mut destination_stage = vk::PipelineStageFlags::empty();

        if (self.current_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        {
            dst_access = vk::AccessFlags::TRANSFER_WRITE;

            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if (self.current_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        {
            src_access = vk::AccessFlags::TRANSFER_WRITE;
            dst_access = vk::AccessFlags::SHADER_READ;

            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else {
            source_stage = vk::PipelineStageFlags::ALL_COMMANDS;
            destination_stage = vk::PipelineStageFlags::ALL_COMMANDS;
            match (self.current_layout) {
                vk::ImageLayout::PREINITIALIZED => src_access = vk::AccessFlags::HOST_WRITE,

                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => {
                    src_access = vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                }

                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => {
                    src_access = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                }

                vk::ImageLayout::TRANSFER_SRC_OPTIMAL => {
                    src_access = vk::AccessFlags::TRANSFER_READ
                }

                vk::ImageLayout::TRANSFER_DST_OPTIMAL => {
                    src_access = vk::AccessFlags::TRANSFER_WRITE
                }

                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => {
                    src_access = vk::AccessFlags::SHADER_READ
                }

                _ => (),
            }
            match (new_layout) {
                vk::ImageLayout::TRANSFER_DST_OPTIMAL => {
                    dst_access = vk::AccessFlags::TRANSFER_WRITE
                }

                vk::ImageLayout::TRANSFER_SRC_OPTIMAL => {
                    dst_access = vk::AccessFlags::TRANSFER_READ
                }

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
                    if (self.current_layout == vk::ImageLayout::TRANSFER_SRC_OPTIMAL) {
                        src_access = vk::AccessFlags::TRANSFER_READ;
                    }
                    dst_access = vk::AccessFlags::SHADER_READ;
                }
                _ => (),
            }
        }

        let command_buffer = VECommandBuffer::new(self.device.clone(), self.command_pool.clone());
        //command_buffer.begin(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        command_buffer.begin(CommandBufferUsageFlags::empty());

        self.image_memory_barrier(
            &command_buffer,
            new_layout,
            src_access,
            dst_access,
            source_stage,
            destination_stage,
        );

        command_buffer.end();

        self.current_layout = new_layout;
    }

    fn image_memory_barrier(
        &self,
        command_buffer: &VECommandBuffer,
        new_layout: vk::ImageLayout,
        src_access: vk::AccessFlags,
        dst_access: vk::AccessFlags,
        source_stage: vk::PipelineStageFlags,
        destination_stage: vk::PipelineStageFlags,
    ) {
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(self.current_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(self.aspect)
                    .base_mip_level(0)
                    .level_count(1) // TODO MIPMAPPING
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .src_access_mask(src_access)
            .dst_access_mask(dst_access);

        unsafe {
            self.device.device.cmd_pipeline_barrier(
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
}
