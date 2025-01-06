mod app;
mod buffer;
mod compute;
mod core;
mod graphics;
mod image;
mod memory;
mod window;

use std::fs::File;
use crate::app::MyApp;
use crate::core::toolkit::{App, VEToolkit};
use std::sync::{Arc, Mutex};
use tokio::main;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::FmtSubscriber;
use winit::dpi::PhysicalSize;
use winit::window::WindowAttributes;

#[main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_ansi(false)
        .with_writer(File::create("log.txt").unwrap())
        .with_span_events(FmtSpan::FULL)
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::trace!("Subscriber test message");

    let window_attributes = WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_title("planetdraw-rs");

    VEToolkit::start(
        Box::from(|toolkit: &VEToolkit| {
            let app = MyApp::new(&toolkit);
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
