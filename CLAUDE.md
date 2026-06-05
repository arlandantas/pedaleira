# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Real-time, low-latency guitar effects pedal app. Hybrid architecture:

- **Audio Engine:** Rust (`cpal` for audio I/O, `hound` for WAV-based DSP testing)
- **UI:** Flutter (Dart)
- **Bridge:** `flutter_rust_bridge`

## Architecture

### Development phases

**Phase 1 – Algorithm Development (pure Rust) ✅**  
Load a DI `.wav` → run DSP function over float array → save output `.wav`. Use `hound` crate. No live audio, no Flutter. Deterministic, debuggable.

**Phase 2 – Live Audio + Bridge API ✅**  
`cpal` audio I/O — reads `sample_audios/guitar_di.wav` on loop, processes through the effects chain, plays to system output. Lock-free SPSC command queue for real-time param sync. `flutter_rust_bridge` API: `start_engine`, `stop_engine`, `toggle_bypass`, `set_param`.

**Phase 3 – Flutter UI ✅**  
Full Flutter UI with Riverpod state management, repository pattern (fake impls for testing), adaptive pedalboard grid, rotary knobs (CustomPainter), pedal editor screen, preset save/load (JSON files). 30 tests; none require audio hardware.

**Phase 4 – Raspberry Pi / Production (next)**  
BLE or WiFi transport replacing the local bridge; Flutter app targeting phone; Rust engine as systemd service on Patchbox OS; GPIO footswitches; I2C OLED display.

### Live audio input (Phase 2 dev setup)

For live guitar testing (instead of the WAV loop): create a Linux virtual audio cable (`module-null-sink`) + `pavucontrol` to route VLC's output into the Rust `cpal` input.

### Audio Thread Rules (CRITICAL — never break these)

The audio callback is real-time. Inside it:
- **No heap allocation** (`Vec::new()`, `String`, etc.)
- **No blocking mutexes**
- **No `println!` or any I/O**
- **No garbage collection triggers**

UI → audio thread communication must use lock-free primitives: atomic variables or bounded SPSC queues (`crossbeam` / `ringbuf`).

### Effects Chain (fixed 8-slot serial + global reverb)

| Slot | Effect              |
|------|---------------------|
| 1    | Noise Gate          |
| 2    | Compressor          |
| 3    | Transparent OD      |
| 4    | Distortion          |
| 5    | Fuzz                |
| 6    | Chorus              |
| 7    | Tremolo             |
| 8    | Delay               |
| Out  | Reverb (global)     |
| 9    | Boost (output gain) |

### Flutter UI Model

- **Main view:** 2×4 grid of 8 fixed pedal slots
- **Single tap:** toggle pedal bypass ON/OFF
- **Long press (or gear icon):** open full-screen edit view for that pedal's parameters
- **Presets:** `< >` arrows at top/bottom; loading a preset instantly applies all 8 pedal states and knob values simultaneously

## Build Commands

```bash
# Run all tests (no audio hardware needed)
flutter test                       # 47 Flutter tests
cargo test                         # Rust DSP tests (from rust/)

# Run / build via Makefile
make run                           # Linux desktop (hot reload)
make run-android                   # connected Android device (adb auto-detect)
make flutter-build                 # release build for Linux
make build-android                 # debug APK (no device needed)
make build-android-release         # release APK

# Rust only (DSP development)
cargo build                        # from rust/
cargo build --release
```

After any toolchain change, clean before rebuilding:
```bash
rm -rf build/
```

### Audio sample

The engine expects `sample_audios/guitar_di.wav` (44100 Hz, mono, 16-bit PCM). A converted sample is already present. To replace it:

```bash
ffmpeg -i your_track.mp3 -ar 44100 -ac 1 -sample_fmt s16 sample_audios/guitar_di.wav
```

## Git Workflow (agents: read this)

This project is in **pre-MVP development**. Commit directly to `main` — do not create feature branches unless the user explicitly asks for one. When the `finishing-a-development-branch` skill presents options, choose "merge locally" (option 1), which is a no-op when already on `main`.

## Running CLI Commands (agents: read this)

Both `flutter` and `cargo` are available via `~/.profile`. The Bash tool starts a login shell, so they are on PATH automatically — **no `export PATH=...` prefix needed**.

```bash
flutter test          # just works
cargo build           # just works
```

If a command is not found, the shell is not login-sourced. Fix with:
```bash
bash -l -c 'flutter test'
```

**Never use the snap `flutter`** — it ships a broken `ld` that silently breaks Cargokit's Rust compilation. The tarball install at `~/flutter/bin` is the only valid one.

## Environment Notes

- **Flutter:** installed from tarball at `~/flutter/bin`, added to PATH in `~/.profile`. Do NOT install via snap.
- **Rust:** installed via rustup, bin at `~/.cargo/bin/`, sourced via `~/.profile` → `.cargo/env`.
- **Codegen:** after changing Rust API surface: `flutter_rust_bridge_codegen generate` from project root.

## Target Platform

Development: single Kubuntu Linux notebook (Rust engine + Flutter UI running locally).  
Production: Raspberry Pi (real-time Linux / Patchbox OS) running the DSP engine; smartphone Flutter app sending lightweight control messages over BLE/WiFi.
