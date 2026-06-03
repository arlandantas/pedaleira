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

    expect(find.text('Overdrive'), findsWidgets);
  });
}
