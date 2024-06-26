use web_time::Instant;
use lazy_static::lazy_static;

lazy_static! {
    static ref TIME_AT_START: Instant = Instant::now();
}

pub fn now() -> f32 {
    Instant::now().duration_since(*TIME_AT_START).as_secs_f64() as f32
}