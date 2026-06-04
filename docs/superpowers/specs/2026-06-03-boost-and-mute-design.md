# Boost Pedal + Global Mute Design

Date: 2026-06-03

## Overview

Two output-level features:
1. **Clean Boost pedal** — slot 9, after Reverb, part of the preset system
2. **Global Mute** — persists across preset changes, shown as a dedicated bottom bar

---

## 1. Clean Boost Pedal (slot 9)

### DSP (Rust)

New file `rust/src/dsp/boost.rs`:
- Single param: `gain: f32` (linear multiplier)
- `process(buffer)`: multiplies every sample by `gain`
- No state, no memory — trivially real-time safe

New param struct in `rust/src/dsp/params.rs`:
```rust
pub struct BoostParams { pub gain: f32 }
```

### Effects Chain

`rust/src/dsp/chain.rs`:
- Add `boost: Boost` field
- Grow `bypass` array from `[bool; 9]` to `[bool; 10]`
- Slot 9 is processed last, after Reverb
- `apply_params` handles `EffectParams::Boost`
- `EffectParams` enum gets `Boost(BoostParams)` variant

### Bridge API

`rust/src/engine/handle.rs`:
- `parse_params` slot 9 → `BoostParams`

No new bridge functions needed — existing `toggle_bypass` and `set_param` already accept `slot: u8`.

### Flutter Models

`lib/src/domain/models.dart`:
- `PedalSlot.boost` added (index 9, after `reverb`)
- `kPedalNames[PedalSlot.boost] = 'Boost'`
- `kDefaultParams[PedalSlot.boost] = {'gain': 1.0}` (unity gain)
- `kParamRanges['gain'] = (0.0, 4.0)`

### UI

Bottom row changes from a single full-width Reverb tile to two equal tiles:

```
[ Boost ]   [ Reverb ]
[      Mute bar      ]
```

`PedalboardScreen` currently has a fixed `SizedBox(height: 72)` for slot 8 (Reverb). This becomes a `Row` of two equal `PedalTile`s for slots 9 (Boost) and 8 (Reverb).

The LayoutBuilder height math must account for this: the bottom row height stays 72, now split between two tiles.

Boost is part of `pedalboardProvider` state (index 9) and is included in `Preset.pedals`, so each preset has its own gain setting. The existing `applyPreset`, `toggleBypass`, and `updateParam` paths handle it automatically once the model is extended.

---

## 2. Global Mute

### Rust

`rust/src/engine/runtime.rs`:
- Add `muted: Arc<AtomicBool>` to `Runtime`
- Audio callback reads `muted` after processing and zeros all output samples if true
- No heap allocation or locking in the callback — `AtomicBool::load(Relaxed)` only

New bridge function in `rust/src/api/engine_api.rs`:
```rust
pub fn set_mute(muted: bool) -> Result<(), String>
```
Sends the value into the `AtomicBool` via `store(muted, Relaxed)`.

`engine_api.rs` holds a second static `MUTE: Mutex<Option<Arc<AtomicBool>>>` alongside `ENGINE`. When the engine starts, the `Arc<AtomicBool>` is cloned into both `Runtime` (for the audio callback) and `MUTE` (for `set_mute`). When the engine stops, `MUTE` is cleared. This avoids locking the full engine mutex on every mute toggle.

### Bridge (Dart codegen)

After adding `set_mute` to the Rust API, run:
```bash
flutter_rust_bridge_codegen generate
```

New generated Dart function: `setMute({required bool muted})` in `engine_api.dart`.

### Flutter

`lib/src/domain/engine_repository.dart`:
- Add `void setMute(bool muted)` to the abstract interface

`lib/src/data/rust_engine_repository.dart`:
- Implement `setMute` → calls `bridge.setMute(muted: muted)`

`lib/src/data/fake_engine_repository.dart`:
- Implement `setMute` → records call as `'setMute:$muted'`

`lib/src/providers/engine_provider.dart` (or new file):
- `muteProvider`: `StateNotifierProvider<MuteNotifier, bool>` (or simple `StateProvider<bool>`)
- On toggle: calls `engineRepositoryProvider.setMute(newValue)`
- State is not reset on preset navigation — lives independently

### UI

New widget `lib/src/ui/pedalboard/mute_bar.dart`:
- Full-width bar, height 56
- Shows "MUTED" label and a `Switch`
- When muted: background turns red/accent, label reads "MUTED"
- When unmuted: normal surface color, label reads "Output"

`PedalboardScreen` layout (bottom, below grid):
```
Column:
  Expanded → GridView (8 slots, fills space)
  Row(height: 72) → [PedalTile(slot:9, Boost)] [PedalTile(slot:8, Reverb)]
  MuteBar (height: 56)
```

LayoutBuilder deducts `72 + 56` from available height for the grid content height calculation (was `72` before).

---

## Out of Scope

- Audio device selection UI (tracked as Phase 4 TODO)
- Mute state persisted to disk across app restarts
- Boost expressed in dB in the UI
