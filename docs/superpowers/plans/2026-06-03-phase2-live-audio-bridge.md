# Phase 2 — Live Audio Engine + Bridge API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the existing `EffectsChain` DSP into a live cpal audio loop fed by a WAV file, expose a lock-free SPSC command channel, and surface a minimal flutter_rust_bridge API — all testable in pure Rust without Flutter or audio hardware.

**Architecture:** An `AudioEngine` pure-Rust struct owns `EffectsChain` and a SPSC consumer; `process_block(&mut [f32])` drains commands then processes the buffer. An `EngineHandle` holds the SPSC producer and is the only thing the bridge (and Rust tests) interact with. A `Runtime` wires the engine to cpal + WAV I/O; it is never touched by tests.

**Tech Stack:** Rust, `cpal 0.15`, `ringbuf 0.4`, `hound 3`, `serde_json 1`, `flutter_rust_bridge 2.12.0`

---

## File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Modify | `rust/Cargo.toml` | Move `hound` to `[dependencies]` |
| Create | `rust/src/engine/mod.rs` | `Command` enum, `AudioEngine`, `make_engine` factory |
| Create | `rust/src/engine/handle.rs` | `EngineHandle`, `parse_params` |
| Create | `rust/src/engine/runtime.rs` | `RuntimeConfig`, `Runtime`, `load_wav` |
| Create | `rust/src/api/engine_api.rs` | FRB-exposed bridge functions |
| Modify | `rust/src/api/mod.rs` | `pub mod engine_api` |
| Modify | `rust/src/lib.rs` | `pub mod engine` |

---

## Task 1: Cargo.toml + Module Skeletons

**Files:**
- Modify: `rust/Cargo.toml`
- Modify: `rust/src/lib.rs`
- Modify: `rust/src/api/mod.rs`
- Create: `rust/src/engine/mod.rs`
- Create: `rust/src/engine/handle.rs`
- Create: `rust/src/engine/runtime.rs`
- Create: `rust/src/api/engine_api.rs`

- [ ] **Step 1: Move hound to `[dependencies]` in `rust/Cargo.toml`**

  Replace the `[dev-dependencies]` section:
  ```toml
  [dependencies]
  flutter_rust_bridge = "=2.12.0"
  ringbuf = "0.4"
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  hound = "3"

  [dependencies.cpal]
  version = "0.15"
  features = []

  [dev-dependencies]
  approx = "0.5"
  ```

- [ ] **Step 2: Add `pub mod engine` to `rust/src/lib.rs`**

  ```rust
  pub mod api;
  mod frb_generated;
  pub mod dsp;
  pub mod engine;
  ```

- [ ] **Step 3: Add `pub mod engine_api` to `rust/src/api/mod.rs`**

  ```rust
  pub mod simple;
  pub mod engine_api;
  ```

- [ ] **Step 4: Create `rust/src/engine/mod.rs` stub**

  ```rust
  pub mod handle;
  pub mod runtime;
  ```

- [ ] **Step 5: Create `rust/src/engine/handle.rs` stub**

  ```rust
  // EngineHandle — populated in Task 3
  ```

- [ ] **Step 6: Create `rust/src/engine/runtime.rs` stub**

  ```rust
  // Runtime — populated in Task 4
  ```

- [ ] **Step 7: Create `rust/src/api/engine_api.rs` stub**

  ```rust
  // Bridge API — populated in Task 5
  ```

- [ ] **Step 8: Verify it compiles**

  ```bash
  cd rust && cargo check
  ```
  Expected: no errors (warnings about unused imports are fine).

- [ ] **Step 9: Commit**

  ```bash
  git add rust/Cargo.toml rust/src/lib.rs rust/src/api/mod.rs \
          rust/src/engine/mod.rs rust/src/engine/handle.rs \
          rust/src/engine/runtime.rs rust/src/api/engine_api.rs
  git commit -m "feat: scaffold engine module and api stubs for Phase 2"
  ```

---

## Task 2: `Command` Enum + `AudioEngine`

**Files:**
- Modify: `rust/src/engine/mod.rs`

