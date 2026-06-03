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
