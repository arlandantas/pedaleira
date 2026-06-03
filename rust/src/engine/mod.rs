pub mod handle;
pub mod runtime;

use ringbuf::{traits::*, HeapRb};
pub use ringbuf::HeapProd as RbProd;
pub use ringbuf::HeapCons as RbCons;

use crate::dsp::chain::{EffectsChain, EffectParams};

#[derive(Debug, Clone)]
pub enum Command {
    ToggleBypass { slot: u8, bypass: bool },
    SetParam { slot: u8, params: EffectParams },
}

pub struct AudioEngine {
    chain: EffectsChain,
    commands: RbCons<Command>,
    sample_rate: f32,
}

impl AudioEngine {
    pub fn new(commands: RbCons<Command>, sample_rate: f32) -> Self {
        Self {
            chain: EffectsChain::new(sample_rate),
            commands,
            sample_rate,
        }
    }

    pub fn process_block(&mut self, buf: &mut [f32]) {
        while let Some(cmd) = self.commands.try_pop() {
            match cmd {
                Command::ToggleBypass { slot, bypass } => {
                    self.chain.set_bypass(slot as usize, bypass);
                }
                Command::SetParam { slot: _, params } => {
                    self.chain.apply_params(self.sample_rate, &params);
                }
            }
        }
        self.chain.process(buf);
    }
}

pub fn make_engine(sample_rate: f32) -> (AudioEngine, RbProd<Command>) {
    let rb = HeapRb::<Command>::new(64);
    let (prod, cons) = rb.split();
    (AudioEngine::new(cons, sample_rate), prod)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::chain::EffectParams;
    use crate::dsp::params::NoiseGateParams;

    fn engine_with_prod() -> (AudioEngine, RbProd<Command>) {
        let rb = HeapRb::<Command>::new(64);
        let (prod, cons) = rb.split();
        (AudioEngine::new(cons, 44100.0), prod)
    }

    #[test]
    fn process_block_passes_through_when_all_bypassed() {
        let (mut engine, _prod) = engine_with_prod();
        let mut buf = vec![0.5f32; 512];
        engine.process_block(&mut buf);
        assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
    }

    #[test]
    fn toggle_bypass_enables_noise_gate_silencing_low_signal() {
        let (mut engine, mut prod) = engine_with_prod();
        // slot 0 = noise gate, default threshold=0.01; signal 0.001 < threshold
        prod.try_push(Command::ToggleBypass { slot: 0, bypass: false }).unwrap();
        let mut buf = vec![0.001f32; 512];
        engine.process_block(&mut buf);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn set_param_updates_noise_gate_threshold() {
        let (mut engine, mut prod) = engine_with_prod();
        prod.try_push(Command::ToggleBypass { slot: 0, bypass: false }).unwrap();
        // Lower threshold to 0.01 — signal 0.5 passes through
        prod.try_push(Command::SetParam {
            slot: 0,
            params: EffectParams::NoiseGate(NoiseGateParams { threshold: 0.01 }),
        }).unwrap();
        let mut buf = vec![0.5f32; 512];
        engine.process_block(&mut buf);
        assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
    }
}
