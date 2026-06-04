# Boost Pedal + Global Mute Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Clean Boost pedal (slot 9, after Reverb, per-preset) and a global Mute bar (persists across presets) to the pedalboard app.

**Architecture:** Boost is a trivial gain-multiply DSP added as slot 9 in the Rust effects chain; it participates in the existing preset system. Mute is an `AtomicBool` in the Rust audio callback, toggled via a new bridge function `set_mute`; on the Flutter side it lives in a `muteProvider` (Riverpod `NotifierProvider<bool>`) outside the preset system.

**Tech Stack:** Rust (cpal, hound, serde_json), flutter_rust_bridge, Flutter, Riverpod

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `rust/src/dsp/boost.rs` | Create | Boost DSP struct |
| `rust/src/dsp/params.rs` | Modify | Add `BoostParams` |
| `rust/src/dsp/mod.rs` | Modify | Expose `boost` module |
| `rust/src/dsp/chain.rs` | Modify | Add boost slot 9, grow bypass array |
| `rust/src/engine/handle.rs` | Modify | Parse slot 9 → BoostParams |
| `rust/src/engine/runtime.rs` | Modify | Accept `Arc<AtomicBool>` mute flag |
| `rust/src/api/engine_api.rs` | Modify | Add MUTE static + `set_mute` fn |
| `rust/tests/boost_test.rs` | Create | Boost DSP unit tests |
| `rust/tests/chain_test.rs` | Modify | Update slot count; add boost chain test |
| `lib/src/domain/engine_repository.dart` | Modify | Add `setMute(bool)` |
| `lib/src/data/fake_engine_repository.dart` | Modify | Implement `setMute` |
| `lib/src/data/rust_engine_repository.dart` | Modify | Implement `setMute` → bridge |
| `lib/src/domain/models.dart` | Modify | Add `PedalSlot.boost`, defaults, ranges |
| `lib/src/providers/mute_provider.dart` | Create | `muteProvider` + `MuteNotifier` |
| `lib/src/ui/pedalboard/mute_bar.dart` | Create | Mute bar widget |
| `lib/src/ui/pedalboard/pedalboard_screen.dart` | Modify | Bottom row + mute bar + layout math |
| `test/mute_provider_test.dart` | Create | Mute provider unit tests |
| `test/pedalboard_notifier_test.dart` | Modify | Update expected count 9→10 |

---

## Task 1: Rust Boost DSP

**Files:**
- Create: `rust/src/dsp/boost.rs`
- Modify: `rust/src/dsp/params.rs`
- Modify: `rust/src/dsp/mod.rs`
- Create: `rust/tests/boost_test.rs`

- [ ] **Step 1: Write failing Boost DSP tests**

Create `rust/tests/boost_test.rs`:

```rust
use rust_lib_pedaleira::dsp::boost::Boost;

#[test]
fn boost_unity_passes_signal_unchanged() {
    let mut b = Boost::new(1.0);
    let mut buf = vec![0.5f32; 64];
    b.process(&mut buf);
    assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
}

#[test]
fn boost_gain_2_doubles_signal() {
    let mut b = Boost::new(2.0);
    let mut buf = vec![0.25f32; 64];
    b.process(&mut buf);
    assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
}

#[test]
fn boost_gain_0_silences_signal() {
    let mut b = Boost::new(0.0);
    let mut buf = vec![0.5f32; 64];
    b.process(&mut buf);
    assert!(buf.iter().all(|&s| s == 0.0));
}

#[test]
fn boost_set_gain_updates_multiplier() {
    let mut b = Boost::new(1.0);
    b.set_gain(3.0);
    let mut buf = vec![0.1f32; 64];
    b.process(&mut buf);
    assert!(buf.iter().all(|&s| (s - 0.3).abs() < 1e-6));
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test --manifest-path rust/Cargo.toml --test boost_test 2>&1 | tail -10
```

Expected: compile error — `boost` module not found.

- [ ] **Step 3: Implement boost.rs**

Create `rust/src/dsp/boost.rs`:

```rust
pub struct Boost {
    gain: f32,
}

impl Boost {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s *= self.gain;
        }
    }
}
```

- [ ] **Step 4: Add BoostParams to params.rs**

Append to `rust/src/dsp/params.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostParams { pub gain: f32 }
```

- [ ] **Step 5: Expose boost module in dsp/mod.rs**

Add `pub mod boost;` to `rust/src/dsp/mod.rs`:

