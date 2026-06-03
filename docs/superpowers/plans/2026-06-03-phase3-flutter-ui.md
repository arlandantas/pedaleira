# Phase 3 — Flutter UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the complete Flutter pedalboard UI — app state, widgets, preset persistence — testable without audio hardware, with the Rust bridge wired in as a final step.

**Architecture:** Repository pattern with abstract `EngineRepository` and `PresetRepository` interfaces. Riverpod `Notifier`/`AsyncNotifier` providers sit on top. `FakeEngineRepository` and `MemoryPresetRepository` are swapped in for all tests via `ProviderScope` overrides. The real `RustEngineRepository` wraps the generated FRB bridge and is injected in `main.dart` only.

**Tech Stack:** Flutter 3, Dart 3, `flutter_riverpod ^2.5.0`, `path_provider ^2.1.0`, `flutter_rust_bridge` (already present), `dart:convert`, `dart:math`, `dart:io`

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `pubspec.yaml` | Modify | Add flutter_riverpod, path_provider |
| `lib/src/domain/models.dart` | Create | PedalSlot enum, PedalState, Preset, defaults, ranges |
| `lib/src/domain/engine_repository.dart` | Create | Abstract EngineRepository interface |
| `lib/src/domain/preset_repository.dart` | Create | Abstract PresetRepository interface |
| `lib/src/data/fake_engine_repository.dart` | Create | Test double: records calls |
| `lib/src/data/memory_preset_repository.dart` | Create | Test double: in-memory map |
| `lib/src/data/file_preset_repository.dart` | Create | Real: path_provider + JSON files |
| `lib/src/data/rust_engine_repository.dart` | Create | Real: wraps FRB bridge functions |
| `lib/src/providers/engine_provider.dart` | Create | engineRepositoryProvider, presetRepositoryProvider |
| `lib/src/providers/pedalboard_provider.dart` | Create | PedalboardNotifier, pedalboardProvider |
| `lib/src/providers/preset_provider.dart` | Create | PresetNotifier, presetListProvider, activePresetIndexProvider |
| `lib/src/ui/app.dart` | Create | MaterialApp + flat dark theme |
| `lib/src/ui/pedalboard/pedalboard_screen.dart` | Create | Adaptive grid + reverb strip |
| `lib/src/ui/pedalboard/pedal_tile.dart` | Create | Pedal card: LED, name, tap/long-press |
| `lib/src/ui/editor/knob_widget.dart` | Create | CustomPainter rotary knob + pan gesture |
| `lib/src/ui/editor/pedal_editor_screen.dart` | Create | Full-screen knob editor per pedal |
| `lib/src/ui/preset_bar.dart` | Create | `< name >` navigation + save dialog |
| `lib/main.dart` | Modify | Wire real repos + RustLib.init |
| `test/models_test.dart` | Create | Unit tests: JSON round-trip for PedalState + Preset |
| `test/pedalboard_notifier_test.dart` | Create | Unit tests: toggle, param update, apply preset |
| `test/preset_notifier_test.dart` | Create | Unit tests: load, save, delete |
| `test/file_preset_repository_test.dart` | Create | Unit tests: JSON round-trip |
| `test/pedal_tile_test.dart` | Create | Widget test: tap bypass, long-press nav |
| `test/pedal_editor_screen_test.dart` | Create | Widget test: knobs rendered per slot |
| `test/preset_bar_test.dart` | Create | Widget test: navigate presets, save |
| `test/widget_test.dart` | Modify | Update smoke test to use new App |

---

## Task 1: Add pubspec dependencies

**Files:**
- Modify: `pubspec.yaml`

- [ ] **Step 1: Add dependencies**

In `pubspec.yaml`, under `dependencies:`, add after the `flutter_rust_bridge` line:

```yaml
  flutter_riverpod: ^2.5.0
  path_provider: ^2.1.0
```

The `dependencies:` block should look like:

```yaml
dependencies:
  flutter:
    sdk: flutter
  cupertino_icons: ^1.0.8
  rust_lib_pedaleira:
    path: rust_builder
  flutter_rust_bridge: 2.12.0
  flutter_riverpod: ^2.5.0
  path_provider: ^2.1.0
```

- [ ] **Step 2: Fetch packages**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter pub get
```

Expected: output ends with `Got dependencies!` (no errors).

- [ ] **Step 3: Commit**

```bash
git add pubspec.yaml pubspec.lock
git commit -m "build: add flutter_riverpod and path_provider"
```

---

## Task 2: Domain models

**Files:**
- Create: `lib/src/domain/models.dart`
- Create: `test/models_test.dart`

- [ ] **Step 1: Write the failing test**

Create `test/models_test.dart`:

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/domain/models.dart';

void main() {
  group('PedalState', () {
    test('JSON round-trip', () {
      final state = PedalState(
        slot: PedalSlot.compressor,
        bypassed: false,
        params: {'threshold_db': -18.0, 'ratio': 4.0, 'attack': 0.01, 'release': 0.1},
      );
      final restored = PedalState.fromJson(state.toJson());
      expect(restored.slot, PedalSlot.compressor);
      expect(restored.bypassed, false);
      expect(restored.params['ratio'], 4.0);
    });

    test('copyWith only changes specified fields', () {
      final original = PedalState(
        slot: PedalSlot.delay,
        bypassed: true,
        params: {'time_ms': 300.0},
      );
      final copy = original.copyWith(bypassed: false);
      expect(copy.bypassed, false);
      expect(copy.slot, PedalSlot.delay);
      expect(copy.params['time_ms'], 300.0);
    });
  });

  group('Preset', () {
    test('JSON round-trip', () {
      final pedals = PedalSlot.values.map((s) => PedalState(
        slot: s,
        bypassed: true,
        params: Map.from(kDefaultParams[s]!),
      )).toList();
      final preset = Preset(name: 'Clean', pedals: pedals);
      final restored = Preset.fromJson(preset.toJson());
      expect(restored.name, 'Clean');
      expect(restored.pedals.length, 9);
      expect(restored.pedals[1].slot, PedalSlot.compressor);
    });
  });
}
```

