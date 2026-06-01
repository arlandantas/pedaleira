mod dsp_wav;
use rust_lib_pedaleira::dsp::compressor::Compressor;

#[test]
fn compressor_reduces_loud_signal() {
    let mut comp = Compressor::new(44100.0, -12.0, 4.0, 0.01, 0.1);
    let loud = vec![0.9f32; 1024];
    let mut buf = loud.clone();
    comp.process(&mut buf);
    let avg_out: f32 = buf.iter().map(|s| s.abs()).sum::<f32>() / buf.len() as f32;
    let avg_in: f32 = loud.iter().map(|s| s.abs()).sum::<f32>() / loud.len() as f32;
    assert!(avg_out < avg_in, "compressor must reduce loud signal, avg_out={avg_out:.4} avg_in={avg_in:.4}");
}

#[test]
fn compressor_passes_quiet_signal() {
    let mut comp = Compressor::new(44100.0, -12.0, 4.0, 0.01, 0.1);
    // 0.01 linear ≈ -40 dBFS, well below -12 dBFS threshold
    let quiet = vec![0.01f32; 512];
    let mut buf = quiet.clone();
    comp.process(&mut buf);
    // Below threshold — output should be close to input (within makeup gain tolerance)
    // makeup_gain at -12dB, 4:1 = 10^(12*(1-0.25)/40) ≈ 1.0 so roughly unchanged
    for (&original, &processed) in quiet.iter().zip(buf.iter()) {
        assert!((original - processed).abs() < 0.1, "quiet signal should not be heavily affected");
    }
}

#[test]
fn compressor_wav_roundtrip() {
    dsp_wav::ensure_test_di_wav("test_assets/di_guitar.wav");
    let samples = dsp_wav::load_wav_mono_f32("test_assets/di_guitar.wav");
    let mut comp = Compressor::new(44100.0, -18.0, 4.0, 0.01, 0.1);
    let mut buf = samples.clone();
    comp.process(&mut buf);
    dsp_wav::save_wav_mono_f32("/tmp/pedaleira_out_2_compressor.wav", &buf, 44100);
    assert_eq!(buf.len(), samples.len());
    assert!(buf.iter().all(|s| s.is_finite()), "no NaN or Inf in output");
}
