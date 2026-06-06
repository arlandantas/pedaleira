use crate::dsp::smooth::SmoothedParam;

const MAX_DELAY_SAMPLES: usize = 88200; // 2 seconds at 44100 Hz

pub struct Delay {
    buffer: Box<[f32; MAX_DELAY_SAMPLES]>,
    write_pos: usize,
    delay_samples: usize,
    feedback: f32,
    mix: SmoothedParam,
}

impl Delay {
    pub fn new(sample_rate: f32, delay_ms: f32, feedback: f32, mix: f32) -> Self {
        let delay_samples = ((sample_rate * delay_ms / 1000.0) as usize)
            .clamp(1, MAX_DELAY_SAMPLES - 1);
        Self {
            buffer: Box::new([0.0; MAX_DELAY_SAMPLES]),
            write_pos: 0,
            delay_samples,
            feedback: feedback.clamp(0.0, 0.95),
            mix: SmoothedParam::new(mix.clamp(0.0, 1.0), sample_rate, 5.0),
        }
    }

    pub fn set_delay_ms(&mut self, sample_rate: f32, delay_ms: f32) {
        self.delay_samples = ((sample_rate * delay_ms / 1000.0) as usize)
            .clamp(1, MAX_DELAY_SAMPLES - 1);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix.set(mix.clamp(0.0, 1.0));
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let read_pos = (self.write_pos + MAX_DELAY_SAMPLES - self.delay_samples)
                % MAX_DELAY_SAMPLES;
            let echo = self.buffer[read_pos];
            self.buffer[self.write_pos] = *sample + echo * self.feedback;
            let mix = self.mix.next();
            *sample = *sample * (1.0 - mix) + echo * mix;
            self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;
        }
    }
}
