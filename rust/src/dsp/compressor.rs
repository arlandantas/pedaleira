pub struct Compressor;

impl Compressor {
    pub fn new(_sr: f32, _threshold_db: f32, _ratio: f32, _attack: f32, _release: f32) -> Self { Self }
    pub fn process(&mut self, _buffer: &mut [f32]) {}
}
