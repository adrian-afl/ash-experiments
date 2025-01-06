use crate::core::toolkit::VEToolkit;
use crate::window::swapchain::VESwapchain;
use ash::Entry;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

pub trait AppCallback {
    fn on_window_ready(&mut self, toolkit: VEToolkit);
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
        let window = event_loop
            .create_window(self.initial_window_attributes.clone())
            .unwrap();

        self.window = Some(window);
        self.on_window_ready();

        let window = self.window.as_ref().unwrap();
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let window = self.window.as_ref().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                window.pre_present_notify();
                self.app.lock().unwrap().on_window_draw();
                window.request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.app.lock().unwrap().on_window_resize(new_size);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        println!("{:?}, {:?}", device_id, event);
    }
}

impl VEWindow {
    pub fn new(
        app: Arc<Mutex<dyn AppCallback>>,
        initial_window_attributes: WindowAttributes,
    ) -> VEWindow {
        let event_loop = EventLoop::new().unwrap();

        let mut window = VEWindow {
            window: None,
            entry: Entry::linked(),
            initial_window_attributes,
            app,
        };

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut window).expect("Can't run`");

        window
    }

    fn on_window_ready(&self) {
        let toolkit = VEToolkit::new(self);
        self.app.lock().unwrap().on_window_ready(toolkit);
    }
}
