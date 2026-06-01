pub struct NoiseGate {
    threshold: f32,
    hold_samples: usize,
    hold_counter: usize,
    open: bool,
}

impl NoiseGate {
    pub fn new(threshold: f32) -> Self {
        Self { threshold, hold_samples: 441, hold_counter: 0, open: false }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let abs = sample.abs();
            if abs >= self.threshold {
                self.open = true;
                self.hold_counter = self.hold_samples;
            } else if self.hold_counter > 0 {
                self.hold_counter -= 1;
            } else {
                self.open = false;
            }
            if !self.open {
                *sample = 0.0;
            }
        }
    }
}
