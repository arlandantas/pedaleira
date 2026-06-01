pub struct Delay;

impl Delay {
    pub fn new(_sr: f32, _time_ms: f32, _feedback: f32, _mix: f32) -> Self { Self }
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
