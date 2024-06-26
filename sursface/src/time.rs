use lazy_static::lazy_static;

#[cfg(target_arch = "wasm32")]
mod platform_time {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = performance)]
        fn now() -> f64;
    }

    pub fn now() -> f64 {
        now() / 1000.0 // Convert milliseconds to seconds
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod platform_time {
    use lazy_static::lazy_static;
    use std::time::Instant;

    lazy_static! {
        static ref TIME_AT_START: Instant = Instant::now();
    }

    pub fn now() -> f64 {
        TIME_AT_START.elapsed().as_secs_f64()
    }
}

lazy_static! {
    static ref TIME_AT_START: f64 = platform_time::now();
}

pub fn now() -> f32 {
    (platform_time::now() - *TIME_AT_START) as f32
}
