// Timer
use std::time::Instant;
pub struct Timer(Instant, &'static str);

impl Timer{
    pub fn new(reason: &'static str)->Self{
        let mut now = Instant::now();
        Timer(now, reason)
    }
}
impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.0.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        println!("Timer {}: {}", self.1, sec);
    }
}