- [ ] **Step 2: Run test to see it fail**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/models_test.dart
```

Expected: FAIL — `lib/src/domain/models.dart` does not exist.

- [ ] **Step 3: Create `lib/src/domain/models.dart`**

```dart
import 'dart:convert';

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
}

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
};

const Map<PedalSlot, Map<String, double>> kDefaultParams = {
  PedalSlot.noiseGate: {'threshold': 0.01},
  PedalSlot.compressor: {
    'threshold_db': -18.0,
    'ratio': 4.0,
    'attack': 0.01,
    'release': 0.1,
  },
  PedalSlot.overdrive: {'drive': 3.0, 'tone': 0.5},
  PedalSlot.distortion: {'drive': 8.0, 'level': 0.5},
  PedalSlot.fuzz: {'fuzz': 0.7, 'level': 0.7},
  PedalSlot.chorus: {'rate': 0.5, 'depth': 1.5, 'mix': 0.5},
  PedalSlot.tremolo: {'rate': 4.0, 'depth': 0.5},
  PedalSlot.delay: {'time_ms': 300.0, 'feedback': 0.4, 'mix': 0.4},
  PedalSlot.reverb: {'room_size': 0.5, 'mix': 0.3},
};

// Min/max ranges per param key — used by KnobWidget
const Map<String, (double, double)> kParamRanges = {
  'threshold': (0.0, 0.5),
  'threshold_db': (-60.0, 0.0),
  'ratio': (1.0, 20.0),
  'attack': (0.001, 0.5),
  'release': (0.01, 2.0),
  'drive': (1.0, 20.0),
  'tone': (0.0, 1.0),
  'level': (0.0, 1.0),
  'fuzz': (0.0, 1.0),
  'rate': (0.1, 10.0),
  'depth': (0.0, 3.0),
  'mix': (0.0, 1.0),
  'time_ms': (50.0, 1000.0),
  'feedback': (0.0, 0.95),
  'room_size': (0.0, 1.0),
};

class PedalState {
  final PedalSlot slot;
  final bool bypassed;
  final Map<String, double> params;

  const PedalState({
    required this.slot,
    required this.bypassed,
    required this.params,
  });

  PedalState copyWith({bool? bypassed, Map<String, double>? params}) {
    return PedalState(
      slot: slot,
      bypassed: bypassed ?? this.bypassed,
      params: params ?? this.params,
    );
  }

  Map<String, dynamic> toJson() => {
    'slot': slot.index,
    'bypassed': bypassed,
    'params': params,
  };

  factory PedalState.fromJson(Map<String, dynamic> json) {
    return PedalState(
      slot: PedalSlot.values[json['slot'] as int],
      bypassed: json['bypassed'] as bool,
      params: Map<String, double>.from(
        (json['params'] as Map<String, dynamic>).map(
          (k, v) => MapEntry(k, (v as num).toDouble()),
        ),
      ),
    );
  }
}

class Preset {
  final String name;
  final List<PedalState> pedals;

  const Preset({required this.name, required this.pedals});

  Map<String, dynamic> toJson() => {
    'name': name,
    'pedals': pedals.map((p) => p.toJson()).toList(),
  };

  factory Preset.fromJson(Map<String, dynamic> json) {
    return Preset(
      name: json['name'] as String,
      pedals: (json['pedals'] as List<dynamic>)
          .map((p) => PedalState.fromJson(p as Map<String, dynamic>))
          .toList(),
    );
  }
}
```

- [ ] **Step 4: Run test to see it pass**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/models_test.dart
```

Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add lib/src/domain/models.dart test/models_test.dart
git commit -m "feat: domain models — PedalState, Preset, defaults, ranges"
```

---

## Task 3: Repository interfaces and test doubles

**Files:**
- Create: `lib/src/domain/engine_repository.dart`
- Create: `lib/src/domain/preset_repository.dart`
- Create: `lib/src/data/fake_engine_repository.dart`
- Create: `lib/src/data/memory_preset_repository.dart`

- [ ] **Step 1: Create `lib/src/domain/engine_repository.dart`**

```dart
abstract class EngineRepository {
  void start(String wavPath);
  void stop();
  void toggleBypass(int slot, bool bypassed);
  void setParam(int slot, String json);
}
```

- [ ] **Step 2: Create `lib/src/domain/preset_repository.dart`**

```dart
import '../domain/models.dart';

abstract class PresetRepository {
  Future<List<Preset>> loadAll();
  Future<void> save(Preset preset);
  Future<void> delete(String name);
}
```

- [ ] **Step 3: Create `lib/src/data/fake_engine_repository.dart`**

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
}
```

- [ ] **Step 4: Create `lib/src/data/memory_preset_repository.dart`**

```dart
import '../domain/models.dart';
import '../domain/preset_repository.dart';

class MemoryPresetRepository implements PresetRepository {
  final Map<String, Preset> _store = {};

  @override
  Future<List<Preset>> loadAll() async => _store.values.toList();

  @override
  Future<void> save(Preset preset) async => _store[preset.name] = preset;

  @override
  Future<void> delete(String name) async => _store.remove(name);
}
```

- [ ] **Step 5: Confirm compilation**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter analyze lib/src/domain lib/src/data
```

Expected: No errors.

- [ ] **Step 6: Commit**

```bash
git add lib/src/domain/engine_repository.dart lib/src/domain/preset_repository.dart \
        lib/src/data/fake_engine_repository.dart lib/src/data/memory_preset_repository.dart
git commit -m "feat: repository interfaces + fake/memory test doubles"
```

---

## Task 4: Providers

**Files:**
- Create: `lib/src/providers/engine_provider.dart`
- Create: `lib/src/providers/pedalboard_provider.dart`
- Create: `lib/src/providers/preset_provider.dart`

- [ ] **Step 1: Create `lib/src/providers/engine_provider.dart`**

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../data/fake_engine_repository.dart';
import '../data/memory_preset_repository.dart';
import '../domain/engine_repository.dart';
import '../domain/preset_repository.dart';

// Override these in main.dart (real impls) and tests (fakes).
final engineRepositoryProvider = Provider<EngineRepository>(
  (ref) => FakeEngineRepository(),
);

final presetRepositoryProvider = Provider<PresetRepository>(
  (ref) => MemoryPresetRepository(),
);
```

