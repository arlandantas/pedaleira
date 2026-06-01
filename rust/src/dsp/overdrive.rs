pub struct Overdrive;

impl Overdrive {
    pub fn new(_gain: f32, _level: f32) -> Self { Self }
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
