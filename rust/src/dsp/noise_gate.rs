pub struct NoiseGate;

impl NoiseGate {
    pub fn new(_threshold: f32) -> Self { Self }
    pub fn set_threshold(&mut self, _t: f32) {}
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