- [ ] **Step 2: Create `lib/src/providers/pedalboard_provider.dart`**

```dart
import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../domain/models.dart';
import 'engine_provider.dart';

class PedalboardNotifier extends Notifier<List<PedalState>> {
  @override
  List<PedalState> build() {
    return PedalSlot.values.map((slot) => PedalState(
      slot: slot,
      bypassed: true,
      params: Map.from(kDefaultParams[slot]!),
    )).toList();
  }

  void toggleBypass(int slot) {
    final current = state[slot];
    final updated = current.copyWith(bypassed: !current.bypassed);
    state = [...state]..[slot] = updated;
    ref.read(engineRepositoryProvider).toggleBypass(slot, updated.bypassed);
  }

  void updateParam(int slot, String key, double value) {
    final current = state[slot];
    final newParams = Map<String, double>.from(current.params)..[key] = value;
    state = [...state]..[slot] = current.copyWith(params: newParams);
    ref.read(engineRepositoryProvider).setParam(slot, jsonEncode(newParams));
  }

  void applyPreset(Preset preset) {
    state = List.from(preset.pedals);
    final engine = ref.read(engineRepositoryProvider);
    for (final pedal in preset.pedals) {
      final idx = pedal.slot.index;
      engine.toggleBypass(idx, pedal.bypassed);
      engine.setParam(idx, jsonEncode(pedal.params));
    }
  }
}

final pedalboardProvider =
    NotifierProvider<PedalboardNotifier, List<PedalState>>(
  PedalboardNotifier.new,
);
```

- [ ] **Step 3: Create `lib/src/providers/preset_provider.dart`**

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../domain/models.dart';
import 'engine_provider.dart';

class PresetNotifier extends AsyncNotifier<List<Preset>> {
  @override
  Future<List<Preset>> build() async {
    return ref.read(presetRepositoryProvider).loadAll();
  }

  Future<void> saveCurrentAs(String name, List<PedalState> pedals) async {
    final preset = Preset(name: name, pedals: pedals);
    await ref.read(presetRepositoryProvider).save(preset);
    state = AsyncValue.data(
      await ref.read(presetRepositoryProvider).loadAll(),
    );
  }

  Future<void> delete(String name) async {
    await ref.read(presetRepositoryProvider).delete(name);
    state = AsyncValue.data(
      state.value?.where((p) => p.name != name).toList() ?? [],
    );
  }
}

final presetListProvider =
    AsyncNotifierProvider<PresetNotifier, List<Preset>>(
  PresetNotifier.new,
);

final activePresetIndexProvider = StateProvider<int>((ref) => 0);
```

- [ ] **Step 4: Verify no analysis errors**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter analyze lib/src/providers
```

Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add lib/src/providers/
git commit -m "feat: Riverpod providers — PedalboardNotifier, PresetNotifier"
```

---

## Task 5: PedalboardNotifier unit tests

**Files:**
- Create: `test/pedalboard_notifier_test.dart`

- [ ] **Step 1: Create `test/pedalboard_notifier_test.dart`**

```dart
import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/domain/models.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/pedalboard_provider.dart';

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
  test('initial state has 9 pedals all bypassed with default params', () {
    final container = makeContainer();
    final state = container.read(pedalboardProvider);
    expect(state.length, 9);
    expect(state.every((p) => p.bypassed), isTrue);
    expect(state[1].params['threshold_db'], -18.0);
  });

  test('toggleBypass flips bypassed and records engine call', () {
    final engine = FakeEngineRepository();
    final container = makeContainer(engine: engine);
    final notifier = container.read(pedalboardProvider.notifier);

    notifier.toggleBypass(0);

    expect(container.read(pedalboardProvider)[0].bypassed, isFalse);
    expect(engine.calls, contains('toggle:0:false'));
  });

  test('toggleBypass twice returns to original state', () {
    final container = makeContainer();
    final notifier = container.read(pedalboardProvider.notifier);

    notifier.toggleBypass(3);
    notifier.toggleBypass(3);

    expect(container.read(pedalboardProvider)[3].bypassed, isTrue);
  });

  test('updateParam updates state and calls setParam with full JSON', () {
    final engine = FakeEngineRepository();
    final container = makeContainer(engine: engine);
    final notifier = container.read(pedalboardProvider.notifier);

    notifier.updateParam(2, 'drive', 7.5);

    final params = container.read(pedalboardProvider)[2].params;
    expect(params['drive'], 7.5);
    expect(params['tone'], 0.5); // other params unchanged

    final call = engine.calls.firstWhere((c) => c.startsWith('setParam:2:'));
    final json = jsonDecode(call.substring('setParam:2:'.length)) as Map;
    expect(json['drive'], 7.5);
    expect(json['tone'], 0.5);
  });

  test('applyPreset replaces all states and syncs engine', () {
    final engine = FakeEngineRepository();
    final container = makeContainer(engine: engine);
    final notifier = container.read(pedalboardProvider.notifier);

    final pedals = PedalSlot.values.map((s) => PedalState(
      slot: s,
      bypassed: s == PedalSlot.overdrive ? false : true,
      params: Map.from(kDefaultParams[s]!),
    )).toList();
    final preset = Preset(name: 'Lead', pedals: pedals);

    notifier.applyPreset(preset);

    expect(container.read(pedalboardProvider)[2].bypassed, isFalse);
    expect(engine.calls.where((c) => c.startsWith('toggle:')).length, 9);
    expect(engine.calls.where((c) => c.startsWith('setParam:')).length, 9);
  });
}
```

- [ ] **Step 2: Run tests**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/pedalboard_notifier_test.dart
```