- [ ] **Step 1: Write the failing tests**

  Replace the contents of `rust/src/engine/mod.rs` with:

  ```rust
  pub mod handle;
  pub mod runtime;

  use ringbuf::{traits::*, HeapRb};
  pub use ringbuf::HeapProd as RbProd;
  pub use ringbuf::HeapCons as RbCons;

  use crate::dsp::chain::{EffectsChain, EffectParams};

  pub enum Command {
      ToggleBypass { slot: u8, bypass: bool },
      SetParam { slot: u8, params: EffectParams },
  }

  pub struct AudioEngine {
      chain: EffectsChain,
      commands: RbCons<Command>,
      sample_rate: f32,
  }

  impl AudioEngine {
      pub fn new(commands: RbCons<Command>, sample_rate: f32) -> Self {
          Self {
              chain: EffectsChain::new(sample_rate),
              commands,
              sample_rate,
          }
      }

      pub fn process_block(&mut self, buf: &mut [f32]) {
          todo!()
      }
  }

  pub fn make_engine(sample_rate: f32) -> (AudioEngine, RbProd<Command>) {
      todo!()
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::dsp::chain::EffectParams;
      use crate::dsp::params::NoiseGateParams;

      fn engine_with_prod() -> (AudioEngine, RbProd<Command>) {
          let rb = HeapRb::<Command>::new(64);
          let (prod, cons) = rb.split();
          (AudioEngine::new(cons, 44100.0), prod)
      }

      #[test]
      fn process_block_passes_through_when_all_bypassed() {
          let (mut engine, _prod) = engine_with_prod();
          let mut buf = vec![0.5f32; 512];
          engine.process_block(&mut buf);
          assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
      }

      #[test]
      fn toggle_bypass_enables_noise_gate_silencing_low_signal() {
          let (mut engine, mut prod) = engine_with_prod();
          // slot 0 = noise gate, default threshold=0.01; signal 0.001 < threshold
          prod.try_push(Command::ToggleBypass { slot: 0, bypass: false }).unwrap();
          let mut buf = vec![0.001f32; 512];
          engine.process_block(&mut buf);
          assert!(buf.iter().all(|&s| s == 0.0));
      }

      #[test]
      fn set_param_updates_noise_gate_threshold() {
          let (mut engine, mut prod) = engine_with_prod();
          prod.try_push(Command::ToggleBypass { slot: 0, bypass: false }).unwrap();
          // Lower threshold to 0.01 — signal 0.5 passes through
          prod.try_push(Command::SetParam {
              slot: 0,
              params: EffectParams::NoiseGate(NoiseGateParams { threshold: 0.01 }),
          }).unwrap();
          let mut buf = vec![0.5f32; 512];
          engine.process_block(&mut buf);
          assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
      }
  }
  ```

- [ ] **Step 2: Run tests — expect failure**

  ```bash
  cd rust && cargo test -q engine::tests 2>&1 | head -20
  ```
  Expected: tests panic at `todo!()`.

- [ ] **Step 3: Implement `process_block` and `make_engine`**

  Replace the two `todo!()` bodies:

  ```rust
  pub fn process_block(&mut self, buf: &mut [f32]) {
      while let Some(cmd) = self.commands.try_pop() {
          match cmd {
              Command::ToggleBypass { slot, bypass } => {
                  self.chain.set_bypass(slot as usize, bypass);
              }
              Command::SetParam { slot: _, params } => {
                  self.chain.apply_params(self.sample_rate, &params);
              }
          }
      }
      self.chain.process(buf);
  }

  pub fn make_engine(sample_rate: f32) -> (AudioEngine, RbProd<Command>) {
      let rb = HeapRb::<Command>::new(64);
      let (prod, cons) = rb.split();
      (AudioEngine::new(cons, sample_rate), prod)
  }
  ```

- [ ] **Step 4: Run tests — expect pass**

  ```bash
  cd rust && cargo test -q engine::tests
  ```
  Expected: `test result: ok. 3 passed; 0 failed`.

- [ ] **Step 5: Commit**

  ```bash
  git add rust/src/engine/mod.rs
  git commit -m "feat: AudioEngine core with Command SPSC drain"
  ```

---

## Task 3: `EngineHandle` + `parse_params`

**Files:**
- Modify: `rust/src/engine/handle.rs`

