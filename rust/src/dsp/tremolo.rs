pub struct Tremolo;

impl Tremolo {
    pub fn new(_sr: f32, _rate_hz: f32, _depth: f32) -> Self { Self }
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
