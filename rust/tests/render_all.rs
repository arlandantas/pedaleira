mod dsp_wav;
use rust_lib_pedaleira::dsp::{
    noise_gate::NoiseGate, compressor::Compressor, overdrive::Overdrive,
    distortion::Distortion, fuzz::Fuzz, tremolo::Tremolo,
    chorus::Chorus, delay::Delay, reverb::Reverb,
    chain::EffectsChain,
};

const DI: &str = "test_assets/di_guitar.wav";
const SR: f32 = 44100.0;

fn render<F: FnMut(&mut [f32])>(name: &str, mut effect: F) {
    let samples = dsp_wav::load_wav_mono_f32(DI);
    let mut buf = samples.clone();
    effect(&mut buf);
    let path = format!("/tmp/pedaleira_out_{}.wav", name);
    dsp_wav::save_wav_mono_f32(&path, &buf, SR as u32);
    println!("  -> {}", path);
}

#[test]
fn render_all_effects() {
    dsp_wav::ensure_test_di_wav(DI);
    println!("\n=== Rendered WAVs (open in Audacity or play with mpv) ===");
    render("1_noise_gate",  |b| NoiseGate::new(0.02).process(b));
    render("2_compressor",  |b| Compressor::new(SR, -18.0, 4.0, 0.01, 0.1).process(b));
    render("3_overdrive",   |b| Overdrive::new(5.0, 0.5).process(b));
    render("4_distortion",  |b| Distortion::new(12.0, 0.6).process(b));
    render("5_fuzz",        |b| Fuzz::new(0.8, 0.7).process(b));
    render("6_tremolo",     |b| Tremolo::new(SR, 5.0, 0.8).process(b));
    render("7_chorus",      |b| Chorus::new(SR, 0.5, 1.5, 0.6).process(b));
    render("8_delay",       |b| Delay::new(SR, 300.0, 0.4, 0.4).process(b));
    render("9_reverb",      |b| Reverb::new(SR, 0.6, 0.4).process(b));
    render("full_chain", |b| {
        let mut chain = EffectsChain::new(SR);
        for i in 0..9 { chain.set_bypass(i, false); }
        chain.process(b);
    });
    println!("Play: mpv /tmp/pedaleira_out_*.wav");
    println!("=======================================================\n");
}
