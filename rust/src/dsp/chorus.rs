use core::f32::consts::TAU;
use crate::dsp::smooth::SmoothedParam;

const MAX_DELAY_SAMPLES: usize = 4096;

pub struct Chorus {
    buffer: [f32; MAX_DELAY_SAMPLES],
    write_pos: usize,
    lfo_phase: f32,
    lfo_increment: f32,
    depth_samples: f32,
    center_delay_samples: f32,
    mix: SmoothedParam,
}

impl Chorus {
    /// rate_hz: LFO rate, depth_ms: modulation depth in ms, mix: 0.0–1.0 wet
    pub fn new(sample_rate: f32, rate_hz: f32, depth_ms: f32, mix: f32) -> Self {
        Self {
            buffer: [0.0; MAX_DELAY_SAMPLES],
            write_pos: 0,
            lfo_phase: 0.0,
            lfo_increment: TAU * rate_hz / sample_rate,
            depth_samples: depth_ms * sample_rate / 1000.0,
            center_delay_samples: 7.0 * sample_rate / 1000.0, // 7 ms center
            mix: SmoothedParam::new(mix.clamp(0.0, 1.0), sample_rate, 5.0),
        }
    }

    pub fn set_rate(&mut self, sample_rate: f32, rate_hz: f32) {
        self.lfo_increment = TAU * rate_hz / sample_rate;
    }

    pub fn set_mix(&mut self, mix: f32) { self.mix.set(mix.clamp(0.0, 1.0)); }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            self.buffer[self.write_pos] = *sample;

            let lfo = self.lfo_phase.sin();
            let delay = self.center_delay_samples + lfo * self.depth_samples;
            let delay_int = delay.floor() as usize;
            let frac = delay.fract();

            let read_pos = (self.write_pos + MAX_DELAY_SAMPLES - delay_int) % MAX_DELAY_SAMPLES;
            let read_next = (read_pos + MAX_DELAY_SAMPLES - 1) % MAX_DELAY_SAMPLES;
            let delayed = self.buffer[read_pos] * (1.0 - frac) + self.buffer[read_next] * frac;

            let mix = self.mix.next();
            *sample = *sample * (1.0 - mix) + delayed * mix;

            self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;
            self.lfo_phase += self.lfo_increment;
            if self.lfo_phase >= TAU {
                self.lfo_phase -= TAU;
            }
        }
    }
}
