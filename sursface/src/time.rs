#![allow(dead_code, unused_imports)]

use wasm_bindgen::prelude::*;
use std::convert::TryInto;
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant as StdInstant};

pub use std::time::*;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(StdInstant);

#[cfg(not(target_arch = "wasm32"))]
impl Instant {
    pub fn now() -> Self {
        Self(StdInstant::now())
    }
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.duration_since(earlier.0)
    }
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.0.checked_add(duration).map(Self)
    }
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(f64);

#[cfg(target_arch = "wasm32")]
impl Instant {
    pub fn now() -> Self {
        Self(performance_now() / 1000.0) // Convert milliseconds to seconds
    }
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let elapsed_secs = self.0 - earlier.0;
        Duration::from_secs_f64(elapsed_secs)
    }
    pub fn elapsed(&self) -> Duration {
        Self::now().duration_since(*self)
    }
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        Some(Self(self.0 + duration.as_secs_f64()))
    }
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        Some(Self(self.0 - duration.as_secs_f64()))
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, other: Duration) -> Instant {
        self.checked_add(other).unwrap()
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, other: Duration) -> Instant {
        self.checked_sub(other).unwrap()
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

use lazy_static::lazy_static;

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

pub fn now() -> f32 {
    let current_time = Instant::now().0;

    // Calculate elapsed time in seconds since the start time
    let elapsed_time = current_time - START_TIME.0;

    elapsed_time.as_secs_f64() as f32 // Convert to f32 explicitly
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
