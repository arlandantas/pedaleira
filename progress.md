# Pedaleira — Progress

## Done ✅
- [x] Project scaffold: Flutter + Rust + flutter_rust_bridge 2.12.0
- [x] Bridge demo (greet + init_app) compiles and runs
- [x] Integration test validates bridge end-to-end

## Phase 1 — DSP Algorithms (Pure Rust, offline WAV tests) ✅
- [x] Test harness: hound + WAV helpers + render_all_effects test
- [x] Noise Gate (threshold + hold, 3 tests)
- [x] Compressor (feedforward RMS, attack/release, 3 tests)
- [x] Transparent Overdrive (soft cubic clipping, 3 tests)
- [x] Distortion (asymmetric hard-clip, 3 tests)
- [x] Fuzz (double tanh saturation, 4 tests)
- [x] Tremolo (LFO amplitude modulation, 3 tests)
- [x] Chorus (LFO delay line, 4 tests)
- [x] Delay (digital delay + feedback, 4 tests)
- [x] Reverb (Schroeder comb + all-pass, 4 tests)
- [x] Effects chain (9-slot serial + bypass + param dispatch, 5 tests)

## Phase 2 — Live Audio + Bridge API ✅
- [x] cpal audio I/O (WAV looping input → DSP → output device + file tee)
- [x] Effects chain struct (8 fixed slots + reverb) — reused from Phase 1
- [x] Lock-free param sync (ringbuf SPSC Command queue)
- [x] flutter_rust_bridge API (toggle bypass, set param, start/stop engine)
- [x] Regenerate bridge bindings

## Phase 3 — Flutter UI ✅
- [x] App state + data models (Riverpod — PedalState, Preset, PedalSlot, defaults, param ranges)
- [x] Repository pattern (EngineRepository + PresetRepository interfaces, fake/memory test doubles)
- [x] PedalboardNotifier (9 pedals, bypass toggle, param update, apply preset)
- [x] PresetNotifier (load/save/delete via FilePresetRepository — JSON files in app documents dir)
- [x] Main pedalboard screen (adaptive 2×4 portrait / 4×2 landscape grid + reverb strip)
- [x] Pedal tile widget (LED indicator, bypass toggle, long-press to editor)
- [x] Rotary knob widget (CustomPainter 300° arc, vertical pan gesture)
- [x] Pedal editor screen (full-screen knob grid per pedal)
- [x] Preset navigation bar (< name > arrows + save dialog)
- [x] RustEngineRepository (bridge wiring — toggle_bypass, set_param, start/stop)
- [x] 30 widget + unit tests (all pass without audio hardware)
- [x] Sample audio converted to WAV (sample_audios/guitar_di.wav, 44100 Hz mono 16-bit)

## Phase 4 — Production / Raspberry Pi (Pending)
- [ ] BLE or WiFi transport layer (replace flutter_rust_bridge with network protocol)
- [ ] Flutter app targets smartphone (Android/iOS build)
- [ ] Raspberry Pi: Rust engine runs as systemd service on Patchbox OS
- [ ] Preset sync between phone and Pi (push/pull over network)
- [ ] GPIO footswitch support on Pi (toggle bypass without phone)
- [ ] I2C OLED display on Pi (show active preset name)
- [ ] Low-latency tuning: real-time kernel, JACK or pipewire-jack, USB audio interface
- [ ] Headless Pi mode: engine starts on boot, last preset loaded automatically

## Known Limitations / Tech Debt
- Audio input is a looping WAV file (`sample_audios/guitar_di.wav`); live microphone/line-in requires a USB audio interface and virtual cable setup (see CLAUDE.md Phase 2 notes)
- No error UI if the engine fails to start (e.g. WAV file missing)
- Preset names must be valid filenames (no `/`, `\`, etc.) — no validation in UI
- Knob drag sensitivity (150px per full range) is not yet user-configurable
