use crate::dsp::smooth::SmoothedParam;

pub struct Boost {
    gain: SmoothedParam,
}

impl Boost {
    pub fn new(gain: f32, sample_rate: f32) -> Self {
        Self { gain: SmoothedParam::new(gain, sample_rate, 5.0) }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain.set(gain);
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s *= self.gain.next();
        }
    }
}