```rust
pub mod noise_gate;
pub mod compressor;
pub mod overdrive;
pub mod distortion;
pub mod fuzz;
pub mod tremolo;
pub mod chorus;
pub mod delay;
pub mod reverb;
pub mod boost;
pub mod chain;
pub mod params;
```

- [ ] **Step 6: Run tests to confirm they pass**

```bash
cargo test --manifest-path rust/Cargo.toml --test boost_test 2>&1 | tail -10
```

Expected: 4 tests pass.

- [ ] **Step 7: Commit**

```bash
git add rust/src/dsp/boost.rs rust/src/dsp/params.rs rust/src/dsp/mod.rs rust/tests/boost_test.rs
git commit -m "feat(rust): add Boost DSP and BoostParams"
```

---

## Task 2: Wire Boost into Effects Chain

**Files:**
- Modify: `rust/src/dsp/chain.rs`
- Modify: `rust/src/engine/handle.rs`
- Modify: `rust/tests/chain_test.rs`

- [ ] **Step 1: Add boost chain tests**

Add to the bottom of `rust/tests/chain_test.rs`:

```rust
#[test]
fn boost_at_gain_2_doubles_signal() {
    let mut chain = EffectsChain::new(44100.0);
    chain.set_bypass(9, false);
    chain.apply_params(44100.0, &EffectParams::Boost(BoostParams { gain: 2.0 }));
    let mut buf = vec![0.25f32; 512];
    chain.process(&mut buf);
    assert!(
        buf.iter().all(|&s| (s - 0.5).abs() < 1e-6),
        "boost x2 should double 0.25 to 0.5"
    );
}
```

Also update the two existing tests that loop `0..9` and `for i in 0..9` to `0..10`:

```rust
// chain_processes_without_panic — change:
for i in 0..10 { chain.set_bypass(i, false); }

// chain_wav_roundtrip_full — change:
for i in 0..10 { chain.set_bypass(i, false); }
```

- [ ] **Step 2: Run tests to confirm the new test fails**

```bash
cargo test --manifest-path rust/Cargo.toml --test chain_test 2>&1 | tail -15
```

Expected: `boost_at_gain_2_doubles_signal` fails (slot 9 unknown), existing tests compile.

- [ ] **Step 3: Update chain.rs**

Full replacement of `rust/src/dsp/chain.rs`:

