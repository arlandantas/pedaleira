pub struct Overdrive {
    drive: f32,  // 1.0–20.0
    tone: f32,   // 0.0–1.0
}

impl Overdrive {
    pub fn new(drive: f32, tone: f32) -> Self {
        Self {
            drive: drive.clamp(1.0, 20.0),
            tone: tone.clamp(0.0, 1.0),
        }
    }

    pub fn set_drive(&mut self, drive: f32) { self.drive = drive.clamp(1.0, 20.0); }
    pub fn set_tone(&mut self, tone: f32)   { self.tone = tone.clamp(0.0, 1.0); }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let driven = (*sample * self.drive).clamp(-1.0, 1.0);
            // Soft cubic saturation: x - x³/3  (range: [-2/3, 2/3], rescale to [-1, 1])
            let clipped = (driven - driven * driven * driven / 3.0) * 1.5;
            *sample = clipped * (1.0 - self.tone * 0.3);
        }
    }
}
