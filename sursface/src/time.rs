use web_time::Instant;

pub fn now() -> f32 {
    Instant::now().elapsed().as_secs_f64() as f32
}