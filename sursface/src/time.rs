#![allow(dead_code, unused_imports)]

use wasm_bindgen::prelude::*;
use wasm_timer::{Instant};
use web_sys::window;
use std::convert::TryInto;
use std::ops::{Add, Sub, AddAssign, SubAssign};
use lazy_static::lazy_static;

#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

pub fn now_secs() -> f32 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Instant::now().duration_since(*START_TIME).as_secs_f64() as f32
    }
    #[cfg(target_arch = "wasm32")]
    {
        let window = window().expect("should have a window in this context");
        let performance = window.performance().unwrap();

        let elapsed = performance.now();
        (elapsed / 1000.0) as f32 // Convert milliseconds to seconds
    }
}