Expected: All 5 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add test/pedalboard_notifier_test.dart
git commit -m "test: PedalboardNotifier — toggle, updateParam, applyPreset"
```

---

## Task 6: PresetNotifier unit tests

**Files:**
- Create: `test/preset_notifier_test.dart`

- [ ] **Step 1: Create `test/preset_notifier_test.dart`**

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/domain/models.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/pedalboard_provider.dart';
import 'package:pedaleira/src/providers/preset_provider.dart';

ProviderContainer makeContainer({MemoryPresetRepository? repo}) {
  final memRepo = repo ?? MemoryPresetRepository();
  final container = ProviderContainer(overrides: [
    engineRepositoryProvider.overrideWithValue(FakeEngineRepository()),
    presetRepositoryProvider.overrideWithValue(memRepo),
  ]);
  addTearDown(container.dispose);
  return container;
}

List<PedalState> _defaultPedals() => PedalSlot.values.map((s) => PedalState(
  slot: s,
  bypassed: true,
  params: Map.from(kDefaultParams[s]!),
)).toList();

void main() {
  test('initial state loads from repository', () async {
    final repo = MemoryPresetRepository();
    await repo.save(Preset(name: 'Init', pedals: _defaultPedals()));

    final container = makeContainer(repo: repo);
    final presets = await container.read(presetListProvider.future);

    expect(presets.length, 1);
    expect(presets.first.name, 'Init');
  });

  test('saveCurrentAs adds preset and refreshes list', () async {
    final container = makeContainer();
    await container.read(presetListProvider.future);
    final notifier = container.read(presetListProvider.notifier);

    await notifier.saveCurrentAs('Clean', _defaultPedals());

    final presets = container.read(presetListProvider).value!;
    expect(presets.length, 1);
    expect(presets.first.name, 'Clean');
  });

  test('saveCurrentAs with same name overwrites', () async {
    final container = makeContainer();
    await container.read(presetListProvider.future);
    final notifier = container.read(presetListProvider.notifier);

    await notifier.saveCurrentAs('A', _defaultPedals());
    await notifier.saveCurrentAs('A', _defaultPedals());

    expect(container.read(presetListProvider).value!.length, 1);
  });

  test('delete removes preset from list', () async {
    final container = makeContainer();
    await container.read(presetListProvider.future);
    final notifier = container.read(presetListProvider.notifier);

    await notifier.saveCurrentAs('X', _defaultPedals());
    await notifier.delete('X');

    expect(container.read(presetListProvider).value!, isEmpty);
  });
}
```

- [ ] **Step 2: Run tests**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/preset_notifier_test.dart
```

Expected: All 4 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add test/preset_notifier_test.dart
git commit -m "test: PresetNotifier — load, save, overwrite, delete"
```

---

## Task 7: FilePresetRepository and tests

**Files:**
- Create: `lib/src/data/file_preset_repository.dart`
- Create: `test/file_preset_repository_test.dart`

- [ ] **Step 1: Write the failing test**

Create `test/file_preset_repository_test.dart`:

```dart
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/file_preset_repository.dart';
import 'package:pedaleira/src/domain/models.dart';

List<PedalState> _defaultPedals() => PedalSlot.values.map((s) => PedalState(
  slot: s,
  bypassed: true,
  params: Map.from(kDefaultParams[s]!),
)).toList();

void main() {
  late Directory tempDir;
  late FilePresetRepository repo;

  setUp(() async {
    tempDir = await Directory.systemTemp.createTemp('preset_test_');
    repo = FilePresetRepository(dirOverride: tempDir.path);
  });

  tearDown(() => tempDir.delete(recursive: true));

  test('save and load round-trip', () async {
    final preset = Preset(name: 'Clean', pedals: _defaultPedals());
    await repo.save(preset);

    final loaded = await repo.loadAll();
    expect(loaded.length, 1);
    expect(loaded.first.name, 'Clean');
    expect(loaded.first.pedals.length, 9);
    expect(loaded.first.pedals[1].slot, PedalSlot.compressor);
    expect(loaded.first.pedals[1].params['threshold_db'], -18.0);
  });

  test('save multiple presets', () async {
    await repo.save(Preset(name: 'A', pedals: _defaultPedals()));
    await repo.save(Preset(name: 'B', pedals: _defaultPedals()));

    final loaded = await repo.loadAll();
    expect(loaded.length, 2);
    expect(loaded.map((p) => p.name).toSet(), {'A', 'B'});
  });

  test('save with same name overwrites', () async {
    await repo.save(Preset(name: 'X', pedals: _defaultPedals()));
    final modified = _defaultPedals();
    final updatedPedals = modified.map((p) =>
      p.slot == PedalSlot.delay ? p.copyWith(bypassed: false) : p
    ).toList();
    await repo.save(Preset(name: 'X', pedals: updatedPedals));

    final loaded = await repo.loadAll();
    expect(loaded.length, 1);
    expect(loaded.first.pedals[7].bypassed, isFalse);
  });

  test('delete removes the file', () async {
    await repo.save(Preset(name: 'Gone', pedals: _defaultPedals()));
    await repo.delete('Gone');

    final loaded = await repo.loadAll();
    expect(loaded, isEmpty);
  });

  test('loadAll on empty dir returns empty list', () async {
    final loaded = await repo.loadAll();
    expect(loaded, isEmpty);
  });
}
```

- [ ] **Step 2: Run test to see it fail**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/file_preset_repository_test.dart
```

Expected: FAIL — `FilePresetRepository` not found.

- [ ] **Step 3: Create `lib/src/data/file_preset_repository.dart`**

```dart
import 'dart:convert';
import 'dart:io';
import 'package:path_provider/path_provider.dart';
import '../domain/models.dart';
import '../domain/preset_repository.dart';

class FilePresetRepository implements PresetRepository {
  final String? dirOverride;

  const FilePresetRepository({this.dirOverride});

  Future<Directory> _presetsDir() async {
    final basePath = dirOverride ??
        (await getApplicationDocumentsDirectory()).path;
    final dir = Directory('$basePath/presets');
    if (!await dir.exists()) await dir.create(recursive: true);
    return dir;
  }

