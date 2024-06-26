#![allow(dead_code, unused_imports)]

use wasm_bindgen::prelude::*;
use wasm_timer::{Instant};
use std::convert::TryInto;
use std::ops::{Add, Sub, AddAssign, SubAssign};
use lazy_static::lazy_static;

#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

#[cfg(target_arch = "wasm32")]
lazy_static! {
    static ref START_TIME: f64 = performance_now();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn performance_now() -> f64;
}

pub fn now_secs() -> f32 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Instant::now().duration_since(*START_TIME).as_secs_f64() as f32
    }
    #[cfg(target_arch = "wasm32")]
    {
        let elapsed = performance_now() - *START_TIME;
        (elapsed / 1000.0) as f32 // Convert milliseconds to seconds
    }
}
