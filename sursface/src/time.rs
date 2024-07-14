use lazy_static::lazy_static;

lazy_static! {
    static ref START_TIME: web_time::Instant = web_time::Instant::now();
}

pub fn now_secs() -> f32 {
    web_time::Instant::now()
        .duration_since(*START_TIME)
        .as_secs_f64() as f32
}
