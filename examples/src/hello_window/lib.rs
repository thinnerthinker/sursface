use sursface::start;

#[cfg(target_arch = "wasm32")]
use sursface::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    start::create_window_browser(canvas);
}