```rust
use crate::dsp::{
    noise_gate::NoiseGate,
    compressor::Compressor,
    overdrive::Overdrive,
    distortion::Distortion,
    fuzz::Fuzz,
    chorus::Chorus,
    tremolo::Tremolo,
    delay::Delay,
    reverb::Reverb,
    boost::Boost,
};
use crate::dsp::params::*;

pub struct EffectsChain {
    noise_gate:  NoiseGate,
    compressor:  Compressor,
    overdrive:   Overdrive,
    distortion:  Distortion,
    fuzz:        Fuzz,
    chorus:      Chorus,
    tremolo:     Tremolo,
    delay:       Delay,
    reverb:      Reverb,
    boost:       Boost,
    bypass: [bool; 10], // indices 0–9: noise_gate..boost
}

impl EffectsChain {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            noise_gate:  NoiseGate::new(0.01),
            compressor:  Compressor::new(sample_rate, -18.0, 4.0, 0.01, 0.1),
            overdrive:   Overdrive::new(3.0, 0.5),
            distortion:  Distortion::new(8.0, 0.5),
            fuzz:        Fuzz::new(0.7, 0.7),
            chorus:      Chorus::new(sample_rate, 0.5, 1.5, 0.5),
            tremolo:     Tremolo::new(sample_rate, 4.0, 0.5),
            delay:       Delay::new(sample_rate, 300.0, 0.4, 0.4),
            reverb:      Reverb::new(sample_rate, 0.5, 0.3),
            boost:       Boost::new(1.0),
            bypass:      [true; 10],
        }
    }

    /// slot 0=noise_gate, 1=compressor, 2=overdrive, 3=distortion,
    /// 4=fuzz, 5=chorus, 6=tremolo, 7=delay, 8=reverb, 9=boost
    pub fn set_bypass(&mut self, slot: usize, bypass: bool) {
        if slot < 10 { self.bypass[slot] = bypass; }
    }

    pub fn is_bypassed(&self, slot: usize) -> bool {
        slot >= 10 || self.bypass[slot]
    }

    pub fn apply_params(&mut self, sample_rate: f32, params: &EffectParams) {
        match params {
            EffectParams::NoiseGate(p)  => self.noise_gate.set_threshold(p.threshold),
            EffectParams::Compressor(p) => {
                self.compressor.set_threshold_db(p.threshold_db);
                self.compressor.set_ratio(p.ratio);
                self.compressor.set_attack(sample_rate, p.attack);
                self.compressor.set_release(sample_rate, p.release);
            }
            EffectParams::Overdrive(p)  => {
                self.overdrive.set_drive(p.drive);
                self.overdrive.set_tone(p.tone);
            }
            EffectParams::Distortion(p) => {
                self.distortion.set_drive(p.drive);
                self.distortion.set_level(p.level);
            }
            EffectParams::Fuzz(p)       => {
                self.fuzz.set_fuzz(p.fuzz);
                self.fuzz.set_level(p.level);
            }
            EffectParams::Chorus(p)     => {
                self.chorus.set_rate(sample_rate, p.rate);
                self.chorus.set_mix(p.mix);
            }
            EffectParams::Tremolo(p)    => {
                self.tremolo.set_rate(sample_rate, p.rate);
                self.tremolo.set_depth(p.depth);
            }
            EffectParams::Delay(p)      => {
                self.delay.set_delay_ms(sample_rate, p.time_ms);
                self.delay.set_feedback(p.feedback);
                self.delay.set_mix(p.mix);
            }
            EffectParams::Reverb(p)     => {
                self.reverb.set_room_size(p.room_size);
                self.reverb.set_mix(p.mix);
            }
            EffectParams::Boost(p)      => self.boost.set_gain(p.gain),
        }
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        if !self.bypass[0] { self.noise_gate.process(buffer); }
        if !self.bypass[1] { self.compressor.process(buffer); }
        if !self.bypass[2] { self.overdrive.process(buffer); }
        if !self.bypass[3] { self.distortion.process(buffer); }
        if !self.bypass[4] { self.fuzz.process(buffer); }
        if !self.bypass[5] { self.chorus.process(buffer); }
        if !self.bypass[6] { self.tremolo.process(buffer); }
        if !self.bypass[7] { self.delay.process(buffer); }
        if !self.bypass[8] { self.reverb.process(buffer); }
        if !self.bypass[9] { self.boost.process(buffer); }
    }
}

#[derive(Debug, Clone)]
pub enum EffectParams {
    NoiseGate(NoiseGateParams),
    Compressor(CompressorParams),
    Overdrive(OverdriveParams),
    Distortion(DistortionParams),
    Fuzz(FuzzParams),
    Chorus(ChorusParams),
    Tremolo(TremoloParams),
    Delay(DelayParams),
    Reverb(ReverbParams),
    Boost(BoostParams),
}
```

- [ ] **Step 4: Update handle.rs to parse slot 9**

In `rust/src/engine/handle.rs`, replace the `parse_params` function and update the `set_param_accepts_all_9_slots` test name and content:

```rust
fn parse_params(slot: u8, json: &str) -> Result<EffectParams, String> {
    match slot {
        0 => serde_json::from_str::<NoiseGateParams>(json)
            .map(EffectParams::NoiseGate)
            .map_err(|e| e.to_string()),
        1 => serde_json::from_str::<CompressorParams>(json)
            .map(EffectParams::Compressor)
            .map_err(|e| e.to_string()),
        2 => serde_json::from_str::<OverdriveParams>(json)
            .map(EffectParams::Overdrive)
            .map_err(|e| e.to_string()),
        3 => serde_json::from_str::<DistortionParams>(json)
            .map(EffectParams::Distortion)
            .map_err(|e| e.to_string()),
        4 => serde_json::from_str::<FuzzParams>(json)
            .map(EffectParams::Fuzz)
            .map_err(|e| e.to_string()),
        5 => serde_json::from_str::<ChorusParams>(json)
            .map(EffectParams::Chorus)
            .map_err(|e| e.to_string()),
        6 => serde_json::from_str::<TremoloParams>(json)
            .map(EffectParams::Tremolo)
            .map_err(|e| e.to_string()),
        7 => serde_json::from_str::<DelayParams>(json)
            .map(EffectParams::Delay)
            .map_err(|e| e.to_string()),
        8 => serde_json::from_str::<ReverbParams>(json)
            .map(EffectParams::Reverb)
            .map_err(|e| e.to_string()),
        9 => serde_json::from_str::<BoostParams>(json)
            .map(EffectParams::Boost)
            .map_err(|e| e.to_string()),
        _ => Err(format!("unknown slot {slot}")),
    }
}
```

Also update the test in `handle.rs` (rename from `set_param_accepts_all_9_slots` → `set_param_accepts_all_10_slots` and add slot 9):