  @override
  Future<List<Preset>> loadAll() async {
    final dir = await _presetsDir();
    final files = dir
        .listSync()
        .whereType<File>()
        .where((f) => f.path.endsWith('.json'));
    return files.map((f) {
      final json = jsonDecode(f.readAsStringSync()) as Map<String, dynamic>;
      return Preset.fromJson(json);
    }).toList();
  }

  @override
  Future<void> save(Preset preset) async {
    final dir = await _presetsDir();
    final file = File('${dir.path}/${preset.name}.json');
    await file.writeAsString(jsonEncode(preset.toJson()));
  }

  @override
  Future<void> delete(String name) async {
    final dir = await _presetsDir();
    final file = File('${dir.path}/$name.json');
    if (await file.exists()) await file.delete();
  }
}
```

- [ ] **Step 4: Run tests**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/file_preset_repository_test.dart
```

Expected: All 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add lib/src/data/file_preset_repository.dart test/file_preset_repository_test.dart
git commit -m "feat: FilePresetRepository — path_provider JSON storage + tests"
```

---

## Task 8: App shell and theme

**Files:**
- Create: `lib/src/ui/app.dart`

- [ ] **Step 1: Create `lib/src/ui/app.dart`**

```dart
import 'package:flutter/material.dart';
import 'pedalboard/pedalboard_screen.dart';

class App extends StatelessWidget {
  const App({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Pedaleira',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFF00E5FF),
          surface: Color(0xFF1E1E1E),
          onSurface: Colors.white,
        ),
        scaffoldBackgroundColor: const Color(0xFF111111),
        appBarTheme: const AppBarTheme(
          backgroundColor: Color(0xFF1A1A1A),
          foregroundColor: Colors.white,
          elevation: 0,
        ),
        cardTheme: const CardThemeData(
          color: Color(0xFF1E1E1E),
          elevation: 0,
        ),
      ),
      home: const PedalboardScreen(),
    );
  }
}
```

- [ ] **Step 2: Skip analysis** — `PedalboardScreen` import resolves in Task 13; analyze runs there.

- [ ] **Step 3: Commit**

```bash
git add lib/src/ui/app.dart
git commit -m "feat: App shell with flat dark theme"
```

---

## Task 9: KnobWidget

**Files:**
- Create: `lib/src/ui/editor/knob_widget.dart`

- [ ] **Step 1: Create `lib/src/ui/editor/knob_widget.dart`**

```dart
import 'dart:math';
import 'package:flutter/material.dart';

class KnobWidget extends StatefulWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final ValueChanged<double> onChanged;

  const KnobWidget({
    super.key,
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.onChanged,
  });

  @override
  State<KnobWidget> createState() => _KnobWidgetState();
}

class _KnobWidgetState extends State<KnobWidget> {
  double? _dragStartY;
  double? _dragStartValue;

