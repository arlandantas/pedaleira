# Phase 3 Design — Flutter UI

## Overview

Build the Flutter UI for the guitar effects pedalboard app. The UI is layered over a repository abstraction so it is fully testable without audio hardware or the Rust bridge. The bridge is wired in as a final step by swapping in the real repository implementation.

Design choices:
- Flat dark visual style
- Adaptive layout: portrait (2×4) and landscape (4×2)
- Rotary knobs built with CustomPainter + GestureDetector
- Riverpod for state management, repository pattern for I/O isolation
- Presets stored as JSON files via path_provider

---

## File Layout

```
lib/
  main.dart                          # RustLib.init(), ProviderScope, run app
  src/
    domain/
      models.dart                    # PedalState, Preset, PedalSlot enum, default params
      engine_repository.dart         # abstract EngineRepository
      preset_repository.dart         # abstract PresetRepository
    data/
      rust_engine_repository.dart    # real impl: calls toggle_bypass / set_param
      fake_engine_repository.dart    # test impl: records calls, no-ops
      file_preset_repository.dart    # real impl: path_provider + JSON
      memory_preset_repository.dart  # test impl: in-memory map
    providers/
      engine_provider.dart           # engineRepositoryProvider (overrideable)
      pedalboard_provider.dart       # PedalboardNotifier — 9 pedals + actions
      preset_provider.dart           # PresetNotifier — list, active, load/save
    ui/
      app.dart                       # MaterialApp, theme, routes
      pedalboard/
        pedalboard_screen.dart       # adaptive grid + preset bar
        pedal_tile.dart              # tile: name, LED, tap/long-press
      editor/
        pedal_editor_screen.dart     # full-screen knob view per pedal
        knob_widget.dart             # CustomPainter rotary knob
      preset_bar.dart                # < name > navigation + save
  src/rust/                          # generated bridge (unchanged)

test/
  pedalboard_notifier_test.dart
  preset_notifier_test.dart
  file_preset_repository_test.dart
  pedal_tile_test.dart
  pedal_editor_screen_test.dart
  preset_bar_test.dart
```

New pubspec dependencies: `flutter_riverpod`, `riverpod_annotation`, `path_provider`

---

## Data Models (`domain/models.dart`)

```dart
enum PedalSlot {
  noiseGate, compressor, overdrive, distortion,
  fuzz, chorus, tremolo, delay, reverb,
}

typedef ParamMap = Map<String, double>;

class PedalState {
  final PedalSlot slot;
  final bool bypassed;
  final ParamMap params;

  const PedalState({required this.slot, required this.bypassed, required this.params});

  PedalState copyWith({bool? bypassed, ParamMap? params});
  Map<String, dynamic> toJson();
  factory PedalState.fromJson(Map<String, dynamic> json);
}

class Preset {
  final String name;
  final List<PedalState> pedals; // 9 entries, indexed by PedalSlot.index

  const Preset({required this.name, required this.pedals});

  Map<String, dynamic> toJson();
  factory Preset.fromJson(Map<String, dynamic> json);
}
```

Default params per slot (constants in `models.dart`) match the Rust `set_param` JSON contract:

| Slot | Effect     | Default params                                                    |
|------|------------|-------------------------------------------------------------------|
| 0    | NoiseGate  | `{threshold: 0.01}`                                              |
| 1    | Compressor | `{threshold_db: -18.0, ratio: 4.0, attack: 0.01, release: 0.1}` |
| 2    | Overdrive  | `{drive: 3.0, tone: 0.5}`                                        |
| 3    | Distortion | `{drive: 8.0, level: 0.5}`                                       |
| 4    | Fuzz       | `{fuzz: 0.7, level: 0.7}`                                        |
| 5    | Chorus     | `{rate: 0.5, depth: 1.5, mix: 0.5}`                              |
| 6    | Tremolo    | `{rate: 4.0, depth: 0.5}`                                        |
| 7    | Delay      | `{time_ms: 300.0, feedback: 0.4, mix: 0.4}`                      |
| 8    | Reverb     | `{room_size: 0.5, mix: 0.3}`                                     |

Knob min/max ranges are also constants in `models.dart`, keyed by param name.

---

## Repository Interfaces

```dart
// domain/engine_repository.dart
abstract class EngineRepository {
  Future<void> start(String wavPath);
  Future<void> stop();
  void toggleBypass(int slot, bool bypassed);
  void setParam(int slot, String json);
}

// domain/preset_repository.dart
abstract class PresetRepository {
  Future<List<Preset>> loadAll();
  Future<void> save(Preset preset);
  Future<void> delete(String name);
}
```

`FakeEngineRepository` records all calls in a `List<String>` for assertion in tests.  
`MemoryPresetRepository` holds presets in a `Map<String, Preset>`.  
`FilePresetRepository` stores one JSON file per preset in `getApplicationDocumentsDirectory()/presets/`.

---

