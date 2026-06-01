use core::f32::consts::TAU;

pub struct Tremolo {
    phase: f32,
    phase_increment: f32,
    depth: f32,
}

impl Tremolo {
    pub fn new(sample_rate: f32, rate_hz: f32, depth: f32) -> Self {
        Self {
            phase: 0.0,
            phase_increment: TAU * rate_hz / sample_rate,
            depth: depth.clamp(0.0, 1.0),
        }
    }

    pub fn set_rate(&mut self, sample_rate: f32, rate_hz: f32) {
        self.phase_increment = TAU * rate_hz / sample_rate;
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let lfo = (self.phase.sin() + 1.0) * 0.5; // 0.0–1.0
            let gain = 1.0 - self.depth * (1.0 - lfo);
            *sample *= gain;
            self.phase += self.phase_increment;
            if self.phase >= TAU {
                self.phase -= TAU;
            }
        }
    }
}
