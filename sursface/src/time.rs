#![allow(dead_code, unused_imports)]

use wasm_bindgen_futures::wasm_bindgen;
use wasm_timer::{Instant};
use std::convert::TryInto;
use std::ops::{Add, Sub, AddAssign, SubAssign};
use lazy_static::lazy_static;

use crate::wasm_bindgen::prelude::wasm_bindgen;

#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

#[cfg(target_arch = "wasm32")]
lazy_static! {
    static ref START_TIME: wasm_timer::SystemTime = wasm_timer::SystemTime::now();
}


pub fn now_secs() -> f32 {
    #[cfg(not(target_arch = "wasm32"))] {
        Instant::now().duration_since(*START_TIME).as_secs_f64() as f32
    }
    #[cfg(target_arch = "wasm32")] {
        wasm_timer::SystemTime::now().duration_since(*START_TIME).unwrap().as_secs_f64() as f32
    }
}