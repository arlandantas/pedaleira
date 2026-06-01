mod dsp_wav;
use rust_lib_pedaleira::dsp::delay::Delay;

#[test]
fn delay_echo_appears_after_delay_time() {
    let sr = 44100.0f32;
    let delay_ms = 10.0f32;
    let delay_samples = (sr * delay_ms / 1000.0) as usize; // 441 samples
    let buf_len = delay_samples + 200;

    let mut delay = Delay::new(sr, delay_ms, 0.5, 1.0); // mix=1 for pure echo
    let mut buf = vec![0.0f32; buf_len];
    buf[0] = 1.0; // impulse at t=0
    delay.process(&mut buf);

    // Echo should appear at delay_samples position
    assert!(
        buf[delay_samples].abs() > 0.3,
        "echo should appear at delay_samples={delay_samples}, got {}",
        buf[delay_samples]
    );
}

#[test]
fn delay_zero_mix_passes_dry() {
    let mut delay = Delay::new(44100.0, 300.0, 0.5, 0.0);
    let input = vec![0.5f32; 512];
    let mut buf = input.clone();
    delay.process(&mut buf);
    for (&orig, &out) in input.iter().zip(buf.iter()) {
        assert!((orig - out).abs() < 1e-6, "mix=0 must pass dry signal");
    }
}

#[test]
fn delay_feedback_does_not_blow_up() {
    let mut delay = Delay::new(44100.0, 50.0, 0.95, 0.5);
    let mut buf = vec![0.0f32; 44100];
    buf[0] = 1.0;
    delay.process(&mut buf);
    assert!(buf.iter().all(|s| s.is_finite()), "no NaN/Inf");
    assert!(buf.iter().all(|s| s.abs() <= 2.0), "feedback=0.95 should stay bounded");
}

#[test]
fn delay_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut delay = Delay::new(44100.0, 300.0, 0.4, 0.4);
    let mut buf = samples.clone();
    delay.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_8_delay.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()));
}