```rust
#[test]
fn set_param_accepts_all_10_slots() {
    let (_engine, prod) = make_engine(44100.0);
    let mut handle = EngineHandle::new(prod);
    assert!(handle.set_param(0, r#"{"threshold": 0.01}"#).is_ok());
    assert!(handle.set_param(1, r#"{"threshold_db": -18.0, "ratio": 4.0, "attack": 0.01, "release": 0.1}"#).is_ok());
    assert!(handle.set_param(2, r#"{"drive": 3.0, "tone": 0.5}"#).is_ok());
    assert!(handle.set_param(3, r#"{"drive": 8.0, "level": 0.5}"#).is_ok());
    assert!(handle.set_param(4, r#"{"fuzz": 0.7, "level": 0.7}"#).is_ok());
    assert!(handle.set_param(5, r#"{"rate": 0.5, "depth": 1.5, "mix": 0.5}"#).is_ok());
    assert!(handle.set_param(6, r#"{"rate": 4.0, "depth": 0.5}"#).is_ok());
    assert!(handle.set_param(7, r#"{"time_ms": 300.0, "feedback": 0.4, "mix": 0.4}"#).is_ok());
    assert!(handle.set_param(8, r#"{"room_size": 0.5, "mix": 0.3}"#).is_ok());
    assert!(handle.set_param(9, r#"{"gain": 1.0}"#).is_ok());
}

#[test]
fn set_param_returns_error_for_unknown_slot() {
    let (_engine, prod) = make_engine(44100.0);
    let mut handle = EngineHandle::new(prod);
    assert!(handle.set_param(10, r#"{}"#).is_err());
}
```

- [ ] **Step 5: Run all Rust tests**

```bash
cargo test --manifest-path rust/Cargo.toml 2>&1 | tail -15
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add rust/src/dsp/chain.rs rust/src/engine/handle.rs rust/tests/chain_test.rs
git commit -m "feat(rust): wire Boost as slot 9 in effects chain"
```

---

## Task 3: Rust Global Mute

**Files:**
- Modify: `rust/src/engine/runtime.rs`
- Modify: `rust/src/api/engine_api.rs`

- [ ] **Step 1: Update runtime.rs to accept mute flag**

Full replacement of `rust/src/engine/runtime.rs`:

