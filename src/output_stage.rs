use crate::attachment::VEAttachment;
use crate::command_buffer::VECommandBuffer;
use crate::command_pool::VECommandPool;
use crate::descriptor_set::VEDescriptorSet;
use crate::descriptor_set_layout::VEDescriptorSetLayout;
use crate::device::VEDevice;
use crate::image::VEImage;
use crate::main_device_queue::VEMainDeviceQueue;
use crate::memory::memory_manager::VEMemoryManager;
use crate::render_stage::{CullMode, VERenderStage};
use crate::semaphore::VESemaphore;
use crate::shader_module::VEShaderModule;
use crate::swapchain::VESwapchain;
use crate::vertex_attributes::VertexAttribFormat;
use ash::vk;
use std::sync::{Arc, Mutex};

static BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;

pub struct VEOutputStage {
    device: Arc<VEDevice>,
    swapchain: Arc<Mutex<VESwapchain>>,
    render_stages: Vec<VERenderStage>,
    current_image: u32,
    pub image_ready_semaphore: VESemaphore,
    pub ready_for_present_semaphore: VESemaphore,
}

impl VEOutputStage {
    pub fn new(
        device: Arc<VEDevice>,
        queue: Arc<VEMainDeviceQueue>,
        command_pool: Arc<VECommandPool>,
        memory_manager: Arc<Mutex<VEMemoryManager>>,
        swapchain: Arc<Mutex<VESwapchain>>,
        clear_color: Option<vk::ClearValue>,
        depth_attachment: Option<&VEAttachment>,
        set_layouts: &[&VEDescriptorSetLayout],
        vertex_shader: &VEShaderModule,
        fragment_shader: &VEShaderModule,
        vertex_attributes: &[VertexAttribFormat],
        primitive_topology: vk::PrimitiveTopology,
        cull_mode: CullMode,
    ) -> VEOutputStage {
        let swapchain_locked = swapchain.lock().unwrap();
        let mut render_stages = vec![];
        for i in 0..swapchain_locked.present_images.len() {
            let mut attachments = vec![];
            match depth_attachment {
                None => (),
                Some(depth_attachment) => attachments.push(depth_attachment),
            }
            let image = VEImage::from_swapchain_present_image(
                device.clone(),
                queue.clone(),
                command_pool.clone(),
                memory_manager.clone(),
                swapchain_locked.width,
                swapchain_locked.height,
                swapchain_locked.present_image_format,
                swapchain_locked.present_images[i],
                swapchain_locked.present_image_views[i],
            );
            let color_atta = image.create_attachment(None, clear_color, true);
            attachments.push(&color_atta);
            render_stages.push(VERenderStage::new(
                device.clone(),
                swapchain_locked.width,
                swapchain_locked.height,
                &attachments,
                &set_layouts,
                &vertex_shader,
                &fragment_shader,
                &vertex_attributes,
                primitive_topology,
                cull_mode.clone(),
            ))
        }

        VEOutputStage {
            device: device.clone(),
            swapchain: swapchain.clone(),
            render_stages,
            current_image: 0,
            image_ready_semaphore: VESemaphore::new(device.clone()),
            ready_for_present_semaphore: VESemaphore::new(device.clone()),
        }
    }

    pub fn next_image(&mut self) {
        self.current_image = self
            .swapchain
            .lock()
            .unwrap()
            .acquire_next_image(&self.image_ready_semaphore);
        println!("Aquired image {}", self.current_image);
    }

    pub fn present(&mut self) {
        let waitfor = [&self.ready_for_present_semaphore];
        println!("Presenting image {}", self.current_image);
        self.swapchain
            .lock()
            .unwrap()
            .present(&waitfor, self.current_image);
    }

    pub fn set_descriptor_set(
        &mut self,
        command_buffer: &VECommandBuffer,
        index: usize,
        set: Arc<VEDescriptorSet>,
    ) {
        for stage in &mut self.render_stages {
            stage.set_descriptor_set(command_buffer, index, set.clone());
        }
    }

    pub fn begin_recording(&self, command_buffer: &VECommandBuffer) {
        self.render_stages[self.current_image as usize].begin_recording(command_buffer);
    }

    pub fn end_recording(&self, command_buffer: &VECommandBuffer) {
        self.render_stages[self.current_image as usize].end_recording(command_buffer);
    }
}
