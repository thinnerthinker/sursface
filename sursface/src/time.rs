#![allow(dead_code, unused_imports)]

use wasm_bindgen::prelude::*;
use std::convert::TryInto;
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration as StdDuration, Instant as StdInstant}; // Import directly from std::time

pub use std::time::*; // Re-export std::time::* if needed

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Instant(StdInstant);

#[cfg(not(target_arch = "wasm32"))]
impl Instant {
    pub fn now() -> Self {
        Self(StdInstant::now())
    }
    pub fn duration_since(&self, earlier: Instant) -> StdDuration {
        self.0.duration_since(earlier.0)
    }
    pub fn elapsed(&self) -> StdDuration {
        self.0.elapsed()
    }
    pub fn checked_add(&self, duration: StdDuration) -> Option<Self> {
        self.0.checked_add(duration).map(Self)
    }
    pub fn checked_sub(&self, duration: StdDuration) -> Option<Self> {
        self.0.checked_sub(duration).map(Self)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance, js_name = now)]
    fn performance_now() -> f64;
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Instant(f64);

#[cfg(target_arch = "wasm32")]
impl Instant {
    pub fn now() -> Self {
        Self(performance_now() / 1000.0) // Convert milliseconds to seconds
    }
    pub fn duration_since(&self, earlier: Instant) -> StdDuration {
        let elapsed_secs = self.0 - earlier.0;
        StdDuration::from_secs_f64(elapsed_secs)
    }
    pub fn elapsed(&self) -> StdDuration {
        Self::now().duration_since(*self)
    }
    pub fn checked_add(&self, duration: StdDuration) -> Option<Self> {
        Some(Self(self.0 + duration.as_secs_f64()))
    }
    pub fn checked_sub(&self, duration: StdDuration) -> Option<Self> {
        Some(Self(self.0 - duration.as_secs_f64()))
    }
}

impl Add<StdDuration> for Instant {
    type Output = Instant;
    fn add(self, other: StdDuration) -> Instant {
        self.checked_add(other).unwrap()
    }
}

impl Sub<StdDuration> for Instant {
    type Output = Instant;
    fn sub(self, other: StdDuration) -> Instant {
        self.checked_sub(other).unwrap()
    }
}

impl Sub<Instant> for Instant {
    type Output = StdDuration;
    fn sub(self, other: Instant) -> StdDuration {
        self.duration_since(other)
    }
}

impl AddAssign<StdDuration> for Instant {
    fn add_assign(&mut self, other: StdDuration) {
        *self = *self + other;
    }
}

impl SubAssign<StdDuration> for Instant {
    fn sub_assign(&mut self, other: StdDuration) {
        *self = *self - other;
    }
}

use lazy_static::lazy_static;

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

pub fn now() -> f32 {
    Instant::now().duration_since(*START_TIME).as_secs_f64() as f32 // Convert to f32 explicitly
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