```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::JoinHandle;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig, SampleRate, BufferSize};
use ringbuf::{traits::*, HeapRb};

use crate::engine::make_engine;
use crate::engine::handle::EngineHandle;

#[cfg(target_os = "linux")]
unsafe impl Send for Runtime {}

pub struct RuntimeConfig {
    pub wav_path: String,
    pub play_output: bool,
    pub write_output: bool,
    pub output_path: String,
}

pub struct Runtime {
    _stream: Stream,
    reader_handle: Option<JoinHandle<()>>,
    sink_handle: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = StreamTrait::pause(&self._stream);
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(h) = self.reader_handle.take() { let _ = h.join(); }
        if let Some(h) = self.sink_handle.take() { let _ = h.join(); }
    }
}

impl Runtime {
    pub fn start(
        config: RuntimeConfig,
        muted: Arc<AtomicBool>,
    ) -> Result<(Runtime, EngineHandle), String> {
        let (wav_samples, sample_rate) = load_wav(&config.wav_path)?;

        let (engine, cmd_prod) = make_engine(sample_rate as f32);
        let handle = EngineHandle::new(cmd_prod);

        let (mut sample_prod, mut sample_cons) = HeapRb::<f32>::new(4096).split();

        let (mut sink_prod_opt, sink_cons_opt) = if config.write_output {
            let (p, c) = HeapRb::<f32>::new(4096).split();
            (Some(p), Some(c))
        } else {
            (None, None)
        };

        let shutdown = Arc::new(AtomicBool::new(false));

        let shutdown_reader = shutdown.clone();
        let reader_handle = std::thread::spawn(move || {
            let len = wav_samples.len();
            let mut idx = 0usize;
            loop {
                if shutdown_reader.load(Ordering::Relaxed) { break; }
                let s = wav_samples[idx];
                idx = (idx + 1) % len;
                loop {
                    if shutdown_reader.load(Ordering::Relaxed) { return; }
                    if sample_prod.try_push(s).is_ok() { break; }
                    std::thread::sleep(std::time::Duration::from_micros(100));
                }
            }
        });

        let sink_handle = if config.write_output {
            let mut cons = sink_cons_opt.unwrap();
            let shutdown_sink = shutdown.clone();
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            };
            let mut writer = hound::WavWriter::create(&config.output_path, spec)
                .map_err(|e| e.to_string())?;
            Some(std::thread::spawn(move || {
                loop {
                    let done = shutdown_sink.load(Ordering::Relaxed);
                    while let Some(s) = cons.try_pop() {
                        writer.write_sample(s).ok();
                    }
                    if done && cons.is_empty() { break; }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                writer.finalize().ok();
            }))
        } else {
            None
        };

        let host = cpal::default_host();
        let device = host.default_output_device()
            .or_else(|| host.output_devices().ok()?.next())
            .ok_or_else(|| "no output device found".to_string())?;

        eprintln!("[engine] using output device: {}", device.name().unwrap_or_else(|_| "<unknown>".into()));

        let stream_config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(sample_rate),
            buffer_size: BufferSize::Default,
        };
        let play_output = config.play_output;
        let mut engine = engine;

        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _| {
                for s in data.iter_mut() {
                    *s = sample_cons.try_pop().unwrap_or(0.0);
                }
                engine.process_block(data);
                if let Some(ref mut sp) = sink_prod_opt {
                    for &s in data.iter() { let _ = sp.try_push(s); }
                }
                if !play_output || muted.load(Ordering::Relaxed) {
                    for s in data.iter_mut() { *s = 0.0; }
                }
            },
            |err| eprintln!("cpal stream error: {err}"),
            None,
        ).map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;

        Ok((
            Runtime { _stream: stream, reader_handle: Some(reader_handle), sink_handle, shutdown },
            handle,
        ))
    }
}

fn load_wav(path: &str) -> Result<(Vec<f32>, u32), String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    if spec.channels != 1 {
        return Err(format!("only mono WAV supported (got {} channels); convert with: ffmpeg -i input.mp3 -ac 1 -ar 44100 -sample_fmt s16 output.wav", spec.channels));
    }
    let scale = (1i64 << (spec.bits_per_sample - 1)) as f32;
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.map_err(|e| e.to_string()))
            .collect::<Result<Vec<f32>, String>>()?,
        hound::SampleFormat::Int => reader
            .samples::<i32>()
            .map(|s| s.map(|v| v as f32 / scale).map_err(|e| e.to_string()))
            .collect::<Result<Vec<f32>, String>>()?,
    };
    Ok((samples, spec.sample_rate))
}
```

- [ ] **Step 2: Update engine_api.rs — add MUTE static and set_mute**

Full replacement of `rust/src/api/engine_api.rs`:

```rust
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use crate::engine::handle::EngineHandle;
use crate::engine::runtime::{Runtime, RuntimeConfig};

static ENGINE: Mutex<Option<(Runtime, EngineHandle)>> = Mutex::new(None);
static MUTE: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

/// Start the audio engine. Call stop_engine first if already running.
#[flutter_rust_bridge::frb(sync)]
pub fn start_engine(
    wav_path: String,
    play_output: bool,
    write_output: bool,
    output_path: String,
) -> Result<(), String> {
    let mut guard = ENGINE.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Err("engine already running; call stop_engine first".to_string());
    }
    let muted = Arc::new(AtomicBool::new(false));
    {
        let mut m = MUTE.lock().map_err(|e| e.to_string())?;
        *m = Some(muted.clone());
    }
    let config = RuntimeConfig { wav_path, play_output, write_output, output_path };
    let (runtime, handle) = Runtime::start(config, muted)?;
    *guard = Some((runtime, handle));
    Ok(())
}

/// Stop the audio engine and release all resources.
#[flutter_rust_bridge::frb(sync)]
pub fn stop_engine() {
    if let Ok(mut guard) = ENGINE.lock() {
        *guard = None;
    }
    if let Ok(mut m) = MUTE.lock() {
        *m = None;
    }
}

/// Toggle bypass for a slot (0=noise gate … 9=boost).
#[flutter_rust_bridge::frb(sync)]
pub fn toggle_bypass(slot: u8, bypass: bool) -> Result<(), String> {
    let mut guard = ENGINE.lock().map_err(|e| e.to_string())?;
    match guard.as_mut() {
        Some((_, handle)) => handle.toggle_bypass(slot, bypass),
        None => Err("engine not running".to_string()),
    }
}

/// Set params for a slot via JSON string.
#[flutter_rust_bridge::frb(sync)]
pub fn set_param(slot: u8, json: String) -> Result<(), String> {
    let mut guard = ENGINE.lock().map_err(|e| e.to_string())?;
    match guard.as_mut() {
        Some((_, handle)) => handle.set_param(slot, &json),
        None => Err("engine not running".to_string()),
    }
}

/// Mute or unmute the audio output. Persists until changed; not part of preset state.
#[flutter_rust_bridge::frb(sync)]
pub fn set_mute(muted: bool) -> Result<(), String> {
    let guard = MUTE.lock().map_err(|e| e.to_string())?;
    match guard.as_ref() {
        Some(flag) => {
            flag.store(muted, Ordering::Relaxed);
            Ok(())
        }
        None => Err("engine not running".to_string()),
    }
}
```

