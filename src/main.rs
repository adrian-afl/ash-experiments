mod app;
mod buffer;
mod compute;
mod core;
mod graphics;
mod image;
mod memory;
mod window;

use crate::app::MyApp;
use crate::core::toolkit::{App, VEToolkit};
use std::sync::{Arc, Mutex};
use tokio::main;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use winit::dpi::PhysicalSize;
use winit::window::WindowAttributes;

#[main]
async fn main() {
    env_logger::init();

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let window_attributes = WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_title("planetdraw-rs");

    VEToolkit::start(
        Box::from(|toolkit: Arc<VEToolkit>| {
            let app = MyApp::new(toolkit);
            Arc::new(Mutex::from(app)) as Arc<Mutex<dyn App>>
        }),
        window_attributes,
    )

    // let app = MyApp {
    //     vertex_buffer: None,
    //     output_command_buffer: None,
    //     output_stage: None,
    //     queue: None,
    //     descriptor_set: None,
    // };
    //

    //
    // let mut window = VEWindow::new(Arc::new(Mutex::from(app)), window_attributes);
}
