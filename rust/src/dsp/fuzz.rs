pub struct Fuzz {
    fuzz: f32,  // 0.0–1.0
    level: f32, // 0.0–1.0
}

impl Fuzz {
    pub fn new(fuzz: f32, level: f32) -> Self {
        Self {
            fuzz: fuzz.clamp(0.0, 1.0),
            level: level.clamp(0.0, 1.0),
        }
    }

    pub fn set_fuzz(&mut self, fuzz: f32)   { self.fuzz = fuzz.clamp(0.0, 1.0); }
    pub fn set_level(&mut self, level: f32) { self.level = level.clamp(0.0, 1.0); }

    pub fn process(&mut self, buffer: &mut [f32]) {
        let drive = 1.0 + self.fuzz * 49.0; // 1–50×
        for sample in buffer.iter_mut() {
            let driven = *sample * drive;
            // Double-cascaded tanh (emulates Big Muff two-stage clipping)
            let stage1 = driven.tanh();
            let stage2 = stage1.tanh();
            *sample = stage2 * self.level;
        }
    }
}
