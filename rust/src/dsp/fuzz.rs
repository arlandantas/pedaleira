pub struct Fuzz;

impl Fuzz {
    pub fn new(_drive: f32, _level: f32) -> Self { Self }
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
