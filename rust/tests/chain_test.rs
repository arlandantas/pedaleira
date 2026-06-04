mod dsp_wav;
use rust_lib_pedaleira::dsp::chain::{EffectsChain, EffectParams};
use rust_lib_pedaleira::dsp::params::*;

#[test]
fn chain_all_bypassed_passes_signal() {
    let mut chain = EffectsChain::new(44100.0);
    // Default: all bypassed
    let input = vec![0.3f32; 512];
    let mut buf = input.clone();
    chain.process(&mut buf);
    for (&orig, &out) in input.iter().zip(buf.iter()) {
        assert!((orig - out).abs() < 1e-6, "all bypassed: signal must pass unchanged");
    }
}

#[test]
fn chain_processes_without_panic() {
    let mut chain = EffectsChain::new(44100.0);
    for i in 0..10 { chain.set_bypass(i, false); }
    let mut buf = vec![0.3f32; 512];
    chain.process(&mut buf);
    assert!(buf.iter().all(|s| s.is_finite()), "full chain must produce finite output");
}

#[test]
fn chain_bypass_gates_effect() {
    let mut chain = EffectsChain::new(44100.0);
    chain.set_bypass(0, false); // enable noise gate only
    let quiet = vec![0.001f32; 1500]; // below gate threshold, longer than hold
    let mut buf = quiet.clone();
    chain.process(&mut buf);
    // Noise gate active → quiet signal should be gated (zeroed after hold)
    let tail_sum: f32 = buf[1000..].iter().map(|s| s.abs()).sum();
    assert!(tail_sum < 0.01, "noise gate should mute quiet signal after hold, sum={tail_sum}");
}

#[test]
fn chain_apply_params_updates_effect() {
    // Verify apply_params reaches the effect by comparing two compressor configs.
    // A very aggressive ratio (100:1) with a low threshold should heavily reduce a loud signal,
    // producing less output than a gentle ratio (1.1:1) with the same threshold.
    let signal = vec![0.9f32; 1024];

    let mut chain_heavy = EffectsChain::new(44100.0);
    chain_heavy.set_bypass(1, false);
    chain_heavy.apply_params(44100.0, &EffectParams::Compressor(CompressorParams {
        threshold_db: -20.0,
        ratio: 100.0,
        attack: 0.0001,
        release: 0.05,
    }));
    let mut buf_heavy = signal.clone();
    chain_heavy.process(&mut buf_heavy);

    let mut chain_gentle = EffectsChain::new(44100.0);
    chain_gentle.set_bypass(1, false);
    chain_gentle.apply_params(44100.0, &EffectParams::Compressor(CompressorParams {
        threshold_db: -20.0,
        ratio: 1.1,
        attack: 0.0001,
        release: 0.05,
    }));
    let mut buf_gentle = signal.clone();
    chain_gentle.process(&mut buf_gentle);

    let avg_heavy: f32 = buf_heavy[512..].iter().map(|s| s.abs()).sum::<f32>() / 512.0;
    let avg_gentle: f32 = buf_gentle[512..].iter().map(|s| s.abs()).sum::<f32>() / 512.0;
    assert!(
        avg_heavy < avg_gentle,
        "100:1 ratio should produce lower output than 1.1:1 ratio; heavy={avg_heavy}, gentle={avg_gentle}"
    );
}

#[test]
fn chain_wav_roundtrip_full() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut chain = EffectsChain::new(44100.0);
    // Enable all effects
    for i in 0..10 { chain.set_bypass(i, false); }
    let mut buf = samples.clone();
    chain.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_full_chain.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()));
}

#[test]
fn boost_at_gain_2_doubles_signal() {
    let mut chain = EffectsChain::new(44100.0);
    chain.set_bypass(9, false);
    chain.apply_params(44100.0, &EffectParams::Boost(BoostParams { gain: 2.0 }));
    let mut buf = vec![0.25f32; 512];
    chain.process(&mut buf);
    assert!(
        buf.iter().all(|&s| (s - 0.5).abs() < 1e-6),
        "boost x2 should double 0.25 to 0.5"
    );
}
