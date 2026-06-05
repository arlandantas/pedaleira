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
    final btn = tester.widget<IconButton>(
      find.widgetWithIcon(IconButton, Icons.ios_share),
    );
    expect(btn.onPressed, isNull);
  });

  testWidgets('export button is enabled when presets exist', (tester) async {
    final repo = MemoryPresetRepository();
    await repo.save(Preset(name: 'Clean', pedals: _pedals()));
    await tester.pumpWidget(_makeApp(repo: repo));
    await tester.pumpAndSettle();
    final btn = tester.widget<IconButton>(
      find.widgetWithIcon(IconButton, Icons.ios_share),
    );
    expect(btn.onPressed, isNotNull);
  });
}
