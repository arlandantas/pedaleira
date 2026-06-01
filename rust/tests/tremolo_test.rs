mod dsp_wav;
use rust_lib_pedaleira::dsp::tremolo::Tremolo;

#[test]
fn tremolo_modulates_amplitude() {
    let mut trem = Tremolo::new(44100.0, 5.0, 1.0); // 5 Hz, full depth
    let mut buf = vec![1.0f32; 44100]; // 1 second DC
    trem.process(&mut buf);
    let min = buf.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = buf.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(max > 0.9, "max should be near 1.0, got {max}");
    assert!(min < 0.1, "min should be near 0.0 at full depth, got {min}");
}

#[test]
fn tremolo_zero_depth_passes_unchanged() {
    let mut trem = Tremolo::new(44100.0, 5.0, 0.0);
    let input = vec![0.5f32; 512];
    let mut buf = input.clone();
    trem.process(&mut buf);
    for (&orig, &out) in input.iter().zip(buf.iter()) {
        assert!((orig - out).abs() < 1e-6, "zero depth should not change signal");
    }
}

#[test]
fn tremolo_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut trem = Tremolo::new(44100.0, 5.0, 0.8);
    let mut buf = samples.clone();
    trem.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_6_tremolo.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()));
}