- [ ] **Step 1: Write the failing tests**

  Replace `rust/src/engine/handle.rs` with:

  ```rust
  use ringbuf::traits::Producer;
  use crate::dsp::chain::EffectParams;
  use crate::dsp::params::*;
  use crate::engine::{Command, RbProd};

  pub struct EngineHandle {
      prod: RbProd<Command>,
  }

  impl EngineHandle {
      pub fn new(prod: RbProd<Command>) -> Self {
          Self { prod }
      }

      pub fn toggle_bypass(&mut self, slot: u8, bypass: bool) {
          let _ = self.prod.try_push(Command::ToggleBypass { slot, bypass });
      }

      pub fn set_param(&mut self, slot: u8, json: &str) -> Result<(), String> {
          todo!()
      }
  }

  fn parse_params(slot: u8, json: &str) -> Result<EffectParams, String> {
      todo!()
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::engine::make_engine;

      #[test]
      fn toggle_bypass_enables_noise_gate() {
          let (mut engine, prod) = make_engine(44100.0);
          let mut handle = EngineHandle::new(prod);
          handle.toggle_bypass(0, false);
          let mut buf = vec![0.001f32; 512];
          engine.process_block(&mut buf);
          assert!(buf.iter().all(|&s| s == 0.0));
      }

      #[test]
      fn set_param_json_lowers_noise_gate_threshold() {
          let (mut engine, prod) = make_engine(44100.0);
          let mut handle = EngineHandle::new(prod);
          handle.toggle_bypass(0, false);
          handle.set_param(0, r#"{"threshold": 0.01}"#).unwrap();
          let mut buf = vec![0.5f32; 512];
          engine.process_block(&mut buf);
          assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
      }

      #[test]
      fn set_param_returns_error_for_unknown_slot() {
          let (_engine, prod) = make_engine(44100.0);
          let mut handle = EngineHandle::new(prod);
          assert!(handle.set_param(9, r#"{}"#).is_err());
      }

      #[test]
      fn set_param_returns_error_for_invalid_json() {
          let (_engine, prod) = make_engine(44100.0);
          let mut handle = EngineHandle::new(prod);
          assert!(handle.set_param(0, r#"not json"#).is_err());
      }

      #[test]
      fn set_param_accepts_all_9_slots() {
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
      }
  }
  ```

- [ ] **Step 2: Run tests — expect failure**

  ```bash
  cd rust && cargo test -q engine::handle::tests 2>&1 | head -20
  ```
  Expected: panic at `todo!()`.

- [ ] **Step 3: Implement `parse_params` and `set_param`**

  Replace the two `todo!()` bodies:

  ```rust
  pub fn set_param(&mut self, slot: u8, json: &str) -> Result<(), String> {
      let params = parse_params(slot, json)?;
      self.prod
          .try_push(Command::SetParam { slot, params })
          .map_err(|_| "command ring full".to_string())
  }

  fn parse_params(slot: u8, json: &str) -> Result<EffectParams, String> {
      match slot {
          0 => serde_json::from_str::<NoiseGateParams>(json).map(EffectParams::NoiseGate),
          1 => serde_json::from_str::<CompressorParams>(json).map(EffectParams::Compressor),
          2 => serde_json::from_str::<OverdriveParams>(json).map(EffectParams::Overdrive),
          3 => serde_json::from_str::<DistortionParams>(json).map(EffectParams::Distortion),
          4 => serde_json::from_str::<FuzzParams>(json).map(EffectParams::Fuzz),
          5 => serde_json::from_str::<ChorusParams>(json).map(EffectParams::Chorus),
          6 => serde_json::from_str::<TremoloParams>(json).map(EffectParams::Tremolo),
          7 => serde_json::from_str::<DelayParams>(json).map(EffectParams::Delay),
          8 => serde_json::from_str::<ReverbParams>(json).map(EffectParams::Reverb),
          _ => Err(format!("unknown slot {slot}")),
      }
      .map_err(|e| e.to_string())
  }
  ```

- [ ] **Step 4: Run tests — expect pass**

  ```bash
  cd rust && cargo test -q engine::handle::tests
  ```
  Expected: `test result: ok. 5 passed; 0 failed`.

- [ ] **Step 5: Run all tests to confirm no regressions**

  ```bash
  cd rust && cargo test -q
  ```
  Expected: all tests pass.

- [ ] **Step 6: Commit**

  ```bash
  git add rust/src/engine/handle.rs
  git commit -m "feat: EngineHandle with JSON parse_params for all 9 slots"
  ```

