use std::sync::{Arc, Mutex};
use ash::Entry;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

pub struct VEWindow {
    // pub event_loop: EventLoop<()>,
    pub window: Option<Window>,
    pub entry: Entry,

    pub app: Arc<Mutex<dyn App>>,
}

pub trait App {
    fn prepare(&mut self, window: &VEWindow);
    fn draw(&mut self);
}

impl ApplicationHandler for VEWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(640, 480))
            .with_title("planetdraw-rs");
        let window = event_loop.create_window(attributes).unwrap();

        self.window = Some(window);
        self.prepare_app();

        let window = self.window.as_ref().unwrap();
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let window = self.window.as_ref().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                println!("Draw");
                window.pre_present_notify();
                self.app.lock().unwrap().draw();
                window.request_redraw();
            }
            _ => (),
        }
    }
}


impl VEWindow {
    pub fn new(app: Arc<Mutex<dyn App>>) -> VEWindow {
        let event_loop = EventLoop::new().unwrap();

        let mut window = VEWindow {
            // event_loop,
            window: None,
            entry: Entry::linked(),

            app
        };

        // let start = move |window: &mut VEWindow| {
            event_loop.set_control_flow(ControlFlow::Poll);

            event_loop.run_app(&mut window).expect("Can't run`");
        // };

        let attributes = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(640, 480))
            .with_title("planetdraw-rs");

        window
    }

    fn prepare_app(&self){
        self.app.lock().unwrap().prepare(self);
    }
}