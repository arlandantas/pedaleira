mod dsp_wav;
use rust_lib_pedaleira::dsp::distortion::Distortion;

#[test]
fn distortion_saturates_hot_signal() {
    let mut dist = Distortion::new(20.0, 1.0, 44100.0);
    let hot = vec![0.5f32; 2048];
    let mut buf = hot.clone();
    dist.process(&mut buf);
    // Output must be bounded
    assert!(buf.iter().all(|&s| s.abs() <= 1.0), "output must not clip to ±1");
    // With drive=20, 0.5*20=10 → saturates hard → output near ±1
    let avg: f32 = buf[1000..].iter().map(|s| s.abs()).sum::<f32>() / 1048.0;
    assert!(avg > 0.4, "should be heavily saturated, avg={avg:.4}");
}

#[test]
fn distortion_output_is_finite() {
    let mut dist = Distortion::new(50.0, 1.0, 44100.0);
    let mut buf: Vec<f32> = (0..1024).map(|i| (i as f32 * 0.01).sin()).collect();
    dist.process(&mut buf);
    assert!(buf.iter().all(|s| s.is_finite()), "no NaN or Inf");
}

#[test]
fn distortion_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut dist = Distortion::new(12.0, 0.6, 44100.0);
    let mut buf = samples.clone();
    dist.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_4_distortion.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()));
}
