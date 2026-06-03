# Pedaleira

Real-time, low-latency guitar effects pedal app. Hybrid Rust + Flutter architecture.

## What it is

A digital pedalboard with 8 fixed effect slots plus global reverb. Tap a pedal to toggle it on/off; long-press to edit its parameters via rotary knobs. Save and recall named presets instantly.

**Current state:** Fully functional on a single Linux development machine. The Rust audio engine loops a WAV file through the effects chain and plays to the system audio output. The Flutter UI connects to the engine via `flutter_rust_bridge`.

## Architecture

| Layer | Technology |
|-------|-----------|
| Audio DSP | Rust (`cpal`, `ringbuf`, `hound`) |
| UI | Flutter / Dart |
| Bridge | `flutter_rust_bridge` 2.12.0 |
| State | Riverpod (`Notifier` / `AsyncNotifier`) |
| Persistence | JSON files via `path_provider` |

### Effects chain (fixed order)

| Slot | Effect |
|------|--------|
| 1 | Noise Gate |
| 2 | Compressor |
| 3 | Overdrive |
| 4 | Distortion |
| 5 | Fuzz |
| 6 | Chorus |
| 7 | Tremolo |
| 8 | Delay |
| Out | Reverb (global) |

## Getting started

### Prerequisites

- Flutter from tarball at `~/flutter/bin` (**not** snap — snap's `ld` breaks Cargokit)
- Rust via rustup
- `ffmpeg` (for converting audio samples)

Both `flutter` and `cargo` are on PATH via `~/.profile`. No manual `export` needed.

### Build and run

```bash
# Run tests (no audio hardware needed)
flutter test

# Run the app (requires sample_audios/guitar_di.wav)
flutter run -d linux

# Build release
flutter build linux
```

### Audio input

The engine reads `sample_audios/guitar_di.wav` on startup. A converted sample is already included. To use your own DI track:

```bash
ffmpeg -i your_track.mp3 -ar 44100 -ac 1 -sample_fmt s16 sample_audios/guitar_di.wav
```

For live guitar input, set up a virtual audio cable (see [CLAUDE.md](CLAUDE.md) → Phase 2 notes).

### After changing the Rust API

```bash
flutter_rust_bridge_codegen generate   # from project root
```

Then rebuild: `flutter run -d linux`.

## Project structure

```
rust/src/
  dsp/          # Pure DSP algorithms (Phase 1 — all tested offline)
  engine/       # AudioEngine + Command SPSC + Runtime (cpal wiring)
  api/          # flutter_rust_bridge-exposed functions

lib/src/
  domain/       # PedalState, Preset, repository interfaces
  data/         # FilePresetRepository, RustEngineRepository, fakes
  providers/    # Riverpod notifiers
  ui/           # Screens and widgets
    pedalboard/ # PedalboardScreen, PedalTile
    editor/     # PedalEditorScreen, KnobWidget
    preset_bar.dart

sample_audios/  # WAV files for the engine (git-ignored large files)
docs/
  superpowers/specs/   # Design specs per phase
  superpowers/plans/   # Implementation plans per phase
```

## Test suite

```bash
flutter test        # 30 tests — runs without audio hardware
cargo test          # Rust DSP unit tests (from rust/)
```

All Flutter tests use `FakeEngineRepository` and `MemoryPresetRepository` — no real audio device or WAV file required.

## Roadmap

See [progress.md](progress.md) for the full task history. Phase 4 (Raspberry Pi / production) is next:

- BLE/WiFi transport replacing the local bridge
- Flutter app targeting Android/iOS
- Rust engine as a systemd service on Patchbox OS (real-time Linux)
- GPIO footswitches and I2C OLED display on the Pi
