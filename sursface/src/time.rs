#![allow(dead_code, unused_imports)]

use wasm_bindgen_futures::wasm_bindgen;
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
    static ref START_TIME: wasm_timer::SystemTime = wasm_timer::SystemTime::now();
}

pub fn now_secs() -> f32 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Instant::now().duration_since(*START_TIME).as_secs_f64() as f32
    }
    #[cfg(target_arch = "wasm32")]
    {
        let elapsed = wasm_timer::SystemTime::now().duration_since(*START_TIME).unwrap();
        (elapsed.as_secs() as f32 + elapsed.subsec_millis() as f32 / 1000.0) * 10f32
    }
}