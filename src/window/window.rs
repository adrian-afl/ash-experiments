use crate::core::toolkit::VEToolkit;
use ash::{Entry, LoadingError};
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

    #[error("vulkan loading error")]
    LoadingError(#[source] LoadingError),
}

pub trait AppCallback {
    fn on_window_ready(&mut self, toolkit: Arc<VEToolkit>, window: Arc<Mutex<Window>>);
    fn on_window_draw(&self);
    fn on_window_resize(&mut self, new_size: PhysicalSize<u32>);

    fn on_window_event(&mut self, event: WindowEvent);
    fn on_device_event(&mut self, device_id: DeviceId, event: DeviceEvent);
}

pub struct VEWindow {
    pub window: Option<Arc<Mutex<Window>>>,
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
                self.window = Some(Arc::new(Mutex::from(window)));
                self.on_window_ready();

                let window = self.window.as_ref();
                match window {
                    None => println!(
                        "Completely unexpected problem that window is None right after assignment"
                    ),
                    Some(window) => {
                        let window = window.lock();
                        match window {
                            Ok(window) => window.request_redraw(),
                            Err(error) => {
                                println!("Could not lock window mutex! Reason: {:?}", error)
                            }
                        }
                    }
                }
            }
            Err(error) => println!("Window cannot be created! Reason: {:?}", error),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match &self.window {
            None => println!("Completely unexpected problem that window is None"),
            Some(window) => match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    {
                        let window = window.lock();
                        match window {
                            Err(error) => {
                                println!("Could not lock window mutex! Reason: {:?}", error)
                            }
                            Ok(window) => window.pre_present_notify(),
                        }
                    }
                    let locked_app = self.app.lock();
                    match locked_app {
                        Ok(app) => {
                            app.on_window_draw();
                            {
                                let window = window.lock();
                                match window {
                                    Err(error) => {
                                        println!("Could not lock window mutex! Reason: {:?}", error)
                                    }
                                    Ok(window) => window.request_redraw(),
                                }
                            }
                        }
                        Err(error) => {
                            println!("Could not lock app mutex! Reason: {:?}", error)
                        }
                    };
                }
                WindowEvent::Resized(new_size) => {
                    let locked_app = self.app.lock();
                    match locked_app {
                        Ok(mut app) => {
                            app.on_window_resize(new_size);
                        }
                        Err(error) => {
                            println!("Could not lock app mutex! Reason: {:?}", error)
                        }
                    };
                }
                _ => {
                    let locked_app = self.app.lock();
                    match locked_app {
                        Ok(mut app) => {
                            app.on_window_event(event);
                        }
                        Err(error) => {
                            println!("Could not lock app mutex! Reason: {:?}", error)
                        }
                    };
                }
            },
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let locked_app = self.app.lock();
        match locked_app {
            Ok(mut app) => {
                app.on_device_event(device_id, event);
            }
            Err(error) => println!("Could not lock app mutex! Reason: {:?}", error),
        };
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
            entry: unsafe { Entry::load().map_err(VEWindowError::LoadingError)? },
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
                match &self.window {
                    None => println!("Completely unexpected problem that window is None"),
                    Some(window) => match locked_app {
                        Ok(mut app) => {
                            app.on_window_ready(toolkit, window.clone());
                        }
                        Err(error) => println!("Could not lock app mutex! Reason: {:?}", error),
                    },
                }
            }
            Err(error) => println!("Toolkit cannot be created! Reason: {:?}", error),
        }
    }
}
