# Phase 2 Design — Live Audio + Bridge API

## Overview

Wire the existing `EffectsChain` DSP (Phase 1) into a live audio loop driven by a WAV file, expose a lock-free command channel for parameter control, and surface a minimal flutter_rust_bridge API. The engine core is a pure-Rust struct testable without `cpal` or Flutter.

---

## Components and File Layout

```
rust/src/
  engine/
    mod.rs        — AudioEngine + Command enum
    handle.rs     — EngineHandle (SPSC producer; used by bridge and Rust tests)
    runtime.rs    — Runtime (cpal stream, WAV reader thread, output tee thread)
  api/
    simple.rs     — existing greet/init_app (unchanged)
    engine_api.rs — FRB-exposed: start_engine, stop_engine, toggle_bypass, set_param
    mod.rs        — pub mod simple; pub mod engine_api;
```

### AudioEngine (`engine/mod.rs`)

Owns `EffectsChain` and the SPSC command consumer. Single public method:

```rust
pub fn process_block(&mut self, buf: &mut [f32])
```

Drains all pending `Command`s, then processes the buffer through the chain. No I/O, no threads, no heap allocation in the hot path. Fully testable with a `&mut [f32]`.

### Command enum (`engine/mod.rs`)

Stack-only — no heap allocation on either thread:

```rust
pub enum Command {
    ToggleBypass { slot: u8, bypass: bool },
    SetParam     { slot: u8, params: EffectParams },
}
```

JSON parsing happens in `EngineHandle::set_param()` on the caller's thread. The audio thread receives only typed data.

### EngineHandle (`engine/handle.rs`)

Holds the SPSC producer (capacity 64). Used by both Rust tests and the bridge — tests never need `Runtime` or `cpal`.

```rust
pub fn toggle_bypass(&self, slot: u8, bypass: bool)
pub fn set_param(&self, slot: u8, json: &str) -> Result<(), String>
```

If the command ring is full, the push is silently dropped (acceptable for UI-rate knob updates).

### Runtime (`engine/runtime.rs`)

Wires the engine to I/O. Owns:
- `cpal::Stream`
- WAV reader `JoinHandle`
- File sink `JoinHandle`
- `Arc<AtomicBool>` shutdown flag

Configuration:

```rust
pub struct RuntimeConfig {
    pub wav_path:     String,
    pub play_output:  bool,
    pub write_output: bool,
    pub output_path:  String,  // ignored if write_output = false
}
```

Dropping `Runtime` stops the stream and signals threads to exit.

---

## Data Flow and Threading

```
[WAV reader thread]
  hound::WavReader → f32 samples → ringbuf::Producer<f32>  (cap: 4096)
  EOF → seek to start (loops indefinitely)

[cpal audio callback — real-time]
  ringbuf::Consumer<f32>    → fill block buffer
  ringbuf::Consumer<Command>→ drain → engine.apply(cmd)
  engine.process_block(&mut block)
  → play_output:  write to cpal output buffer
  → write_output: ringbuf::Producer<f32> (cap: 4096, file sink ring)

[file sink thread]
  ringbuf::Consumer<f32> → hound::WavWriter → output .wav
```

- **Sample rate:** read from WAV header once at startup; passed to `EffectsChain::new(sample_rate)`.
- **Block size:** whatever `cpal` requests per callback (typically 256–1024 samples).
- **WAV looping:** reader thread seeks to start on EOF; audio runs indefinitely.
- **Shutdown:** `Arc<AtomicBool>` signals reader and sink threads; dropping `cpal::Stream` stops the callback.

---

## Bridge API (`api/engine_api.rs`)

```rust
pub fn start_engine(
    wav_path: String,
    play_output: bool,
    write_output: bool,
    output_path: String,
) -> Result<(), String>

pub fn toggle_bypass(slot: u8, bypass: bool)

pub fn set_param(slot: u8, json: String) -> Result<(), String>

pub fn stop_engine()
```

**State:** a module-level `OnceLock<Mutex<Option<(Runtime, EngineHandle)>>>`.
- `start_engine` initializes it.
- `stop_engine` drops the inner value.
- `toggle_bypass` / `set_param` borrow only `EngineHandle` (non-blocking SPSC push).

**`set_param` JSON contract:** Flutter passes a JSON object matching the effect's params struct, dispatched by slot index:

| Slot | Effect       | Example JSON                                          |
|------|--------------|-------------------------------------------------------|
| 0    | NoiseGate    | `{"threshold": 0.01}`                                |
| 1    | Compressor   | `{"threshold_db": -18.0, "ratio": 4.0, "attack": 0.01, "release": 0.1}` |
| 2    | Overdrive    | `{"drive": 3.0, "tone": 0.5}`                        |
| 3    | Distortion   | `{"drive": 8.0, "level": 0.5}`                       |
| 4    | Fuzz         | `{"fuzz": 0.7, "level": 0.7}`                        |
| 5    | Chorus       | `{"rate": 0.5, "depth": 1.5, "mix": 0.5}`            |
| 6    | Tremolo      | `{"rate": 4.0, "depth": 0.5}`                        |
| 7    | Delay        | `{"time_ms": 300.0, "feedback": 0.4, "mix": 0.4}`    |
| 8    | Reverb       | `{"room_size": 0.5, "mix": 0.3}`                     |

Unknown fields are ignored by `serde_json`.

---

## Rust Test Strategy

Tests bypass `Runtime` entirely:

```rust
let rb = ringbuf::HeapRb::<Command>::new(64);
let (producer, consumer) = rb.split();
let handle = EngineHandle::new(producer);
let mut engine = AudioEngine::new(consumer, sample_rate);

handle.toggle_bypass(0, false);
handle.set_param(2, r#"{"drive": 5.0, "tone": 0.8}"#).unwrap();

let mut buf = vec![0.5_f32; 512];
engine.process_block(&mut buf);
// assert on buf
```

Tests live in `rust/src/engine/mod.rs` under `#[cfg(test)]`. No `cpal`, no audio hardware required.

---

## Audio Thread Rules (unchanged from CLAUDE.md)

Inside `process_block` and the `cpal` callback:
- No heap allocation
- No blocking mutexes
- No `println!` or I/O
- All cross-thread communication via lock-free SPSC rings