- [ ] **Step 3: Run all Rust tests**

```bash
cargo test --manifest-path rust/Cargo.toml 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add rust/src/engine/runtime.rs rust/src/api/engine_api.rs
git commit -m "feat(rust): add AtomicBool mute flag and set_mute bridge function"
```

---

## Task 4: Run Codegen

**Files:**
- Modify: `lib/src/rust/api/engine_api.dart` (auto-generated)
- Modify: `lib/src/rust/frb_generated.dart` (auto-generated)
- Modify: `rust/src/frb_generated.rs` (auto-generated)

- [ ] **Step 1: Run codegen**

```bash
flutter_rust_bridge_codegen generate
```

Expected: Done! — no errors.

- [ ] **Step 2: Verify set_mute appears in generated Dart**

```bash
grep -n "set_mute\|setMute" lib/src/rust/api/engine_api.dart
```

Expected: a `setMute` function appears.

- [ ] **Step 3: Commit generated files**

```bash
git add lib/src/rust/ rust/src/frb_generated.rs
git commit -m "chore: regenerate flutter_rust_bridge after set_mute addition"
```

---

## Task 5: Flutter Engine Interface + Implementations

**Files:**
- Modify: `lib/src/domain/engine_repository.dart`
- Modify: `lib/src/data/fake_engine_repository.dart`
- Modify: `lib/src/data/rust_engine_repository.dart`
- Modify: `test/pedalboard_notifier_test.dart`

- [ ] **Step 1: Add setMute to engine_repository.dart**

Full replacement of `lib/src/domain/engine_repository.dart`:

```dart
abstract class EngineRepository {
  void start(String wavPath);
  void stop();
  void toggleBypass(int slot, bool bypassed);
  void setParam(int slot, String json);
  void setMute(bool muted);
}
```

- [ ] **Step 2: Implement setMute in fake_engine_repository.dart**

Full replacement of `lib/src/data/fake_engine_repository.dart`:

```dart
import '../domain/engine_repository.dart';

class FakeEngineRepository implements EngineRepository {
  final List<String> calls = [];

  @override
  void start(String wavPath) => calls.add('start:$wavPath');

  @override
  void stop() => calls.add('stop');

  @override
  void toggleBypass(int slot, bool bypassed) =>
      calls.add('toggle:$slot:$bypassed');

  @override
  void setParam(int slot, String json) => calls.add('setParam:$slot:$json');

  @override
  void setMute(bool muted) => calls.add('setMute:$muted');
}
```

- [ ] **Step 3: Implement setMute in rust_engine_repository.dart**

Full replacement of `lib/src/data/rust_engine_repository.dart`:

```dart
import '../domain/engine_repository.dart';
import '../rust/api/engine_api.dart' as bridge;

class RustEngineRepository implements EngineRepository {
  @override
  void start(String wavPath) {
    bridge.startEngine(
      wavPath: wavPath,
      playOutput: true,
      writeOutput: false,
      outputPath: '',
    );
  }

  @override
  void stop() => bridge.stopEngine();

  @override
  void toggleBypass(int slot, bool bypassed) =>
      bridge.toggleBypass(slot: slot, bypass: bypassed);

  @override
  void setParam(int slot, String json) =>
      bridge.setParam(slot: slot, json: json);

  @override
  void setMute(bool muted) => bridge.setMute(muted: muted);
}
```

- [ ] **Step 4: Update pedalboard_notifier_test.dart count 9→10**

In `test/pedalboard_notifier_test.dart`, update:

```dart
// Change:
test('initial state has 9 pedals all bypassed with default params', () {
  final container = makeContainer();
  final state = container.read(pedalboardProvider);
  expect(state.length, 9);
// To:
test('initial state has 10 pedals all bypassed with default params', () {
  final container = makeContainer();
  final state = container.read(pedalboardProvider);
  expect(state.length, 10);
```

Also update `applyPreset` test (toggle and setParam call counts 9→10):