---

## Task 4: `Runtime` (WAV reader + cpal + file tee)

**Files:**
- Modify: `rust/src/engine/runtime.rs`

No unit tests — Runtime depends on audio hardware and file I/O. Verification is a `cargo check` plus the smoke test in Task 6.

- [ ] **Step 1: Replace `rust/src/engine/runtime.rs` with the full implementation**

  ```rust
  use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
  use std::thread::JoinHandle;
  use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
  use cpal::{Stream, StreamConfig, SampleRate, BufferSize};
  use ringbuf::{traits::*, HeapRb};

  use crate::engine::{AudioEngine, make_engine};
  use crate::engine::handle::EngineHandle;

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
          self.shutdown.store(true, Ordering::Relaxed);
          if let Some(h) = self.reader_handle.take() { let _ = h.join(); }
          if let Some(h) = self.sink_handle.take() { let _ = h.join(); }
      }
  }

  impl Runtime {
      pub fn start(config: RuntimeConfig) -> Result<(Runtime, EngineHandle), String> {
          // 1. Load WAV into memory
          let (wav_samples, sample_rate) = load_wav(&config.wav_path)?;

          // 2. Engine + command channel
          let (engine, cmd_prod) = make_engine(sample_rate as f32);
          let handle = EngineHandle::new(cmd_prod);

          // 3. Sample ring: reader thread → audio callback
          let (mut sample_prod, mut sample_cons) = HeapRb::<f32>::new(4096).split();

          // 4. File sink ring: audio callback → writer thread (optional)
          let (mut sink_prod_opt, sink_cons_opt) = if config.write_output {
              let (p, c) = HeapRb::<f32>::new(4096).split();
              (Some(p), Some(c))
          } else {
              (None, None)
          };

          let shutdown = Arc::new(AtomicBool::new(false));

          // 5. WAV reader thread: loops wav_samples → sample ring
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

          // 6. File sink thread (optional)
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

          // 7. cpal output stream
          let host = cpal::default_host();
          let device = host.default_output_device()
              .ok_or_else(|| "no default output device".to_string())?;
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
                  if !play_output {
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

  /// Load a mono WAV file (any bit depth, float or int) into a Vec<f32>.
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

- [ ] **Step 2: Cargo check**

  ```bash
  cd rust && cargo check
  ```
  Expected: no errors. If you see `` `Stream` cannot be sent between threads safely ``, add this after the `use` block in `runtime.rs`:
  ```rust
  // Safety: cpal's ALSA Stream is safe to send across threads.
  unsafe impl Send for Runtime {}
  ```

- [ ] **Step 3: Run all existing tests to confirm no regressions**

  ```bash
  cd rust && cargo test -q
  ```
  Expected: all tests pass.

- [ ] **Step 4: Commit**

  ```bash
  git add rust/src/engine/runtime.rs
  git commit -m "feat: Runtime wires AudioEngine to cpal output + WAV reader + file tee"
  ```

---

## Task 5: Bridge API

**Files:**
- Modify: `rust/src/api/engine_api.rs`

- [ ] **Step 1: Replace `rust/src/api/engine_api.rs` with the bridge functions**

  ```rust
  use std::sync::Mutex;
  use crate::engine::handle::EngineHandle;
  use crate::engine::runtime::{Runtime, RuntimeConfig};

  static ENGINE: Mutex<Option<(Runtime, EngineHandle)>> = Mutex::new(None);

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
      let config = RuntimeConfig { wav_path, play_output, write_output, output_path };
      let (runtime, handle) = Runtime::start(config)?;
      *guard = Some((runtime, handle));
      Ok(())
  }

  /// Stop the audio engine and release all resources.
  #[flutter_rust_bridge::frb(sync)]
  pub fn stop_engine() {
      if let Ok(mut guard) = ENGINE.lock() {
          *guard = None;
      }
  }

  /// Toggle bypass for a slot (0=noise gate … 8=reverb).
  #[flutter_rust_bridge::frb(sync)]
  pub fn toggle_bypass(slot: u8, bypass: bool) {
      if let Ok(mut guard) = ENGINE.lock() {
          if let Some((_, ref mut handle)) = *guard {
              handle.toggle_bypass(slot, bypass);
          }
      }
  }

  /// Set params for a slot via JSON string (see design doc for schema per slot).
  #[flutter_rust_bridge::frb(sync)]
  pub fn set_param(slot: u8, json: String) -> Result<(), String> {
      let mut guard = ENGINE.lock().map_err(|e| e.to_string())?;
      match guard.as_mut() {
          Some((_, handle)) => handle.set_param(slot, &json),
          None => Err("engine not running".to_string()),
      }
  }
  ```

- [ ] **Step 2: Cargo check**

  ```bash
  cd rust && cargo check
  ```
  Expected: no errors.

- [ ] **Step 3: Run all tests**

  ```bash
  cd rust && cargo test -q
  ```
  Expected: all pass.

- [ ] **Step 4: Regenerate flutter_rust_bridge bindings**

  ```bash
  cd /home/arlan/ai-tests/pedaleira && flutter_rust_bridge_codegen generate
  ```
  Expected: regenerated `rust/src/frb_generated.rs` and Dart bindings in `lib/src/rust/`.

- [ ] **Step 5: Full Flutter build check**

  ```bash
  cd /home/arlan/ai-tests/pedaleira && flutter build linux 2>&1 | tail -10
  ```
  Expected: `Building Linux application... (completed)`.

- [ ] **Step 6: Commit**

  ```bash
  git add rust/src/api/engine_api.rs rust/src/frb_generated.rs \
          lib/src/rust/
  git commit -m "feat: bridge API — start/stop engine, toggle_bypass, set_param"
  ```

---

## Task 6: Smoke Test

Verify the Runtime actually plays audio and writes a file.

- [ ] **Step 1: Convert the sample MP3 to a mono WAV**

  ```bash
  ffmpeg -y -i sample_audios/freesound_community-electric-guitar-riff-jingle-83860.mp3 \
    -ac 1 -ar 44100 -sample_fmt s16 /tmp/pedaleira_test.wav
  ```
  Expected: `/tmp/pedaleira_test.wav` created.

- [ ] **Step 2: Write an integration test in `rust/tests/`**

  Create `rust/tests/engine_runtime.rs`:

  ```rust
  use rust_lib_pedaleira::engine::runtime::{Runtime, RuntimeConfig};
  use std::path::Path;

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

      let (runtime, mut handle) = Runtime::start(config).expect("Runtime::start failed");

      // Enable overdrive (slot 2) and let it run for 2 seconds
      handle.toggle_bypass(2, false);
      handle.set_param(2, r#"{"drive": 5.0, "tone": 0.6}"#).unwrap();
      std::thread::sleep(std::time::Duration::from_secs(2));

      drop(runtime); // stops stream, joins threads, finalizes WAV

      assert!(Path::new(output).exists(), "output WAV not created");
      let meta = std::fs::metadata(output).unwrap();
      assert!(meta.len() > 44, "output WAV appears empty (header only)");
      println!("Output: {output} ({} bytes)", meta.len());
  }
  ```

- [ ] **Step 3: Run the ignored test**

  ```bash
  cd rust && cargo test -- --ignored --nocapture 2>&1 | tail -20
  ```
  Expected: test passes, `Output: /tmp/pedaleira_out.wav (...)` printed.
  If you hear audio and the file exists with > 44 bytes, the runtime is working.

- [ ] **Step 4: Verify output with ffprobe (optional)**

  ```bash
  ffprobe /tmp/pedaleira_out.wav 2>&1 | grep -E "Duration|Stream"
  ```
  Expected: shows ~2 second duration, 44100 Hz mono float WAV.

- [ ] **Step 5: Update `progress.md`**

  Mark the Phase 2 items complete:
  ```markdown
  ## Phase 2 — Live Audio + Bridge API
  - [x] cpal audio I/O (WAV looping input → DSP → output device + file tee)
  - [x] Effects chain struct (8 fixed slots + reverb) — reused from Phase 1
  - [x] Lock-free param sync (ringbuf SPSC Command queue)
  - [x] flutter_rust_bridge API (toggle bypass, set param, start/stop engine)
  - [x] Regenerate bridge bindings
  ```

- [ ] **Step 6: Commit**

  ```bash
  git add rust/tests/engine_runtime.rs progress.md
  git commit -m "test: engine runtime integration test + mark Phase 2 complete"
  ```
