mod dsp_wav;
use rust_lib_pedaleira::dsp::reverb::Reverb;

#[test]
fn reverb_extends_impulse() {
    let mut reverb = Reverb::new(44100.0, 0.7, 0.8);
    let mut buf = vec![0.0f32; 4096];
    buf[0] = 1.0; // impulse
    reverb.process(&mut buf);
    // Energy should persist after the initial impulse (reverb tail)
    let tail_energy: f32 = buf[50..].iter().map(|s| s * s).sum();
    assert!(
        tail_energy > 0.001,
        "reverb should produce a decay tail, energy={tail_energy:.6}"
    );
}

#[test]
fn reverb_zero_mix_passes_dry() {
    let mut reverb = Reverb::new(44100.0, 0.5, 0.0);
    let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin()).collect();
    let mut buf = input.clone();
    reverb.process(&mut buf);
    for (&orig, &out) in input.iter().zip(buf.iter()) {
        assert!((orig - out).abs() < 1e-6, "mix=0 must pass dry signal");
    }
}

#[test]
fn reverb_output_is_finite() {
    let mut reverb = Reverb::new(44100.0, 0.98, 1.0);
    let mut buf: Vec<f32> = (0..8192).map(|i| (i as f32 * 0.05).sin()).collect();
    reverb.process(&mut buf);
    assert!(
        buf.iter().all(|s| s.is_finite()),
        "no NaN/Inf even at max room size"
    );
}

#[test]
fn reverb_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut reverb = Reverb::new(44100.0, 0.6, 0.4);
    let mut buf = samples.clone();
    reverb.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_9_reverb.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()));
}