  @override
  Widget build(BuildContext context) {
    final normalized =
        (widget.value - widget.min) / (widget.max - widget.min);
    final activeColor = Theme.of(context).colorScheme.primary;

    return GestureDetector(
      onPanStart: (d) {
        _dragStartY = d.localPosition.dy;
        _dragStartValue = widget.value;
      },
      onPanUpdate: (d) {
        final delta = (_dragStartY! - d.localPosition.dy) / 150.0;
        final newValue =
            (_dragStartValue! + delta * (widget.max - widget.min))
                .clamp(widget.min, widget.max);
        widget.onChanged(newValue);
      },
      child: SizedBox(
        width: 80,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CustomPaint(
              size: const Size(64, 64),
              painter: _KnobPainter(
                normalized: normalized.clamp(0.0, 1.0),
                activeColor: activeColor,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              widget.label,
              style: const TextStyle(fontSize: 10, color: Colors.grey),
              textAlign: TextAlign.center,
            ),
            Text(
              widget.value.toStringAsFixed(2),
              style: const TextStyle(fontSize: 10, color: Colors.white),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }
}

class _KnobPainter extends CustomPainter {
  final double normalized; // 0.0 – 1.0
  final Color activeColor;

  const _KnobPainter({required this.normalized, required this.activeColor});

  // Arc from 7 o'clock to 5 o'clock (300° sweep), clockwise.
  // Flutter canvas: 0° = East (3 o'clock). Angles increase clockwise.
  // 7 o'clock = 120° = 2π * 120/360
  // Sweep = 300° = 2π * 300/360
  static const double _startRad = 2.0944; // 120° in radians
  static const double _sweepRad = 5.2360; // 300° in radians

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = size.width / 2 - 10;
    final rect = Rect.fromCircle(center: center, radius: radius);

    final trackPaint = Paint()
      ..color = Colors.grey.shade800
      ..style = PaintingStyle.stroke
      ..strokeWidth = 4
      ..strokeCap = StrokeCap.round;

    canvas.drawArc(rect, _startRad, _sweepRad, false, trackPaint);

    if (normalized > 0) {
      final valuePaint = Paint()
        ..color = activeColor
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4
        ..strokeCap = StrokeCap.round;
      canvas.drawArc(rect, _startRad, _sweepRad * normalized, false, valuePaint);
    }

    final dotAngle = _startRad + _sweepRad * normalized;
    final dotOffset = Offset(
      center.dx + radius * cos(dotAngle),
      center.dy + radius * sin(dotAngle),
    );
    canvas.drawCircle(
      dotOffset,
      4,
      Paint()..color = activeColor,
    );
  }

  @override
  bool shouldRepaint(_KnobPainter old) =>
      old.normalized != normalized || old.activeColor != activeColor;
}
```

- [ ] **Step 2: Verify no analysis errors**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter analyze lib/src/ui/editor/knob_widget.dart
```

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add lib/src/ui/editor/knob_widget.dart
git commit -m "feat: KnobWidget — CustomPainter rotary knob with pan gesture"
```

---

## Task 10: PedalTile widget and test

**Files:**
- Create: `lib/src/ui/pedalboard/pedal_tile.dart`
- Create: `test/pedal_tile_test.dart`

- [ ] **Step 1: Create `lib/src/ui/pedalboard/pedal_tile.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../domain/models.dart';
import '../../providers/pedalboard_provider.dart';
import '../editor/pedal_editor_screen.dart';

class PedalTile extends ConsumerWidget {
  final int slot;
  const PedalTile({super.key, required this.slot});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final pedal = ref.watch(
      pedalboardProvider.select((s) => s[slot]),
    );
    final isActive = !pedal.bypassed;
    final theme = Theme.of(context);
    final activeColor = theme.colorScheme.primary;

    return GestureDetector(
      onTap: () => ref.read(pedalboardProvider.notifier).toggleBypass(slot),
      onLongPress: () => _openEditor(context),
      child: Container(
        decoration: BoxDecoration(
          color: theme.colorScheme.surface,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: isActive ? activeColor : Colors.grey.shade800,
            width: isActive ? 1.5 : 1,
          ),
        ),
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Container(
                  width: 10,
                  height: 10,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: isActive ? activeColor : Colors.grey.shade700,
                  ),
                ),
                GestureDetector(
                  onTap: () => _openEditor(context),
                  child: Icon(
                    Icons.settings,
                    size: 16,
                    color: Colors.grey.shade500,
                  ),
                ),
              ],
            ),
            const Spacer(),
            Text(
              kPedalNames[pedal.slot]!,
              style: theme.textTheme.labelMedium?.copyWith(
                color: isActive ? Colors.white : Colors.grey.shade600,
                fontWeight: FontWeight.bold,
                letterSpacing: 0.5,
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _openEditor(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => PedalEditorScreen(slot: slot),
      ),
    );
  }
}
```

- [ ] **Step 2: Create `test/pedal_tile_test.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/pedalboard_provider.dart';
import 'package:pedaleira/src/ui/pedalboard/pedal_tile.dart';

Widget makeTestApp(Widget child, {FakeEngineRepository? engine}) {
  return ProviderScope(
    overrides: [
      engineRepositoryProvider
          .overrideWithValue(engine ?? FakeEngineRepository()),
      presetRepositoryProvider
          .overrideWithValue(MemoryPresetRepository()),
    ],
    child: MaterialApp(
      home: Scaffold(body: SizedBox(width: 160, height: 160, child: child)),
    ),
  );
}

void main() {
  testWidgets('renders pedal name', (tester) async {
    await tester.pumpWidget(makeTestApp(const PedalTile(slot: 0)));
    expect(find.text('Noise Gate'), findsOneWidget);
  });

  testWidgets('tap toggles bypass and calls engine', (tester) async {
    final engine = FakeEngineRepository();
    await tester.pumpWidget(makeTestApp(const PedalTile(slot: 0), engine: engine));

    await tester.tap(find.byType(PedalTile));
    await tester.pump();

    expect(engine.calls, contains('toggle:0:false'));
  });

  testWidgets('tap twice returns to bypassed', (tester) async {
    final engine = FakeEngineRepository();
    await tester.pumpWidget(makeTestApp(const PedalTile(slot: 1), engine: engine));

    await tester.tap(find.byType(PedalTile));
    await tester.pump();
    await tester.tap(find.byType(PedalTile));
    await tester.pump();

    expect(engine.calls.last, 'toggle:1:true');
  });

  testWidgets('long-press navigates to editor screen', (tester) async {
    await tester.pumpWidget(makeTestApp(const PedalTile(slot: 2)));

    await tester.longPress(find.byType(PedalTile));
    await tester.pumpAndSettle();

    // PedalEditorScreen shows the pedal name in AppBar
    expect(find.text('Overdrive'), findsWidgets);
  });
}
```

- [ ] **Step 3: Run tests**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/pedal_tile_test.dart
```

Expected: All 4 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add lib/src/ui/pedalboard/pedal_tile.dart test/pedal_tile_test.dart
git commit -m "feat: PedalTile — LED, bypass toggle, long-press nav + tests"
```

---

## Task 11: PedalEditorScreen and test

**Files:**
- Create: `lib/src/ui/editor/pedal_editor_screen.dart`
- Create: `test/pedal_editor_screen_test.dart`

- [ ] **Step 1: Create `lib/src/ui/editor/pedal_editor_screen.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../domain/models.dart';
import '../../providers/pedalboard_provider.dart';
import 'knob_widget.dart';

class PedalEditorScreen extends ConsumerWidget {
  final int slot;
  const PedalEditorScreen({super.key, required this.slot});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final pedal = ref.watch(
      pedalboardProvider.select((s) => s[slot]),
    );
    return Scaffold(
      appBar: AppBar(title: Text(kPedalNames[pedal.slot]!)),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(32),
        child: Wrap(
          spacing: 32,
          runSpacing: 32,
          children: pedal.params.entries.map((entry) {
            final range = kParamRanges[entry.key] ?? (0.0, 1.0);
            return KnobWidget(
              label: entry.key,
              value: entry.value,
              min: range.$1,
              max: range.$2,
              onChanged: (v) => ref
                  .read(pedalboardProvider.notifier)
                  .updateParam(slot, entry.key, v),
            );
          }).toList(),
        ),
      ),
    );
  }
}
```

- [ ] **Step 2: Create `test/pedal_editor_screen_test.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/domain/models.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/pedalboard_provider.dart';
import 'package:pedaleira/src/ui/editor/pedal_editor_screen.dart';
import 'package:pedaleira/src/ui/editor/knob_widget.dart';

Widget makeTestApp(int slot) {
  return ProviderScope(
    overrides: [
      engineRepositoryProvider.overrideWithValue(FakeEngineRepository()),
      presetRepositoryProvider.overrideWithValue(MemoryPresetRepository()),
    ],
    child: MaterialApp(home: PedalEditorScreen(slot: slot)),
  );
}

