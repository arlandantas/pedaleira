use crate::dsp::smooth::SmoothedParam;

pub struct Distortion {
    drive: f32,
    level: SmoothedParam,
    lp_state: f32,
}

impl Distortion {
    pub fn new(drive: f32, level: f32, sample_rate: f32) -> Self {
        Self {
            drive: drive.clamp(1.0, 50.0),
            level: SmoothedParam::new(level.clamp(0.0, 1.0), sample_rate, 5.0),
            lp_state: 0.0,
        }
    }

    pub fn set_drive(&mut self, drive: f32) { self.drive = drive.clamp(1.0, 50.0); }
    pub fn set_level(&mut self, level: f32) { self.level.set(level.clamp(0.0, 1.0)); }

    pub fn process(&mut self, buffer: &mut [f32]) {
        let lp_coeff = 0.9f32;
        for sample in buffer.iter_mut() {
            let driven = (*sample * self.drive).clamp(-1.0, 1.0);
            // Asymmetric hard clip modelling transistor soft-knee
            let clipped = if driven >= 0.0 {
                1.0 - (-2.0 * driven).exp()
            } else {
                -1.0 + (2.0 * driven).exp()
            };
            self.lp_state = lp_coeff * self.lp_state + (1.0 - lp_coeff) * clipped;
            *sample = self.lp_state * self.level.next();
        }
    }
}
