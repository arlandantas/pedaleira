mod dsp_wav;
use rust_lib_pedaleira::dsp::noise_gate::NoiseGate;

#[test]
fn noise_gate_passes_loud_signal() {
    let mut gate = NoiseGate::new(0.01);
    let mut buf = vec![0.5f32; 512];
    gate.process(&mut buf);
    assert!(buf.iter().all(|&s| s != 0.0), "loud signal should pass");
}

#[test]
fn noise_gate_mutes_quiet_signal() {
    let mut gate = NoiseGate::new(0.01);
    // Need more than hold_samples (441) of quiet to fully close
    let mut buf = vec![0.005f32; 1024];
    gate.process(&mut buf);
    // After hold expires, remaining samples should be muted
    assert!(buf[500..].iter().all(|&s| s == 0.0), "quiet signal should be muted after hold expires");
}

#[test]
fn noise_gate_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut gate = NoiseGate::new(0.02);
    let mut buf = samples.clone();
    gate.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_1_noise_gate.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
}
