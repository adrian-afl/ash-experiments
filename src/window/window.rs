use ash::Entry;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

pub struct VEWindow {
    pub event_loop: EventLoop<()>,
    pub window: Window,
    pub entry: Entry
}

impl VEWindow {
    pub fn new() -> VEWindow {
        let event_loop = EventLoop::new().unwrap();

        let attributes = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(640, 480))
            .with_title("planetdraw-rs");

        let window = event_loop.create_window(attributes).unwrap();
        let entry = Entry::linked();
        VEWindow {
            event_loop,
            window,
            entry
        }
    }
}