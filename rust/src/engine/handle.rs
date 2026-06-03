use ringbuf::traits::Producer;
use crate::dsp::chain::EffectParams;
use crate::dsp::params::*;
use crate::engine::{Command, RbProd};

pub struct EngineHandle {
    prod: RbProd<Command>,
}

impl EngineHandle {
    pub fn new(prod: RbProd<Command>) -> Self {
        Self { prod }
    }

    pub fn toggle_bypass(&mut self, slot: u8, bypass: bool) -> Result<(), String> {
        self.prod
            .try_push(Command::ToggleBypass { slot, bypass })
            .map_err(|_| "command ring full".to_string())
    }

    pub fn set_param(&mut self, slot: u8, json: &str) -> Result<(), String> {
        let params = parse_params(slot, json)?;
        self.prod
            .try_push(Command::SetParam { params })
            .map_err(|_| "command ring full".to_string())
    }
}

fn parse_params(slot: u8, json: &str) -> Result<EffectParams, String> {
    match slot {
        0 => serde_json::from_str::<NoiseGateParams>(json)
            .map(EffectParams::NoiseGate)
            .map_err(|e| e.to_string()),
        1 => serde_json::from_str::<CompressorParams>(json)
            .map(EffectParams::Compressor)
            .map_err(|e| e.to_string()),
        2 => serde_json::from_str::<OverdriveParams>(json)
            .map(EffectParams::Overdrive)
            .map_err(|e| e.to_string()),
        3 => serde_json::from_str::<DistortionParams>(json)
            .map(EffectParams::Distortion)
            .map_err(|e| e.to_string()),
        4 => serde_json::from_str::<FuzzParams>(json)
            .map(EffectParams::Fuzz)
            .map_err(|e| e.to_string()),
        5 => serde_json::from_str::<ChorusParams>(json)
            .map(EffectParams::Chorus)
            .map_err(|e| e.to_string()),
        6 => serde_json::from_str::<TremoloParams>(json)
            .map(EffectParams::Tremolo)
            .map_err(|e| e.to_string()),
        7 => serde_json::from_str::<DelayParams>(json)
            .map(EffectParams::Delay)
            .map_err(|e| e.to_string()),
        8 => serde_json::from_str::<ReverbParams>(json)
            .map(EffectParams::Reverb)
            .map_err(|e| e.to_string()),
        _ => Err(format!("unknown slot {slot}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::make_engine;

    #[test]
    fn toggle_bypass_enables_noise_gate() {
        let (mut engine, prod) = make_engine(44100.0);
        let mut handle = EngineHandle::new(prod);
        handle.toggle_bypass(0, false).unwrap();
        let mut buf = vec![0.001f32; 512];
        engine.process_block(&mut buf);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn set_param_json_lowers_noise_gate_threshold() {
        let (mut engine, prod) = make_engine(44100.0);
        let mut handle = EngineHandle::new(prod);
        handle.toggle_bypass(0, false).unwrap();
        handle.set_param(0, r#"{"threshold": 0.01}"#).unwrap();
        let mut buf = vec![0.5f32; 512];
        engine.process_block(&mut buf);
        assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
    }

    #[test]
    fn set_param_returns_error_for_unknown_slot() {
        let (_engine, prod) = make_engine(44100.0);
        let mut handle = EngineHandle::new(prod);
        assert!(handle.set_param(9, r#"{}"#).is_err());
    }

    #[test]
    fn set_param_returns_error_for_invalid_json() {
        let (_engine, prod) = make_engine(44100.0);
        let mut handle = EngineHandle::new(prod);
        assert!(handle.set_param(0, r#"not json"#).is_err());
    }

    #[test]
    fn set_param_accepts_all_9_slots() {
        let (_engine, prod) = make_engine(44100.0);
        let mut handle = EngineHandle::new(prod);
        assert!(handle.set_param(0, r#"{"threshold": 0.01}"#).is_ok());
        assert!(handle.set_param(1, r#"{"threshold_db": -18.0, "ratio": 4.0, "attack": 0.01, "release": 0.1}"#).is_ok());
        assert!(handle.set_param(2, r#"{"drive": 3.0, "tone": 0.5}"#).is_ok());
        assert!(handle.set_param(3, r#"{"drive": 8.0, "level": 0.5}"#).is_ok());
        assert!(handle.set_param(4, r#"{"fuzz": 0.7, "level": 0.7}"#).is_ok());
        assert!(handle.set_param(5, r#"{"rate": 0.5, "depth": 1.5, "mix": 0.5}"#).is_ok());
        assert!(handle.set_param(6, r#"{"rate": 4.0, "depth": 0.5}"#).is_ok());
        assert!(handle.set_param(7, r#"{"time_ms": 300.0, "feedback": 0.4, "mix": 0.4}"#).is_ok());
        assert!(handle.set_param(8, r#"{"room_size": 0.5, "mix": 0.3}"#).is_ok());
    }
}
