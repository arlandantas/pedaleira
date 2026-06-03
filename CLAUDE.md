# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Real-time, low-latency guitar effects pedal app. Hybrid architecture:

- **Audio Engine:** Rust (`cpal` for audio I/O, `hound` for WAV-based DSP testing)
- **UI:** Flutter (Dart)
- **Bridge:** `flutter_rust_bridge`

## Architecture

### Two-phase development model

**Phase 1 – Algorithm Development (pure Rust):**  
Load a DI `.wav` → run DSP function over float array → save output `.wav`. Use `hound` crate. No live audio, no Flutter. Deterministic, debuggable.

**Phase 2 – UI + Live Integration (Rust + Flutter):**  
Use a Linux virtual audio cable (`module-null-sink`) + `pavucontrol` to route a looping DI `.wav` (via VLC) into the Rust `cpal` input. Connect Flutter knobs via `flutter_rust_bridge`.

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

### Flutter UI Model

- **Main view:** 2×4 grid of 8 fixed pedal slots
- **Single tap:** toggle pedal bypass ON/OFF
- **Long press (or gear icon):** open full-screen edit view for that pedal's parameters
- **Presets:** `< >` arrows at top/bottom; loading a preset instantly applies all 8 pedal states and knob values simultaneously

## Build Commands

```bash
# Rust only (Phase 1 DSP dev)
cargo build                        # from rust/
cargo build --release

# Full app (Flutter + Rust bridge)
flutter build linux                # from project root
flutter run -d linux               # dev mode with hot reload
```

After any toolchain change, clean before rebuilding:
```bash
rm -rf build/
```

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
