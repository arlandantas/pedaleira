pub struct Reverb;

impl Reverb {
    pub fn new(_sr: f32, _decay: f32, _mix: f32) -> Self { Self }
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
