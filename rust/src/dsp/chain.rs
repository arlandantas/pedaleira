pub struct EffectsChain;

impl EffectsChain {
    pub fn new(_sr: f32) -> Self { Self }
    pub fn set_bypass(&mut self, _slot: usize, _bypass: bool) {}
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
