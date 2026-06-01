pub struct Distortion;

impl Distortion {
    pub fn new(_gain: f32, _level: f32) -> Self { Self }
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
