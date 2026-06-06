use crate::dsp::smooth::SmoothedParam;

pub struct Fuzz {
    fuzz: f32,
    level: SmoothedParam,
}

impl Fuzz {
    pub fn new(fuzz: f32, level: f32, sample_rate: f32) -> Self {
        Self {
            fuzz: fuzz.clamp(0.0, 1.0),
            level: SmoothedParam::new(level.clamp(0.0, 1.0), sample_rate, 5.0),
        }
    }

    pub fn set_fuzz(&mut self, fuzz: f32) { self.fuzz = fuzz.clamp(0.0, 1.0); }
    pub fn set_level(&mut self, level: f32) { self.level.set(level.clamp(0.0, 1.0)); }

    pub fn process(&mut self, buffer: &mut [f32]) {
        let drive = 1.0 + self.fuzz * 49.0; // 1–50×
        for sample in buffer.iter_mut() {
            let driven = *sample * drive;
            let stage1 = driven.tanh();
            let stage2 = stage1.tanh();
            *sample = stage2 * self.level.next();
        }
    }
}
