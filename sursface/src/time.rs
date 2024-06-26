use lazy_static::lazy_static;

#[cfg(target_arch = "wasm32")]
mod platform_time {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = performance)]
        fn now() -> f64;
    }

    pub fn current_time() -> f64 {
        now() / 1000.0 // Convert milliseconds to seconds
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod platform_time {
    use std::time::Instant;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref TIME_AT_START: Instant = Instant::now();
    }

    pub fn current_time() -> f64 {
        TIME_AT_START.elapsed().as_secs_f64()
    }
}

lazy_static! {
    static ref TIME_AT_START: f64 = platform_time::current_time();
}

pub fn now() -> f32 {
    (platform_time::current_time() - *TIME_AT_START) as f32
}

fn main() {
    // Example usage
    println!("Current time: {} seconds since start", now());
}
