use crate::core::toolkit::VEToolkit;
use ash::Entry;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::error::EventLoopError;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Error, Debug)]
pub enum VEWindowError {
    #[error("event loop creation failed")]
    EventLoopCreationFailed(#[source] EventLoopError),

    #[error("event loop cannot start")]
    EventLoopCannotStart(#[source] EventLoopError),
}

pub trait AppCallback {
    fn on_window_ready(&mut self, toolkit: Arc<VEToolkit>);
    fn on_window_draw(&self);
    fn on_window_resize(&self, new_size: PhysicalSize<u32>);
}

pub struct VEWindow {
    pub window: Option<Window>,
    pub entry: Entry,

    initial_window_attributes: WindowAttributes,

    pub app: Arc<Mutex<dyn AppCallback>>,
}

impl Debug for VEWindow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("VEWindow")
    }
}

impl ApplicationHandler for VEWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(self.initial_window_attributes.clone());

        match window {
            Ok(window) => {
                self.window = Some(window);
                self.on_window_ready();

                let window = self.window.as_ref();
                match window {
                    None => println!(
                        "Completely unexpected problem that window is None right after assignment"
                    ),
                    Some(window) => window.request_redraw(),
                }
            }
            Err(error) => println!("Window cannot be created! Reason: {:?}", error),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let window = self.window.as_ref();
        match window {
            None => println!("Completely unexpected problem that window is None"),
            Some(window) => match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    window.pre_present_notify();
                    let locked_app = self.app.lock();
                    match locked_app {
                        Ok(app) => {
                            app.on_window_draw();
                            window.request_redraw();
                        }
                        Err(error) => println!("Could not lock app mutex! Reason: {:?}", error),
                    };
                }
                WindowEvent::Resized(new_size) => {
                    let locked_app = self.app.lock();
                    match locked_app {
                        Ok(app) => {
                            app.on_window_resize(new_size);
                        }
                        Err(error) => println!("Could not lock app mutex! Reason: {:?}", error),
                    };
                }
                _ => (),
            },
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // println!("{:?}, {:?}", device_id, event);
    }
}

impl VEWindow {
    pub fn new(
        app: Arc<Mutex<dyn AppCallback>>,
        initial_window_attributes: WindowAttributes,
    ) -> Result<VEWindow, VEWindowError> {
        let event_loop = EventLoop::new().map_err(VEWindowError::EventLoopCreationFailed)?;

        let mut window = VEWindow {
            window: None,
            entry: Entry::linked(),
            initial_window_attributes,
            app,
        };

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop
            .run_app(&mut window)
            .map_err(VEWindowError::EventLoopCannotStart)?;

        Ok(window)
    }

    fn on_window_ready(&self) {
        let toolkit = VEToolkit::new(self);
        match toolkit {
            Ok(toolkit) => {
                let toolkit = Arc::new(toolkit);
                let locked_app = self.app.lock();
                match locked_app {
                    Ok(mut app) => {
                        app.on_window_ready(toolkit);
                    }
                    Err(error) => println!("Could not lock app mutex! Reason: {:?}", error),
                };
            }
            Err(error) => println!("Toolkit cannot be created! Reason: {:?}", error),
        }
    }
}
