use lazy_static::lazy_static;

#[cfg(target_arch = "wasm32")]
mod platform_time {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    pub fn current_time() -> f64 {
        web_sys::window()
            .expect("no global `window` exists")
            .performance()
            .expect("performance should be available")
            .now() / 1000.0 // Convert milliseconds to seconds
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod platform_time {
    use std::time::Instant;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref START: Instant = Instant::now();
    }

    pub fn current_time() -> f64 {
        START.elapsed().as_secs_f64()
    }
}

lazy_static! {
    static ref START_TIME: f64 = platform_time::current_time();
}

pub fn now() -> f32 {
    (platform_time::current_time() - *START_TIME) as f32
}

fn main() {
    // Example usage
    let start = now();
    // Simulate some workload
    for _ in 0..1_000_000 {
        let _ = 1 + 1;
    }
    let end = now();
    println!("Elapsed time: {} seconds", end - start);
}