```dart
// Change:
expect(engine.calls.where((c) => c.startsWith('toggle:')).length, 9);
expect(engine.calls.where((c) => c.startsWith('setParam:')).length, 9);
// To:
expect(engine.calls.where((c) => c.startsWith('toggle:')).length, 10);
expect(engine.calls.where((c) => c.startsWith('setParam:')).length, 10);
```

- [ ] **Step 5: Run Flutter tests**

```bash
flutter test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add lib/src/domain/engine_repository.dart lib/src/data/fake_engine_repository.dart lib/src/data/rust_engine_repository.dart test/pedalboard_notifier_test.dart
git commit -m "feat(flutter): add setMute to EngineRepository interface and implementations"
```

---

## Task 6: Flutter Models — Add Boost Slot

**Files:**
- Modify: `lib/src/domain/models.dart`

- [ ] **Step 1: Update models.dart**

In `lib/src/domain/models.dart`, make the following additions:

Add `boost` to `PedalSlot` enum (after `reverb`):

```dart
enum PedalSlot {
  noiseGate,
  compressor,
  overdrive,
  distortion,
  fuzz,
  chorus,
  tremolo,
  delay,
  reverb,
  boost,
}
```

Add to `kPedalNames`:

```dart
const Map<PedalSlot, String> kPedalNames = {
  PedalSlot.noiseGate: 'Noise Gate',
  PedalSlot.compressor: 'Compressor',
  PedalSlot.overdrive: 'Overdrive',
  PedalSlot.distortion: 'Distortion',
  PedalSlot.fuzz: 'Fuzz',
  PedalSlot.chorus: 'Chorus',
  PedalSlot.tremolo: 'Tremolo',
  PedalSlot.delay: 'Delay',
  PedalSlot.reverb: 'Reverb',
  PedalSlot.boost: 'Boost',
};
```

Add to `kDefaultParams`:

```dart
PedalSlot.boost: {'gain': 1.0},
```

Add to `kParamRanges`:

```dart
'gain': (0.0, 4.0),
```

- [ ] **Step 2: Run Flutter tests**

```bash
flutter test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add lib/src/domain/models.dart
git commit -m "feat(flutter): add PedalSlot.boost to models"
```

---

## Task 7: Flutter Mute Provider

**Files:**
- Create: `lib/src/providers/mute_provider.dart`
- Create: `test/mute_provider_test.dart`

- [ ] **Step 1: Write failing mute provider tests**

Create `test/mute_provider_test.dart`:

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/mute_provider.dart';

ProviderContainer makeContainer({FakeEngineRepository? engine}) {
  final fake = engine ?? FakeEngineRepository();
  final container = ProviderContainer(overrides: [
    engineRepositoryProvider.overrideWithValue(fake),
    presetRepositoryProvider.overrideWithValue(MemoryPresetRepository()),
  ]);
  addTearDown(container.dispose);
  return container;
}

