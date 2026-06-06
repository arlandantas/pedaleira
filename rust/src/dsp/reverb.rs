use crate::dsp::smooth::SmoothedParam;

// Schroeder reverberator: 4 parallel comb filters → 2 series all-pass filters
const COMB_DELAYS: [usize; 4] = [1557, 1617, 1491, 1422];
const ALLPASS_DELAYS: [usize; 2] = [225, 556];

struct CombFilter {
    buffer: Vec<f32>,
    pos: usize,
    feedback: f32,
}

impl CombFilter {
    fn new(delay: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; delay],
            pos: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let out = self.buffer[self.pos];
        self.buffer[self.pos] = input + out * self.feedback;
        self.pos = (self.pos + 1) % self.buffer.len();
        out
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }
}

struct AllPassFilter {
    buffer: Vec<f32>,
    pos: usize,
}

impl AllPassFilter {
    fn new(delay: usize) -> Self {
        Self {
            buffer: vec![0.0; delay],
            pos: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.pos];
        self.buffer[self.pos] = input + delayed * 0.5;
        self.pos = (self.pos + 1) % self.buffer.len();
        delayed - input * 0.5
    }
}

pub struct Reverb {
    combs: [CombFilter; 4],
    allpasses: [AllPassFilter; 2],
    room_size: f32,
    mix: SmoothedParam,
}

impl Reverb {
    pub fn new(sample_rate: f32, room_size: f32, mix: f32) -> Self {
        let room_size = room_size.clamp(0.0, 0.98);
        Self {
            combs: [
                CombFilter::new(COMB_DELAYS[0], room_size),
                CombFilter::new(COMB_DELAYS[1], room_size),
                CombFilter::new(COMB_DELAYS[2], room_size),
                CombFilter::new(COMB_DELAYS[3], room_size),
            ],
            allpasses: [
                AllPassFilter::new(ALLPASS_DELAYS[0]),
                AllPassFilter::new(ALLPASS_DELAYS[1]),
            ],
            room_size,
            mix: SmoothedParam::new(mix.clamp(0.0, 1.0), sample_rate, 5.0),
        }
    }

    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size.clamp(0.0, 0.98);
        for comb in self.combs.iter_mut() {
            comb.set_feedback(self.room_size);
        }
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix.set(mix.clamp(0.0, 1.0));
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let input = *sample;
            // 4 parallel comb filters
            let wet = (self.combs[0].process(input)
                + self.combs[1].process(input)
                + self.combs[2].process(input)
                + self.combs[3].process(input))
                * 0.25;
            // 2 series all-pass filters
            let wet = self.allpasses[0].process(wet);
            let wet = self.allpasses[1].process(wet);
            let mix = self.mix.next();
            *sample = input * (1.0 - mix) + wet * mix;
        }
    }
}