void main() {
  testWidgets('shows pedal name in app bar', (tester) async {
    await tester.pumpWidget(makeTestApp(1));
    expect(find.text('Compressor'), findsOneWidget);
  });

  testWidgets('renders one KnobWidget per compressor param', (tester) async {
    // Compressor has 4 params: threshold_db, ratio, attack, release
    await tester.pumpWidget(makeTestApp(1));
    expect(find.byType(KnobWidget), findsNWidgets(4));
  });

  testWidgets('renders one KnobWidget per overdrive param', (tester) async {
    // Overdrive has 2 params: drive, tone
    await tester.pumpWidget(makeTestApp(2));
    expect(find.byType(KnobWidget), findsNWidgets(2));
  });

  testWidgets('updateParam is called when knob changes', (tester) async {
    final engine = FakeEngineRepository();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          engineRepositoryProvider.overrideWithValue(engine),
          presetRepositoryProvider.overrideWithValue(MemoryPresetRepository()),
        ],
        child: MaterialApp(home: PedalEditorScreen(slot: 2)),
      ),
    );

    // Directly call updateParam through the notifier
    final container = ProviderScope.containerOf(
      tester.element(find.byType(PedalEditorScreen)),
    );
    container.read(pedalboardProvider.notifier).updateParam(2, 'drive', 5.0);
    await tester.pump();

    expect(engine.calls.any((c) => c.startsWith('setParam:2:')), isTrue);
  });
}
```

- [ ] **Step 3: Run tests**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/pedal_editor_screen_test.dart
```

Expected: All 4 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add lib/src/ui/editor/pedal_editor_screen.dart test/pedal_editor_screen_test.dart
git commit -m "feat: PedalEditorScreen — knob grid per pedal + tests"
```

---

## Task 12: PresetBar and test

**Files:**
- Create: `lib/src/ui/preset_bar.dart`
- Create: `test/preset_bar_test.dart`

- [ ] **Step 1: Create `lib/src/ui/preset_bar.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../domain/models.dart';
import '../providers/pedalboard_provider.dart';
import '../providers/preset_provider.dart';

class PresetBar extends ConsumerWidget {
  const PresetBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final presetsAsync = ref.watch(presetListProvider);
    final activeIndex = ref.watch(activePresetIndexProvider);
    final pedalboard = ref.watch(pedalboardProvider);

    return presetsAsync.when(
      data: (presets) {
        final hasPresets = presets.isNotEmpty;
        final clampedIndex = hasPresets
            ? activeIndex.clamp(0, presets.length - 1)
            : 0;
        final name = hasPresets ? presets[clampedIndex].name : '—';

        return Row(
          children: [
            IconButton(
              icon: const Icon(Icons.chevron_left),
              onPressed: hasPresets && activeIndex > 0
                  ? () => _navigate(ref, presets, activeIndex - 1)
                  : null,
            ),
            Expanded(
              child: Text(
                name,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  fontWeight: FontWeight.bold,
                  fontSize: 14,
                ),
              ),
            ),
            IconButton(
              icon: const Icon(Icons.chevron_right),
              onPressed: hasPresets && activeIndex < presets.length - 1
                  ? () => _navigate(ref, presets, activeIndex + 1)
                  : null,
            ),
            TextButton(
              onPressed: () => _save(context, ref, presets, activeIndex, pedalboard),
              child: const Text('Save'),
            ),
          ],
        );
      },
      loading: () => const SizedBox.shrink(),
      error: (_, __) => const SizedBox.shrink(),
    );
  }

  void _navigate(WidgetRef ref, List<Preset> presets, int idx) {
    ref.read(activePresetIndexProvider.notifier).state = idx;
    ref.read(pedalboardProvider.notifier).applyPreset(presets[idx]);
  }

  Future<void> _save(
    BuildContext context,
    WidgetRef ref,
    List<Preset> presets,
    int activeIndex,
    List<PedalState> pedalboard,
  ) async {
    final hasPresets = presets.isNotEmpty;
    if (hasPresets) {
      final currentName = presets[activeIndex.clamp(0, presets.length - 1)].name;
      await ref
          .read(presetListProvider.notifier)
          .saveCurrentAs(currentName, pedalboard);
    } else {
      if (!context.mounted) return;
      final name = await _promptName(context);
      if (name != null && name.isNotEmpty && context.mounted) {
        await ref
            .read(presetListProvider.notifier)
            .saveCurrentAs(name, pedalboard);
      }
    }
  }

  Future<String?> _promptName(BuildContext context) {
    final controller = TextEditingController();
    return showDialog<String>(
      context: context,
      builder: (_) => AlertDialog(
        title: const Text('Save Preset'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(labelText: 'Name'),
          autofocus: true,
          onSubmitted: (v) => Navigator.pop(context, v.trim()),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () =>
                Navigator.pop(context, controller.text.trim()),
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }
}
```

- [ ] **Step 2: Create `test/preset_bar_test.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/domain/models.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/pedalboard_provider.dart';
import 'package:pedaleira/src/providers/preset_provider.dart';
import 'package:pedaleira/src/ui/preset_bar.dart';

List<PedalState> _defaultPedals() => PedalSlot.values.map((s) => PedalState(
  slot: s,
  bypassed: true,
  params: Map.from(kDefaultParams[s]!),
)).toList();

Widget makeTestApp({MemoryPresetRepository? repo, FakeEngineRepository? engine}) {
  return ProviderScope(
    overrides: [
      engineRepositoryProvider
          .overrideWithValue(engine ?? FakeEngineRepository()),
      presetRepositoryProvider
          .overrideWithValue(repo ?? MemoryPresetRepository()),
    ],
    child: const MaterialApp(
      home: Scaffold(body: PresetBar()),
    ),
  );
}

void main() {
  testWidgets('shows — when no presets', (tester) async {
    await tester.pumpWidget(makeTestApp());
    await tester.pumpAndSettle();
    expect(find.text('—'), findsOneWidget);
  });

  testWidgets('shows preset name when presets exist', (tester) async {
    final repo = MemoryPresetRepository();
    await repo.save(Preset(name: 'Clean', pedals: _defaultPedals()));

    await tester.pumpWidget(makeTestApp(repo: repo));
    await tester.pumpAndSettle();

    expect(find.text('Clean'), findsOneWidget);
  });

  testWidgets('> button navigates to next preset', (tester) async {
    final repo = MemoryPresetRepository();
    await repo.save(Preset(name: 'A', pedals: _defaultPedals()));
    await repo.save(Preset(name: 'B', pedals: _defaultPedals()));

    await tester.pumpWidget(makeTestApp(repo: repo));
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.chevron_right));
    await tester.pumpAndSettle();

    // After navigating forward, the second preset name is shown
    final presets = await repo.loadAll();
    final names = presets.map((p) => p.name).toList();
    expect(find.text(names[1]), findsOneWidget);
  });

  testWidgets('save overwrites active preset when presets exist', (tester) async {
    final repo = MemoryPresetRepository();
    await repo.save(Preset(name: 'X', pedals: _defaultPedals()));

    await tester.pumpWidget(makeTestApp(repo: repo));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();

    // Still only one preset (overwrite, no new)
    expect((await repo.loadAll()).length, 1);
  });
}
```

- [ ] **Step 3: Run tests**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test test/preset_bar_test.dart
```

