use crate::compute::compute_stage::VEComputeStage;
use crate::core::device::VEDevice;
use crate::core::main_device_queue::VEMainDeviceQueue;
use crate::core::semaphore::VESemaphore;
use crate::graphics::render_stage::VERenderStage;
use crate::image::image::VEImage;
use crate::window::swapchain::VESwapchain;
use std::sync::{Arc, Mutex};

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
        for i in 0..layers_count {
            layers.push(Arc::new(Mutex::from(ScheduleLayer { items: vec![] })));
        }
        VEScheduler {
            device,
            swapchain,
            queue,
            layers,
        }
    }

    pub fn make_render_item(&self, stage: Arc<VERenderStage>) -> Arc<Mutex<ScheduleItem>> {
        Arc::new(Mutex::from(ScheduleItem {
            stage: Stage::Render(stage),
            semaphore: Arc::new(Mutex::from(VESemaphore::new(self.device.clone()))),
        }))
    }

    pub fn make_compute_item(&self, stage: Arc<VEComputeStage>) -> Arc<Mutex<ScheduleItem>> {
        Arc::new(Mutex::from(ScheduleItem {
            stage: Stage::Compute(stage),
            semaphore: Arc::new(Mutex::from(VESemaphore::new(self.device.clone()))),
        }))
    }

    pub fn make_blit_item(&self, source: Arc<VEImage>) -> Arc<Mutex<ScheduleItem>> {
        Arc::new(Mutex::from(ScheduleItem {
            stage: Stage::Blit(BlitStage { source }),
            semaphore: Arc::new(Mutex::from(VESemaphore::new(self.device.clone()))),
        }))
    }

    pub fn set_layer(&mut self, index: u8, items: Vec<Arc<Mutex<ScheduleItem>>>) {
        self.layers[index as usize].lock().unwrap().items = items;
    }

    pub fn run(&mut self) {
        let mut swapchain = self.swapchain.lock().unwrap();
        let blit_semaphore = &swapchain.blit_done_semaphore;

        let non_empty_layers: Vec<&Arc<Mutex<ScheduleLayer>>> = self
            .layers
            .iter()
            .filter(|l| l.lock().unwrap().items.len() > 0)
            .collect();

        let mut layer_semaphores: Vec<Vec<Arc<Mutex<VESemaphore>>>> = vec![];
        for i in 0..non_empty_layers.len() {
            layer_semaphores.push(vec![]);
        }
        for i in 0..non_empty_layers.len() {
            let layer = non_empty_layers[i].lock().unwrap();
            let items = &layer.items;
            for g in 0..layer.items.len() {
                let item = items[g].lock().unwrap();
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

        non_empty_layers.iter().enumerate().for_each(|(i, layer)| {
            // println!("LAYER {i}");
            let mut layer = layer.lock().unwrap();
            let previous_i = if i == 0 {
                non_empty_layers.len() - 1
            } else {
                i - 1
            };

            // let mut previous_semaphores: Vec<MutexGuard<VESemaphore>> = layer_semaphores[previous_i]
            //     .iter()
            //     .map(|x| {
            //         let locked = x.lock().unwrap();
            //         locked
            //     })
            //     .collect();
            // let previous_semaphores = previous_semaphores.as_mut_slice();
            // // let current_semaphores = &layer_semaphores[i];

            let items = &layer.items;
            for h in 0..items.len() {
                let item = items[h].lock().unwrap();
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
                        )
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
                        )
                    }
                    Stage::Blit(stage) => {
                        let mut previous_semaphores: Vec<Arc<Mutex<VESemaphore>>> = vec![];
                        for x in 0..layer_semaphores[previous_i].len() {
                            previous_semaphores.push(layer_semaphores[previous_i][x].clone());
                        }
                        swapchain.blit(&stage.source, previous_semaphores)
                    }
                }
            }
        });
    }
}
