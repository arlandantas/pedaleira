pub struct Compressor {
    threshold_linear: f32,
    ratio: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
    makeup_gain: f32,
}

impl Compressor {
    /// threshold_db: e.g. -12.0, ratio: e.g. 4.0 (4:1), attack/release in seconds
    pub fn new(sample_rate: f32, threshold_db: f32, ratio: f32, attack_s: f32, release_s: f32) -> Self {
        let threshold_linear = db_to_linear(threshold_db);
        let attack_coeff = (-1.0 / (sample_rate * attack_s)).exp();
        let release_coeff = (-1.0 / (sample_rate * release_s)).exp();
        // makeup_gain: for a signal at threshold + 6dB (i.e., 2x threshold), with 4:1 ratio,
        // the gain reduction is: (6dB / 4) - 6dB = 1.5dB - 6dB = -4.5dB
        // makeup_gain should restore some of this loss. Use a modest amount.
        let makeup_gain_db = -threshold_db * (1.0 - 1.0 / ratio) / 4.0;
        let makeup_gain = db_to_linear(makeup_gain_db);
        Self { threshold_linear, ratio, attack_coeff, release_coeff, envelope: 0.0, makeup_gain }
    }

    pub fn set_threshold_db(&mut self, db: f32) {
        self.threshold_linear = db_to_linear(db);
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio;
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let abs = sample.abs();
            let coeff = if abs > self.envelope { self.attack_coeff } else { self.release_coeff };
            self.envelope = abs + coeff * (self.envelope - abs);

            let gain = if self.envelope > self.threshold_linear {
                let excess_db = linear_to_db(self.envelope / self.threshold_linear);
                let reduced_db = excess_db / self.ratio;
                db_to_linear(reduced_db - excess_db)
            } else {
                1.0
            };
            *sample *= gain * self.makeup_gain;
        }
    }
}

fn db_to_linear(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

fn linear_to_db(linear: f32) -> f32 {
    20.0 * linear.abs().max(1e-10).log10()
}
