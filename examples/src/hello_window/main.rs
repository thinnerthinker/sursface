#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    sursface::start::start_desktop(PhysicalSize::new(1280, 720));
}

#[cfg(target_arch = "wasm32")]
fn main() {}