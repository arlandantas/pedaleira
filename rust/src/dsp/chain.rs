use crate::dsp::{
    noise_gate::NoiseGate,
    compressor::Compressor,
    overdrive::Overdrive,
    distortion::Distortion,
    fuzz::Fuzz,
    chorus::Chorus,
    tremolo::Tremolo,
    delay::Delay,
    reverb::Reverb,
};
use crate::dsp::params::*;

pub struct EffectsChain {
    noise_gate:  NoiseGate,
    compressor:  Compressor,
    overdrive:   Overdrive,
    distortion:  Distortion,
    fuzz:        Fuzz,
    chorus:      Chorus,
    tremolo:     Tremolo,
    delay:       Delay,
    reverb:      Reverb,
    bypass: [bool; 9], // indices 0–8: noise_gate..reverb
}

impl EffectsChain {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            noise_gate:  NoiseGate::new(0.01),
            compressor:  Compressor::new(sample_rate, -18.0, 4.0, 0.01, 0.1),
            overdrive:   Overdrive::new(3.0, 0.5),
            distortion:  Distortion::new(8.0, 0.5),
            fuzz:        Fuzz::new(0.7, 0.7),
            chorus:      Chorus::new(sample_rate, 0.5, 1.5, 0.5),
            tremolo:     Tremolo::new(sample_rate, 4.0, 0.5),
            delay:       Delay::new(sample_rate, 300.0, 0.4, 0.4),
            reverb:      Reverb::new(sample_rate, 0.5, 0.3),
            bypass:      [true; 9], // all bypassed by default
        }
    }

    /// slot 0=noise_gate, 1=compressor, 2=overdrive, 3=distortion,
    /// 4=fuzz, 5=chorus, 6=tremolo, 7=delay, 8=reverb
    pub fn set_bypass(&mut self, slot: usize, bypass: bool) {
        if slot < 9 { self.bypass[slot] = bypass; }
    }

    pub fn is_bypassed(&self, slot: usize) -> bool {
        slot >= 9 || self.bypass[slot]
    }

    pub fn apply_params(&mut self, sample_rate: f32, params: &EffectParams) {
        match params {
            EffectParams::NoiseGate(p)  => self.noise_gate.set_threshold(p.threshold),
            EffectParams::Compressor(p) => {
                self.compressor.set_threshold_db(p.threshold_db);
                self.compressor.set_ratio(p.ratio);
            }
            EffectParams::Overdrive(p)  => {
                self.overdrive.set_drive(p.drive);
                self.overdrive.set_tone(p.tone);
            }
            EffectParams::Distortion(p) => {
                self.distortion.set_drive(p.drive);
                self.distortion.set_level(p.level);
            }
            EffectParams::Fuzz(p)       => {
                self.fuzz.set_fuzz(p.fuzz);
                self.fuzz.set_level(p.level);
            }
            EffectParams::Chorus(p)     => {
                self.chorus.set_rate(sample_rate, p.rate);
                self.chorus.set_mix(p.mix);
            }
            EffectParams::Tremolo(p)    => {
                self.tremolo.set_rate(sample_rate, p.rate);
                self.tremolo.set_depth(p.depth);
            }
            EffectParams::Delay(p)      => {
                self.delay.set_delay_ms(sample_rate, p.time_ms);
                self.delay.set_feedback(p.feedback);
                self.delay.set_mix(p.mix);
            }
            EffectParams::Reverb(p)     => {
                self.reverb.set_room_size(p.room_size);
                self.reverb.set_mix(p.mix);
            }
        }
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        if !self.bypass[0] { self.noise_gate.process(buffer); }
        if !self.bypass[1] { self.compressor.process(buffer); }
        if !self.bypass[2] { self.overdrive.process(buffer); }
        if !self.bypass[3] { self.distortion.process(buffer); }
        if !self.bypass[4] { self.fuzz.process(buffer); }
        if !self.bypass[5] { self.chorus.process(buffer); }
        if !self.bypass[6] { self.tremolo.process(buffer); }
        if !self.bypass[7] { self.delay.process(buffer); }
        if !self.bypass[8] { self.reverb.process(buffer); }
    }
}

pub enum EffectParams {
    NoiseGate(NoiseGateParams),
    Compressor(CompressorParams),
    Overdrive(OverdriveParams),
    Distortion(DistortionParams),
    Fuzz(FuzzParams),
    Chorus(ChorusParams),
    Tremolo(TremoloParams),
    Delay(DelayParams),
    Reverb(ReverbParams),
}
