use rust_lib_pedaleira::engine::runtime::{Runtime, RuntimeConfig};
use std::path::Path;
use std::sync::{Arc, atomic::AtomicBool};

#[test]
#[ignore] // requires audio hardware — run with: cargo test -- --ignored
fn runtime_plays_and_writes_wav() {
    let input = "/tmp/pedaleira_test.wav";
    let output = "/tmp/pedaleira_out.wav";
    assert!(Path::new(input).exists(), "run ffmpeg conversion first");

    let config = RuntimeConfig {
        wav_path: input.to_string(),
        play_output: true,
        write_output: true,
        output_path: output.to_string(),
    };

    let muted = Arc::new(AtomicBool::new(false));
    let (runtime, mut handle) = Runtime::start(config, muted).expect("Runtime::start failed");

    // Enable overdrive (slot 2) and let it run for 2 seconds
    handle.toggle_bypass(2, false).unwrap();
    handle.set_param(2, r#"{"drive": 5.0, "tone": 0.6}"#).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));

    drop(runtime); // stops stream, joins threads, finalizes WAV

    assert!(Path::new(output).exists(), "output WAV not created");
    let meta = std::fs::metadata(output).unwrap();
    assert!(meta.len() > 44, "output WAV appears empty (header only)");
    println!("Output: {output} ({} bytes)", meta.len());
}
