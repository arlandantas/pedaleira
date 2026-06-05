# Preset Share & Import Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Import and Export icon buttons to the AppBar so users can exchange preset `.json` files via the native share sheet (Android) or file manager (Linux).

**Architecture:** Pure helper functions for JSON parsing and name conflict resolution live in a new `preset_io.dart` domain file. The platform calls (`share_plus`, `file_picker`) live as private methods on `PedalboardScreen`, converted to a `ConsumerStatefulWidget` to safely use `BuildContext` after async gaps. No new providers.

**Tech Stack:** Flutter/Dart, Riverpod, `share_plus ^13.0.0`, `file_picker ^11.0.0`, `path_provider` (already present)

---

### Task 1: Add packages

**Files:**
- Modify: `pubspec.yaml`

- [ ] **Step 1: Add dependencies**

In `pubspec.yaml`, under `dependencies:`, add after `path_provider`:

```yaml
  share_plus: ^13.0.0
  file_picker: ^11.0.0
```

- [ ] **Step 2: Fetch packages**

```bash
flutter pub get
```

Expected: resolves without error, lock file updated.

- [ ] **Step 3: Commit**

```bash
git add pubspec.yaml pubspec.lock
git commit -m "feat: add share_plus and file_picker packages"
```

---

### Task 2: Add pure preset I/O helpers

**Files:**
- Create: `lib/src/domain/preset_io.dart`
- Create: `test/preset_io_test.dart`

- [ ] **Step 1: Write failing tests**

Create `test/preset_io_test.dart`:

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/domain/models.dart';
import 'package:pedaleira/src/domain/preset_io.dart';