Expected: All 4 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add lib/src/ui/preset_bar.dart test/preset_bar_test.dart
git commit -m "feat: PresetBar — preset navigation, save dialog + tests"
```

---

## Task 13: PedalboardScreen

**Files:**
- Create: `lib/src/ui/pedalboard/pedalboard_screen.dart`

- [ ] **Step 1: Create `lib/src/ui/pedalboard/pedalboard_screen.dart`**

```dart
import 'package:flutter/material.dart';
import 'pedal_tile.dart';
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
          final crossAxisCount =
              orientation == Orientation.portrait ? 2 : 4;
          return Column(
            children: [
              Expanded(
                child: GridView.builder(
                  padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: crossAxisCount,
                    crossAxisSpacing: 12,
                    mainAxisSpacing: 12,
                    childAspectRatio: 1.2,
                  ),
                  itemCount: 8,
                  itemBuilder: (_, i) => PedalTile(slot: i),
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                child: SizedBox(
                  height: 72,
                  child: PedalTile(slot: 8),
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}
```

- [ ] **Step 2: Verify no analysis errors**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter analyze lib/src/ui/
```

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add lib/src/ui/pedalboard/pedalboard_screen.dart
git commit -m "feat: PedalboardScreen — adaptive 2x4/4x2 grid + reverb strip"
```

---

## Task 14: Wire main.dart and fix widget_test

**Files:**
- Modify: `lib/main.dart`
- Modify: `test/widget_test.dart`

- [ ] **Step 1: Replace `lib/main.dart`**

`engineRepositoryProvider` and `presetRepositoryProvider` already default to the fake/memory impls in `engine_provider.dart`, so no overrides are needed here yet. Task 15 replaces this with the real engine.

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'src/rust/frb_generated.dart';
import 'src/ui/app.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const ProviderScope(child: App()));
}
```

- [ ] **Step 2: Replace `test/widget_test.dart`**

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/pedalboard_provider.dart';
import 'package:pedaleira/src/ui/app.dart';

void main() {
  testWidgets('app renders pedalboard screen', (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          engineRepositoryProvider
              .overrideWithValue(FakeEngineRepository()),
          presetRepositoryProvider
              .overrideWithValue(MemoryPresetRepository()),
        ],
        child: const App(),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Pedaleira'), findsOneWidget);
    expect(find.text('Noise Gate'), findsOneWidget);
    expect(find.text('Reverb'), findsOneWidget);
  });
}
```

- [ ] **Step 3: Run all tests**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test
```

Expected: All tests PASS (models, pedalboard_notifier, preset_notifier, file_preset_repository, pedal_tile, pedal_editor_screen, preset_bar, widget_test).

- [ ] **Step 4: Commit**

```bash
git add lib/main.dart test/widget_test.dart
git commit -m "feat: wire main.dart + update smoke test"
```

---

## Task 15: RustEngineRepository (bridge wiring)

**Files:**
- Create: `lib/src/data/rust_engine_repository.dart`
- Modify: `lib/main.dart`

> **Prerequisite:** Requires a WAV input file. The sample audio `sample_audios/freesound_community-electric-guitar-riff-jingle-83860.mp3` is MP3 and cannot be used directly — convert to WAV first (e.g. `ffmpeg -i sample_audios/*.mp3 sample_audios/guitar_di.wav`). Alternatively, configure a virtual audio cable (see CLAUDE.md Phase 2 setup) and pass an appropriate WAV path.

- [ ] **Step 1: Create `lib/src/data/rust_engine_repository.dart`**

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
}
```

- [ ] **Step 2: Update `lib/main.dart` to use the real engine**

Replace the import of `fake_engine_repository.dart` and update `main()`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'src/data/rust_engine_repository.dart';
import 'src/providers/engine_provider.dart';
import 'src/providers/pedalboard_provider.dart';
import 'src/rust/frb_generated.dart';
import 'src/ui/app.dart';

const _kWavPath = 'sample_audios/guitar_di.wav';

Future<void> main() async {
  await RustLib.init();
  final engine = RustEngineRepository();
  engine.start(_kWavPath);
  runApp(ProviderScope(
    overrides: [
      engineRepositoryProvider.overrideWithValue(engine),
    ],
    child: const App(),
  ));
}
```

- [ ] **Step 3: Verify analysis passes**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter analyze lib/
```

Expected: No errors.

- [ ] **Step 4: Run all tests (engine is still faked in tests)**

```bash
cd /home/arlan/ai-tests/pedaleira && flutter test
```

Expected: All tests PASS — `main.dart` changes do not affect test overrides.

- [ ] **Step 5: Commit**

```bash
git add lib/src/data/rust_engine_repository.dart lib/main.dart
git commit -m "feat: RustEngineRepository — wire flutter_rust_bridge to engine interface"
```

---

## Done

All Phase 3 items from `progress.md` are implemented:

- [x] App state + data models (Riverpod)
- [x] Main pedalboard screen (2×4 / 4×2 adaptive grid)
- [x] Pedal tile widget (bypass toggle + long-press to edit)
- [x] Rotary knob widget (CustomPainter + pan gesture)
- [x] Pedal editor screen (full-screen knobs)
- [x] Preset navigation bar (< name >)
- [x] Preset load/save persistence (JSON files)
- [x] Widget tests updated
