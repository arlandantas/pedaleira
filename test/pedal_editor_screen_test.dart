import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/pedalboard_provider.dart';
import 'package:pedaleira/src/ui/editor/knob_widget.dart';
import 'package:pedaleira/src/ui/editor/pedal_editor_screen.dart';

Widget makeTestApp(int slot, {FakeEngineRepository? engine}) {
  return ProviderScope(
    overrides: [
      engineRepositoryProvider
          .overrideWithValue(engine ?? FakeEngineRepository()),
      presetRepositoryProvider
          .overrideWithValue(MemoryPresetRepository()),
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

  testWidgets('updateParam is called when notifier is triggered', (tester) async {
    final engine = FakeEngineRepository();
    await tester.pumpWidget(makeTestApp(2, engine: engine));

    final container = ProviderScope.containerOf(
      tester.element(find.byType(PedalEditorScreen)),
    );
    container.read(pedalboardProvider.notifier).updateParam(2, 'drive', 5.0);
    await tester.pump();

    expect(engine.calls.any((c) => c.startsWith('setParam:2:')), isTrue);
  });

  testWidgets('shows Enabled switch, initially off when pedal is bypassed', (tester) async {
    await tester.pumpWidget(makeTestApp(0)); // slot 0 starts bypassed
    final switchFinder = find.byType(Switch);
    expect(switchFinder, findsOneWidget);
    final sw = tester.widget<Switch>(switchFinder);
    expect(sw.value, isFalse); // bypassed=true → switch OFF
  });

  testWidgets('tapping Enabled switch calls toggleBypass on engine', (tester) async {
    final engine = FakeEngineRepository();
    await tester.pumpWidget(makeTestApp(0, engine: engine));
    await tester.tap(find.byType(Switch));
    await tester.pump();
    expect(engine.calls, contains('toggle:0:false'));
  });
}