## Providers

```dart
// overrideable — swap FakeEngineRepository in tests
final engineRepositoryProvider = Provider<EngineRepository>(
  (ref) => RustEngineRepository(),
);

final presetRepositoryProvider = Provider<PresetRepository>(
  (ref) => FilePresetRepository(),
);

final pedalboardProvider = NotifierProvider<PedalboardNotifier, List<PedalState>>(
  PedalboardNotifier.new,
);

final presetListProvider = NotifierProvider<PresetNotifier, List<Preset>>(
  PresetNotifier.new,
);

final activePresetIndexProvider = StateProvider<int>((ref) => 0);
```

**`PedalboardNotifier`** methods:
- `toggleBypass(int slot)` — flips `bypassed`, calls `engineRepository.toggleBypass`
- `applyPreset(Preset p)` — replaces all 9 states, syncs all params and bypasses to engine
- `updateParam(int slot, String key, double value)` — updates one knob, calls `engineRepository.setParam` with full param JSON for that slot

**`PresetNotifier`** methods:
- `build()` — calls `repository.loadAll()` on init
- `saveCurrentAs(String name, List<PedalState> pedals)` — writes to repository, refreshes list
- `delete(String name)` — removes from repository and list

---

## UI Layer

### App startup (`main.dart`)

```dart
await RustLib.init();
runApp(ProviderScope(
  overrides: [
    engineRepositoryProvider.overrideWithValue(RustEngineRepository()),
    presetRepositoryProvider.overrideWithValue(FilePresetRepository()),
  ],
  child: const App(),
));
```

`PedalboardNotifier.build()` calls `engineRepository.start(wavPath)` once. `wavPath` is a compile-time constant pointing to the sample DI file (e.g. `'sample_audios/di_guitar.wav'`). In tests the override uses `FakeEngineRepository` — no audio setup required.

### Pedalboard screen

`OrientationBuilder` drives the grid cross-axis count:
- Portrait → 2 columns (slots 0–7 in the grid, reverb as a separate bottom strip)
- Landscape → 4 columns (same slots, reverb strip at bottom)

### PedalTile

```
┌────────────────────┐
│ ●  [⚙]             │  ← LED + gear icon
│                    │
│    NOISE GATE      │
└────────────────────┘
```
- LED: accent color when active, dim grey when bypassed
- Tap → `pedalboardProvider.notifier.toggleBypass(slot)`
- Long-press or gear icon → `Navigator.push(PedalEditorScreen(slot))`

### Rotary knob (`KnobWidget`)

- CustomPainter draws a 300° arc (210° start → 330° end)
- A dot on the arc indicates current value
- `GestureDetector.onPanUpdate`: vertical drag delta maps to value change
- Value clamped to `[min, max]` from constants; normalized for arc position
- Label displays param name and formatted value below the knob

### PedalEditorScreen

- AppBar: pedal name + back button
- Body: `Wrap` of `KnobWidget`s, one per param key in `PedalState.params`
- Each knob calls `pedalboardProvider.notifier.updateParam(slot, key, value)` on change

### PresetBar

```
  [<]   Clean Boost   [>]   [Save]
```
- Pinned at top of `PedalboardScreen`
- `<` / `>` change `activePresetIndexProvider`, then call `applyPreset`
- Save: if active preset exists, overwrites; if no presets, prompts for a name via `showDialog`

---

## Testing Strategy

All tests run with `flutter test` — no audio hardware, no Rust bridge needed.

**Unit tests** (use `ProviderContainer` directly):
- `pedalboard_notifier_test.dart` — toggle bypass, update param, apply preset; assert `FakeEngineRepository.calls`
- `preset_notifier_test.dart` — load/save/delete with `MemoryPresetRepository`
- `file_preset_repository_test.dart` — write JSON to temp dir, read back, assert round-trip

**Widget tests** (use `ProviderScope` with overrides):
- `pedal_tile_test.dart` — tap toggles bypass state, long-press navigates to editor
- `pedal_editor_screen_test.dart` — knob drag fires `updateParam` with correct args
- `preset_bar_test.dart` — `<`/`>` advance index, save writes to repository

No new integration tests in this phase — bridge integration is already covered by the Phase 2 integration test.

---

## Bridge Wiring (final step)

`RustEngineRepository` is the last file implemented. It wraps the generated bridge:

```dart
class RustEngineRepository implements EngineRepository {
  void toggleBypass(int slot, bool bypassed) =>
    unawaited(toggleBypassBridge(slot: slot, bypass: bypassed));

  void setParam(int slot, String json) =>
    unawaited(setParamBridge(slot: slot, json: json));

  Future<void> start(String wavPath) =>
    startEngine(wavPath: wavPath, playOutput: true, writeOutput: false, outputPath: '');

  Future<void> stop() => stopEngine();
}
```

Until this file is wired in `main.dart`, the app runs entirely against `FakeEngineRepository`.
