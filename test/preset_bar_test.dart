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

    expect((await repo.loadAll()).length, 1);
  });
}
