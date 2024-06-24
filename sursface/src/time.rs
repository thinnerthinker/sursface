use web_sys::window;

pub fn now() -> f32 {
    #[cfg(target_arch = "wasm32")]
    {
        let window = window().expect("no global `window` exists");
        let performance = window
            .performance()
            .expect("should have a `performance` object on `window`");
        performance.now() as f32
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let now = Instant::now();
        let duration = now.elapsed();
        duration.as_secs_f32()
    }
}
