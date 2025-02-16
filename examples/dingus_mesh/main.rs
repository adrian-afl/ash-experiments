mod dingus_app;

use dingus_app::DingusApp;

use std::sync::{Arc, Mutex};
use vengine_rs::core::toolkit::{App, VEToolkit};
use winit::dpi::PhysicalSize;
use winit::window::{Window, WindowAttributes};

#[allow(clippy::unwrap_used)]
fn main() {
    let window_attributes = WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_title("dingus_mesh");

    VEToolkit::start(
        Box::from(|toolkit: Arc<VEToolkit>, window: Arc<Mutex<Window>>| {
            let app = DingusApp::new(toolkit, window);
            Arc::new(Mutex::from(app)) as Arc<Mutex<dyn App>>
        }),
        window_attributes,
    )
    .unwrap()
}
