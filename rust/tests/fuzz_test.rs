mod dsp_wav;
use rust_lib_pedaleira::dsp::fuzz::Fuzz;

#[test]
fn fuzz_produces_square_like_waveform() {
    let mut fuzz = Fuzz::new(1.0, 1.0, 44100.0); // max fuzz
    let sine: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin() * 0.8).collect();
    let mut buf = sine.clone();
    fuzz.process(&mut buf);
    // Heavy fuzz with double tanh → most samples should be significantly compressed/saturated
    let saturated = buf.iter().filter(|&&s| s.abs() > 0.5).count();
    assert!(saturated > 200, "fuzz at max should produce heavily saturated waveform, saturated={saturated}");
}

#[test]
fn fuzz_zero_fuzz_passes_signal() {
    let mut fuzz = Fuzz::new(0.0, 1.0, 44100.0);
    let input = vec![0.5f32; 256];
    let mut buf = input.clone();
    fuzz.process(&mut buf);
    // drive=1.0 at fuzz=0 → tanh(0.5).tanh() ≈ 0.435
    for &s in buf.iter() {
        assert!(s > 0.3 && s < 0.6, "zero fuzz should pass signal lightly processed, got {s}");
    }
}

#[test]
fn fuzz_output_bounded() {
    let mut fuzz = Fuzz::new(1.0, 1.0, 44100.0);
    let mut buf: Vec<f32> = (0..1024).map(|i| (i as f32 * 0.05).sin()).collect();
    fuzz.process(&mut buf);
    assert!(buf.iter().all(|&s| s.abs() <= 1.0), "tanh output must be bounded to ±1");
}

#[test]
fn fuzz_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut fuzz = Fuzz::new(0.8, 0.7, 44100.0);
    let mut buf = samples.clone();
    fuzz.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_5_fuzz.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()));
}
