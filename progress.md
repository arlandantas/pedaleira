# Pedaleira — Progress

## Done ✅
- [x] Project scaffold: Flutter + Rust + flutter_rust_bridge 2.12.0
- [x] Bridge demo (greet + init_app) compiles and runs
- [x] Integration test validates bridge end-to-end

## Phase 1 — DSP Algorithms (Pure Rust, offline WAV tests)
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

## Phase 2 — Live Audio + Bridge API
- [ ] cpal audio I/O (input → process → output)
- [ ] Effects chain struct (8 fixed slots + reverb)
- [ ] Lock-free param sync (ringbuf SPSC)
- [ ] flutter_rust_bridge API (toggle bypass, set param, load/save preset)
- [ ] Regenerate bridge bindings

## Phase 3 — Flutter UI
- [ ] App state + data models (Riverpod)
- [ ] Main pedalboard screen (2×4 grid)
- [ ] Pedal tile widget (bypass toggle + long-press to edit)
- [ ] Rotary knob widget
- [ ] Pedal editor screen (full-screen knobs)
- [ ] Preset navigation bar (< name >)
- [ ] Preset load/save persistence (JSON)
- [ ] Fix/update widget tests
