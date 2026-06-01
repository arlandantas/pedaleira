mod dsp_wav;
use rust_lib_pedaleira::dsp::overdrive::Overdrive;

#[test]
fn overdrive_clips_softly() {
    let mut od = Overdrive::new(5.0, 0.5);
    let mut buf = vec![2.0f32; 256];
    od.process(&mut buf);
    assert!(buf.iter().all(|&s| s >= -1.5 && s <= 1.5), "soft clip must bound output");
}

#[test]
fn overdrive_small_input_passes() {
    let mut od = Overdrive::new(1.0, 0.0);
    let small = vec![0.1f32; 256];
    let mut buf = small.clone();
    od.process(&mut buf);
    // drive=1.0 → driven = 0.1, clipped ≈ 0.1 * 1.5 * (1 - 0.1²/3) ≈ 0.149
    for &s in buf.iter() {
        assert!(s > 0.0 && s < 0.3, "small input should pass with minimal distortion, got {s}");
    }
}

#[test]
fn overdrive_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut od = Overdrive::new(5.0, 0.5);
    let mut buf = samples.clone();
    od.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_3_overdrive.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()));
}
