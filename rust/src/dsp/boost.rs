pub struct Boost {
    gain: f32,
}

impl Boost {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s *= self.gain;
        }
    }
}