void main() {
  test('muteProvider initial state is false', () {
    final container = makeContainer();
    expect(container.read(muteProvider), isFalse);
  });

  test('toggle sets muted=true and calls engine setMute(true)', () {
    final engine = FakeEngineRepository();
    final container = makeContainer(engine: engine);
    container.read(muteProvider.notifier).toggle();
    expect(container.read(muteProvider), isTrue);
    expect(engine.calls, contains('setMute:true'));
  });

  test('toggle twice restores unmuted and calls setMute(false)', () {
    final engine = FakeEngineRepository();
    final container = makeContainer(engine: engine);
    container.read(muteProvider.notifier).toggle();
    container.read(muteProvider.notifier).toggle();
    expect(container.read(muteProvider), isFalse);
    expect(engine.calls, contains('setMute:false'));
  });
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
flutter test test/mute_provider_test.dart 2>&1 | tail -5
```

Expected: compile error — `muteProvider` not found.

- [ ] **Step 3: Implement mute_provider.dart**

Create `lib/src/providers/mute_provider.dart`:

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'engine_provider.dart';

class MuteNotifier extends Notifier<bool> {
  @override
  bool build() => false;

  void toggle() {
    state = !state;
    ref.read(engineRepositoryProvider).setMute(state);
  }
}

final muteProvider = NotifierProvider<MuteNotifier, bool>(MuteNotifier.new);
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
flutter test test/mute_provider_test.dart 2>&1 | tail -5
```

Expected: 3 tests pass.

- [ ] **Step 5: Run full suite**

```bash
flutter test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add lib/src/providers/mute_provider.dart test/mute_provider_test.dart
git commit -m "feat(flutter): add MuteNotifier and muteProvider"
```

---

## Task 8: Flutter UI — Bottom Row + Mute Bar

**Files:**
- Create: `lib/src/ui/pedalboard/mute_bar.dart`
- Modify: `lib/src/ui/pedalboard/pedalboard_screen.dart`

- [ ] **Step 1: Create mute_bar.dart**

Create `lib/src/ui/pedalboard/mute_bar.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/mute_provider.dart';

class MuteBar extends ConsumerWidget {
  const MuteBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final muted = ref.watch(muteProvider);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      child: GestureDetector(
        onTap: () => ref.read(muteProvider.notifier).toggle(),
        child: Container(
          height: 56,
          decoration: BoxDecoration(
            color: muted ? Colors.red.shade900 : theme.colorScheme.surface,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: muted ? Colors.red : Colors.grey.shade800,
              width: muted ? 1.5 : 1,
            ),
          ),
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                children: [
                  Icon(
                    muted ? Icons.volume_off : Icons.volume_up,
                    color: muted ? Colors.red : Colors.grey.shade500,
                    size: 18,
                  ),
                  const SizedBox(width: 8),
                  Text(
                    muted ? 'MUTED' : 'Output',
                    style: theme.textTheme.labelMedium?.copyWith(
                      color: muted ? Colors.red : Colors.grey.shade500,
                      fontWeight: FontWeight.bold,
                      letterSpacing: 0.5,
                    ),
                  ),
                ],
              ),
              Switch(
                value: !muted,
                onChanged: (_) => ref.read(muteProvider.notifier).toggle(),
                activeColor: theme.colorScheme.primary,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
```

- [ ] **Step 2: Update pedalboard_screen.dart**

Full replacement of `lib/src/ui/pedalboard/pedalboard_screen.dart`:

```dart
import 'package:flutter/material.dart';
import 'pedal_tile.dart';
import 'mute_bar.dart';
import '../preset_bar.dart';

class PedalboardScreen extends StatelessWidget {
  const PedalboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Pedaleira'),
        bottom: const PreferredSize(
          preferredSize: Size.fromHeight(48),
          child: PresetBar(),
        ),
      ),
      body: OrientationBuilder(
        builder: (context, orientation) {
          final crossAxisCount = orientation == Orientation.portrait ? 2 : 4;
          final rowCount = orientation == Orientation.portrait ? 4 : 2;
          return LayoutBuilder(
            builder: (context, constraints) {
              const reverbRowHeight = 72.0;
              const reverbRowBottomPad = 8.0;
              const muteBarHeight = 56.0;
              const muteBarBottomPad = 16.0;
              const gridPadTop = 16.0;
              const gridPadBottom = 8.0;
              const tileSpacing = 12.0;
              final gridContentHeight = constraints.maxHeight
                  - reverbRowHeight
                  - reverbRowBottomPad
                  - muteBarHeight
                  - muteBarBottomPad
                  - gridPadTop
                  - gridPadBottom;
              final tileHeight =
                  (gridContentHeight - tileSpacing * (rowCount - 1)) / rowCount;
              return Column(
                children: [
                  Expanded(
                    child: GridView.builder(
                      padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                      physics: const NeverScrollableScrollPhysics(),
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: crossAxisCount,
                        crossAxisSpacing: 12,
                        mainAxisSpacing: 12,
                        mainAxisExtent: tileHeight,
                      ),
                      itemCount: 8,
                      itemBuilder: (_, i) => PedalTile(slot: i),
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 0, 16, reverbRowBottomPad),
                    child: SizedBox(
                      height: reverbRowHeight,
                      child: Row(
                        children: [
                          Expanded(child: PedalTile(slot: 9)), // Boost
                          const SizedBox(width: 12),
                          Expanded(child: PedalTile(slot: 8)), // Reverb
                        ],
                      ),
                    ),
                  ),
                  const MuteBar(),
                ],
              );
            },
          );
        },
      ),
    );
  }
}
```

- [ ] **Step 3: Run all Flutter tests**

```bash
flutter test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add lib/src/ui/pedalboard/mute_bar.dart lib/src/ui/pedalboard/pedalboard_screen.dart
git commit -m "feat(flutter): boost+reverb bottom row and global mute bar"
```

---

## Final Check

- [ ] **Run full Rust test suite**

```bash
cargo test --manifest-path rust/Cargo.toml 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Run full Flutter test suite**

```bash
flutter test 2>&1 | tail -5
```

Expected: all tests pass (37+ tests).
