use crate::compute::compute_stage::VEComputeStage;
use crate::core::command_buffer::VECommandBufferError;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::semaphore::{VESemaphore, VESemaphoreError};
use crate::graphics::render_stage::VERenderStage;
use crate::image::image::VEImage;
use crate::window::swapchain::{VESwapchain, VESwapchainError};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VESchedulerError {
    #[error("layer locking failed")]
    LayerLockingFailed,

    #[error("item locking failed")]
    ItemLockingFailed,

    #[error("swapchain locking failed")]
    SwapchainLockingFailed,

    #[error("semaphore error")]
    SemaphoreError(#[from] VESemaphoreError),

    #[error("swapchain error")]
    SwapchainError(#[from] VESwapchainError),

    #[error("command buffer error")]
    CommandBufferError(#[from] VECommandBufferError),
}

struct BlitStage {
    pub source: Arc<VEImage>,
}

enum Stage {
    Compute(Arc<VEComputeStage>),
    Render(Arc<VERenderStage>),
    Blit(BlitStage),
}

pub struct ScheduleItem {
    pub stage: Stage,
    pub semaphore: Arc<Mutex<VESemaphore>>,
}

struct ScheduleLayer {
    pub items: Vec<Arc<Mutex<ScheduleItem>>>,
}

pub struct VEScheduler {
    device: Arc<VEDevice>,
    swapchain: Arc<Mutex<VESwapchain>>,
    queue: Arc<VEMainDeviceQueue>,
    layers: Vec<Arc<Mutex<ScheduleLayer>>>,
}

impl VEScheduler {
    pub fn new(
        device: Arc<VEDevice>,
        swapchain: Arc<Mutex<VESwapchain>>,
        queue: Arc<VEMainDeviceQueue>,
        layers_count: u8,
    ) -> VEScheduler {
        let mut layers = vec![];
        for _ in 0..layers_count {
            layers.push(Arc::new(Mutex::from(ScheduleLayer { items: vec![] })));
        }
        VEScheduler {
            device,
            swapchain,
            queue,
            layers,
        }
    }

    pub fn create_render_item(
        &self,
        stage: Arc<VERenderStage>,
    ) -> Result<Arc<Mutex<ScheduleItem>>, VESchedulerError> {
        Ok(Arc::new(Mutex::from(ScheduleItem {
            stage: Stage::Render(stage),
            semaphore: Arc::new(Mutex::from(VESemaphore::new(self.device.clone())?)),
        })))
    }

    pub fn create_compute_item(
        &self,
        stage: Arc<VEComputeStage>,
    ) -> Result<Arc<Mutex<ScheduleItem>>, VESchedulerError> {
        Ok(Arc::new(Mutex::from(ScheduleItem {
            stage: Stage::Compute(stage),
            semaphore: Arc::new(Mutex::from(VESemaphore::new(self.device.clone())?)),
        })))
    }

    pub fn create_blit_item(
        &self,
        source: Arc<VEImage>,
    ) -> Result<Arc<Mutex<ScheduleItem>>, VESchedulerError> {
        Ok(Arc::new(Mutex::from(ScheduleItem {
            stage: Stage::Blit(BlitStage { source }),
            semaphore: Arc::new(Mutex::from(VESemaphore::new(self.device.clone())?)),
        })))
    }

    pub fn set_layer(
        &mut self,
        index: u8,
        items: Vec<Arc<Mutex<ScheduleItem>>>,
    ) -> Result<(), VESchedulerError> {
        self.layers[index as usize]
            .lock()
            .map_err(|_| VESchedulerError::LayerLockingFailed)?
            .items = items;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), VESchedulerError> {
        let mut swapchain = self
            .swapchain
            .lock()
            .map_err(|_| VESchedulerError::SwapchainLockingFailed)?;
        let blit_semaphore = &swapchain.blit_done_semaphore;

        let mut non_empty_layers: Vec<&Arc<Mutex<ScheduleLayer>>> = vec![];
        for i in 0..self.layers.len() {
            let layer = self.layers[i]
                .lock()
                .map_err(|_| VESchedulerError::LayerLockingFailed)?;
            if layer.items.len() > 0 {
                non_empty_layers.push(&self.layers[i]);
            }
        }

        let mut layer_semaphores: Vec<Vec<Arc<Mutex<VESemaphore>>>> = vec![];
        for _ in 0..non_empty_layers.len() {
            layer_semaphores.push(vec![]);
        }
        for i in 0..non_empty_layers.len() {
            let layer = non_empty_layers[i]
                .lock()
                .map_err(|_| VESchedulerError::LayerLockingFailed)?;
            let items = &layer.items;
            for g in 0..layer.items.len() {
                let item = items[g]
                    .lock()
                    .map_err(|_| VESchedulerError::ItemLockingFailed)?;
                let item_semaphore = item.semaphore.clone();
                match item.stage {
                    Stage::Compute(_) => layer_semaphores[i].push(item_semaphore),
                    Stage::Render(_) => layer_semaphores[i].push(item_semaphore),
                    Stage::Blit(_) => {
                        layer_semaphores[i].push(item_semaphore);
                        layer_semaphores[i].push(blit_semaphore.clone())
                    }
                };
            }
        }

        for i in 0..non_empty_layers.len() {
            // println!("LAYER {i}");
            let layer = self.layers[i]
                .lock()
                .map_err(|_| VESchedulerError::LayerLockingFailed)?;
            let previous_i = if i == 0 {
                non_empty_layers.len() - 1
            } else {
                i - 1
            };

            let items = &layer.items;
            for h in 0..items.len() {
                let item = items[h]
                    .lock()
                    .map_err(|_| VESchedulerError::ItemLockingFailed)?;
                // println!("ITEM {}", item.name);
                match &item.stage {
                    Stage::Compute(stage) => {
                        let mut previous_semaphores: Vec<Arc<Mutex<VESemaphore>>> = vec![];
                        for x in 0..layer_semaphores[previous_i].len() {
                            previous_semaphores.push(layer_semaphores[previous_i][x].clone());
                        }
                        stage.command_buffer.submit(
                            &self.queue,
                            previous_semaphores,
                            vec![item.semaphore.clone()],
                        )?
                    }
                    Stage::Render(stage) => {
                        let mut previous_semaphores: Vec<Arc<Mutex<VESemaphore>>> = vec![];
                        for x in 0..layer_semaphores[previous_i].len() {
                            previous_semaphores.push(layer_semaphores[previous_i][x].clone());
                        }
                        stage.command_buffer.submit(
                            &self.queue,
                            previous_semaphores,
                            vec![item.semaphore.clone()],
                        )?
                    }
                    Stage::Blit(stage) => {
                        let mut previous_semaphores: Vec<Arc<Mutex<VESemaphore>>> = vec![];
                        for x in 0..layer_semaphores[previous_i].len() {
                            previous_semaphores.push(layer_semaphores[previous_i][x].clone());
                        }
                        swapchain.blit(&stage.source, previous_semaphores)?
                    }
                }
            }
        }

        Ok(())
    }
}