void main() {
  group('parsePresetJson', () {
    test('returns Preset for valid JSON', () {
      final preset = Preset(
        name: 'Test',
        pedals: PedalSlot.values.map((s) => PedalState(
          slot: s,
          bypassed: false,
          params: Map.from(kDefaultParams[s]!),
        )).toList(),
      );
      final json = preset.toJsonString();
      final result = parsePresetJson(json);
      expect(result, isNotNull);
      expect(result!.name, 'Test');
    });

    test('returns null for invalid JSON', () {
      expect(parsePresetJson('not json'), isNull);
    });

    test('returns null for JSON missing required fields', () {
      expect(parsePresetJson('{"foo": 1}'), isNull);
    });
  });

  group('resolveImportName', () {
    test('returns name unchanged when no conflict', () {
      expect(resolveImportName('Clean', ['Crunch', 'Lead']), 'Clean');
    });

    test('appends - imported when name already exists', () {
      expect(resolveImportName('Clean', ['Clean', 'Lead']), 'Clean - imported');
    });

    test('returns name unchanged for empty list', () {
      expect(resolveImportName('Clean', []), 'Clean');
    });
  });
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
flutter test test/preset_io_test.dart
```

Expected: compile error — `preset_io.dart` does not exist yet.

- [ ] **Step 3: Create `lib/src/domain/preset_io.dart`**

```dart
import 'dart:convert';
import 'models.dart';

/// Returns null if [json] is not a valid Preset.
Preset? parsePresetJson(String json) {
  try {
    final map = jsonDecode(json) as Map<String, dynamic>;
    return Preset.fromJson(map);
  } catch (_) {
    return null;
  }
}

/// Returns [name] if not in [existing]; otherwise returns '[name] - imported'.
String resolveImportName(String name, List<String> existing) {
  if (!existing.contains(name)) return name;
  return '$name - imported';
}
```

- [ ] **Step 4: Add `toJsonString()` to `Preset` in `lib/src/domain/models.dart`**

In `models.dart`, inside the `Preset` class after `toJson()`:

```dart
  String toJsonString() => jsonEncode(toJson());
```

Also add the import at the top of `models.dart` if not present:

```dart
import 'dart:convert';
```

- [ ] **Step 5: Run tests to confirm they pass**

```bash
flutter test test/preset_io_test.dart
```

Expected: all 6 tests pass.

- [ ] **Step 6: Commit**

```bash
git add lib/src/domain/preset_io.dart lib/src/domain/models.dart test/preset_io_test.dart
git commit -m "feat: add preset_io helpers (parsePresetJson, resolveImportName)"
```

---

### Task 3: Convert PedalboardScreen and add AppBar buttons

**Files:**
- Modify: `lib/src/ui/pedalboard/pedalboard_screen.dart`
- Create: `test/pedalboard_screen_test.dart`

- [ ] **Step 1: Write failing widget tests**

Create `test/pedalboard_screen_test.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/domain/models.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/preset_provider.dart';
import 'package:pedaleira/src/ui/pedalboard/pedalboard_screen.dart';

List<PedalState> _pedals() => PedalSlot.values.map((s) => PedalState(
  slot: s, bypassed: true, params: Map.from(kDefaultParams[s]!),
)).toList();

Widget _makeApp({MemoryPresetRepository? repo}) {
  return ProviderScope(
    overrides: [
      engineRepositoryProvider.overrideWithValue(FakeEngineRepository()),
      presetRepositoryProvider
          .overrideWithValue(repo ?? MemoryPresetRepository()),
    ],
    child: const MaterialApp(home: PedalboardScreen()),
  );
}

void main() {
  testWidgets('import button is always present', (tester) async {
    await tester.pumpWidget(_makeApp());
    await tester.pumpAndSettle();
    expect(find.byIcon(Icons.upload_file), findsOneWidget);
  });

  testWidgets('export button is present', (tester) async {
    await tester.pumpWidget(_makeApp());
    await tester.pumpAndSettle();
    expect(find.byIcon(Icons.ios_share), findsOneWidget);
  });

  testWidgets('export button is disabled when no presets', (tester) async {
    await tester.pumpWidget(_makeApp());
    await tester.pumpAndSettle();
    final btn = tester.widget<IconButton>(find.widgetWithIcon(IconButton, Icons.ios_share));
    expect(btn.onPressed, isNull);
  });

  testWidgets('export button is enabled when presets exist', (tester) async {
    final repo = MemoryPresetRepository();
    await repo.save(Preset(name: 'Clean', pedals: _pedals()));
    await tester.pumpWidget(_makeApp(repo: repo));
    await tester.pumpAndSettle();
    final btn = tester.widget<IconButton>(find.widgetWithIcon(IconButton, Icons.ios_share));
    expect(btn.onPressed, isNotNull);
  });
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
flutter test test/pedalboard_screen_test.dart
```

Expected: compile error or test failures — buttons don't exist yet.

- [ ] **Step 3: Convert `PedalboardScreen` to `ConsumerStatefulWidget` and add buttons**

Replace the entire content of `lib/src/ui/pedalboard/pedalboard_screen.dart`:

```dart
import 'dart:convert';
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';
import '../../domain/preset_io.dart';
import '../../providers/preset_provider.dart';
import 'pedal_tile.dart';
import 'mute_bar.dart';
import '../preset_bar.dart';

class PedalboardScreen extends ConsumerStatefulWidget {
  const PedalboardScreen({super.key});

  @override
  ConsumerState<PedalboardScreen> createState() => _PedalboardScreenState();
}

class _PedalboardScreenState extends ConsumerState<PedalboardScreen> {
  @override
  Widget build(BuildContext context) {
    final presetsAsync = ref.watch(presetListProvider);
    final hasPresets = presetsAsync.valueOrNull?.isNotEmpty ?? false;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Pedaleira'),
        actions: [
          IconButton(
            icon: const Icon(Icons.upload_file),
            tooltip: 'Import preset',
            onPressed: _importPreset,
          ),
          IconButton(
            icon: const Icon(Icons.ios_share),
            tooltip: 'Export preset',
            onPressed: hasPresets ? _exportPreset : null,
          ),
        ],
        bottom: const PreferredSize(
          preferredSize: Size.fromHeight(48),
          child: PresetBar(),
        ),
      ),
      body: OrientationBuilder(
        builder: (context, orientation) {
          final crossAxisCount = orientation == Orientation.portrait ? 2 : 4;
          final rowCount = orientation == Orientation.portrait ? 4 : 2;
          return LayoutBuilder(
            builder: (context, constraints) {
              const reverbRowHeight = 72.0;
              const reverbRowBottomPad = 8.0;
              const muteBarHeight = 56.0;
              const muteBarBottomPad = 16.0;
              const gridPadTop = 16.0;
              const gridPadBottom = 8.0;
              const tileSpacing = 12.0;
              final gridContentHeight = constraints.maxHeight
                  - reverbRowHeight
                  - reverbRowBottomPad
                  - muteBarHeight
                  - muteBarBottomPad
                  - gridPadTop
                  - gridPadBottom;
              final tileHeight =
                  (gridContentHeight - tileSpacing * (rowCount - 1)) / rowCount;
              return Column(
                children: [
                  Expanded(
                    child: GridView.builder(
                      padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                      physics: const NeverScrollableScrollPhysics(),
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: crossAxisCount,
                        crossAxisSpacing: 12,
                        mainAxisSpacing: 12,
                        mainAxisExtent: tileHeight,
                      ),
                      itemCount: 8,
                      itemBuilder: (_, i) => PedalTile(slot: i),
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 0, 16, reverbRowBottomPad),
                    child: SizedBox(
                      height: reverbRowHeight,
                      child: Row(
                        children: [
                          Expanded(child: PedalTile(slot: 8)),
                          const SizedBox(width: 12),
                          Expanded(child: PedalTile(slot: 9)),
                        ],
                      ),
                    ),
                  ),
                  const MuteBar(),
                ],
              );
            },
          );
        },
      ),
    );
  }

  Future<void> _exportPreset() async {
    final presets = ref.read(presetListProvider).valueOrNull ?? [];
    final idx = ref.read(activePresetIndexProvider);
    if (presets.isEmpty) return;
    final preset = presets[idx.clamp(0, presets.length - 1)];

    final dir = await getTemporaryDirectory();
    final file = File('${dir.path}/${preset.name}.json');
    await file.writeAsString(preset.toJsonString());
    await SharePlus.instance.shareXFiles(
      [XFile(file.path)],
      subject: preset.name,
    );
    await file.delete();
  }

  Future<void> _importPreset() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['json'],
      withData: true,
    );
    if (result == null || !mounted) return;

    final pickedFile = result.files.single;
    final String jsonString;
    if (pickedFile.bytes != null) {
      jsonString = utf8.decode(pickedFile.bytes!);
    } else if (pickedFile.path != null) {
      jsonString = await File(pickedFile.path!).readAsString();
    } else {
      _showSnackBar('Could not read file.');
      return;
    }

    final preset = parsePresetJson(jsonString);
    if (!mounted) return;
    if (preset == null) {
      _showSnackBar('Invalid preset file.');
      return;
    }

    final existing = (ref.read(presetListProvider).valueOrNull ?? [])
        .map((p) => p.name)
        .toList();

    final String finalName;
    if (existing.contains(preset.name)) {
      final overwrite = await showDialog<bool>(
        context: context,
        builder: (_) => AlertDialog(
          title: const Text('Name conflict'),
          content: Text(
            'A preset named "${preset.name}" already exists.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Save as copy'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('Overwrite'),
            ),
          ],
        ),
      );
      if (!mounted) return;
      if (overwrite == null) return;
      finalName = overwrite
          ? preset.name
          : resolveImportName(preset.name, existing);
    } else {
      finalName = preset.name;
    }

    await ref.read(presetListProvider.notifier).saveCurrentAs(
          finalName,
          preset.pedals,
        );

    if (!mounted) return;
    final updated = ref.read(presetListProvider).valueOrNull ?? [];
    final newIdx = updated.indexWhere((p) => p.name == finalName);
    if (newIdx >= 0) {
      ref.read(activePresetIndexProvider.notifier).state = newIdx;
    }
    _showSnackBar('Preset "${finalName}" imported.');
  }

  void _showSnackBar(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }
}
```

- [ ] **Step 4: Run widget tests**

```bash
flutter test test/pedalboard_screen_test.dart
```

Expected: all 4 tests pass.

- [ ] **Step 5: Run full test suite**

```bash
flutter test
```

Expected: all tests pass (30 existing + 10 new).

- [ ] **Step 6: Commit**

```bash
git add lib/src/ui/pedalboard/pedalboard_screen.dart test/pedalboard_screen_test.dart
git commit -m "feat: add import/export preset buttons to AppBar"
```
