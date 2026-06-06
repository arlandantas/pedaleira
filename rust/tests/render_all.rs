mod dsp_wav;
use rust_lib_pedaleira::dsp::{
    noise_gate::NoiseGate, compressor::Compressor, overdrive::Overdrive,
    distortion::Distortion, fuzz::Fuzz, tremolo::Tremolo,
    chorus::Chorus, delay::Delay, reverb::Reverb,
    chain::EffectsChain,
};
use std::fs;
use std::path::Path;
use std::process::Command;

const SAMPLE_DIR: &str = "../sample_audios";
const OUTPUT_DIR: &str = "../test_output";
const SR: f32 = 44100.0;

/// Convert any audio file to a temporary mono 44100 Hz WAV via ffmpeg.
fn to_wav(input: &Path) -> String {
    let tmp = format!("/tmp/pedaleira_src_{}.wav",
        input.file_stem().unwrap().to_str().unwrap());
    Command::new("ffmpeg")
        .args(["-y", "-i", input.to_str().unwrap(),
               "-ac", "1", "-ar", "44100", "-sample_fmt", "s16",
               &tmp])
        .output()
        .expect("ffmpeg not found — install it with: sudo apt install ffmpeg");
    tmp
}

/// Apply one effect closure to the samples from `wav_path` and save the result.
fn render<F: FnMut(&mut [f32])>(wav: &str, stem: &str, label: &str, mut effect: F) {
    let samples = dsp_wav::load_wav_mono_f32(wav);
    let mut buf = samples;
    effect(&mut buf);
    let out = format!("{}/{}_{}.wav", OUTPUT_DIR, stem, label);
    dsp_wav::save_wav_mono_f32(&out, &buf, SR as u32);
    println!("  → {}", out);
}

/// Process one source file through every effect and the full chain.
fn render_all_for(wav_path: &str, stem: &str) {
    println!("\n  [{stem}]");
    render(wav_path, stem, "1_noise_gate",  |b| NoiseGate::new(0.02).process(b));
    render(wav_path, stem, "2_compressor",  |b| Compressor::new(SR, -18.0, 4.0, 0.01, 0.1).process(b));
    render(wav_path, stem, "3_overdrive",   |b| Overdrive::new(5.0, 0.5).process(b));
    render(wav_path, stem, "4_distortion",  |b| Distortion::new(12.0, 0.6, 44100.0).process(b));
    render(wav_path, stem, "5_fuzz",        |b| Fuzz::new(0.8, 0.7, 44100.0).process(b));
    render(wav_path, stem, "6_tremolo",     |b| Tremolo::new(SR, 5.0, 0.8).process(b));
    render(wav_path, stem, "7_chorus",      |b| Chorus::new(SR, 0.5, 1.5, 0.6).process(b));
    render(wav_path, stem, "8_delay",       |b| Delay::new(SR, 300.0, 0.4, 0.4).process(b));
    render(wav_path, stem, "9_reverb",      |b| Reverb::new(SR, 0.6, 0.4).process(b));
    render(wav_path, stem, "full_chain",    |b| {
        let mut chain = EffectsChain::new(SR);
        for i in 0..9 { chain.set_bypass(i, false); }
        chain.process(b);
    });
}

#[test]
fn render_all_effects() {
    fs::create_dir_all(OUTPUT_DIR).expect("failed to create test_output/");

    let sample_dir = Path::new(SAMPLE_DIR);
    let mut entries: Vec<_> = fs::read_dir(sample_dir)
        .expect("sample_audios/ not found")
        .filter_map(|e| e.ok())
        .filter(|e| {
            matches!(
                e.path().extension().and_then(|x| x.to_str()),
                Some("wav" | "mp3" | "flac" | "ogg" | "aiff" | "aif")
            )
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    assert!(!entries.is_empty(), "no audio files found in {SAMPLE_DIR}");

    println!("\n=== Rendering effects for {} file(s) → {OUTPUT_DIR}/ ===", entries.len());

    for entry in &entries {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let wav_path = if path.extension().and_then(|x| x.to_str()) == Some("wav") {
            path.to_str().unwrap().to_string()
        } else {
            to_wav(&path)
        };
        render_all_for(&wav_path, &stem);
    }

    println!("\nPlay all: mpv {}/*.wav", OUTPUT_DIR);
    println!("=======================================================\n");
}
