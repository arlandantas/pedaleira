pub struct Chorus;

impl Chorus {
    pub fn new(_sr: f32, _rate_hz: f32, _depth_ms: f32, _mix: f32) -> Self { Self }
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
