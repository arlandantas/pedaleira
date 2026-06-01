mod dsp_wav;
use rust_lib_pedaleira::dsp::chorus::Chorus;

#[test]
fn chorus_does_not_clip() {
    let mut chorus = Chorus::new(44100.0, 0.5, 1.5, 0.7);
    let sine: Vec<f32> = (0..4096).map(|i| (i as f32 * 0.05).sin()).collect();
    let mut buf = sine.clone();
    chorus.process(&mut buf);
    assert!(buf.iter().all(|&s| s.abs() <= 2.0), "chorus output must not explode");
}

#[test]
fn chorus_output_differs_from_input() {
    let mut chorus = Chorus::new(44100.0, 0.5, 1.5, 0.7);
    let sine: Vec<f32> = (0..4096).map(|i| (i as f32 * 0.05).sin()).collect();
    let mut buf = sine.clone();
    chorus.process(&mut buf);
    let diff: f32 = sine.iter().zip(buf.iter()).map(|(a, b)| (a - b).abs()).sum();
    assert!(diff > 0.1, "chorus must modify the signal, diff={diff}");
}

#[test]
fn chorus_zero_mix_passes_dry() {
    let mut chorus = Chorus::new(44100.0, 1.0, 2.0, 0.0);
    let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin()).collect();
    let mut buf = input.clone();
    chorus.process(&mut buf);
    for (&orig, &out) in input.iter().zip(buf.iter()) {
        assert!((orig - out).abs() < 1e-6, "mix=0 should pass dry signal unchanged");
    }
}

#[test]
fn chorus_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut chorus = Chorus::new(44100.0, 0.5, 1.5, 0.6);
    let mut buf = samples.clone();
    chorus.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_7_chorus.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()));
}
