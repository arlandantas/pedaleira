import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
import 'package:pedaleira/src/providers/mute_provider.dart';

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
  test('muteProvider initial state is false', () {
    final container = makeContainer();
    expect(container.read(muteProvider), isFalse);
  });

  test('toggle sets muted=true and calls engine setMute(true)', () {
    final engine = FakeEngineRepository();
    final container = makeContainer(engine: engine);
    container.read(muteProvider.notifier).toggle();
    expect(container.read(muteProvider), isTrue);
    expect(engine.calls, contains('setMute:true'));
  });

  test('toggle twice restores unmuted and calls setMute(false)', () {
    final engine = FakeEngineRepository();
    final container = makeContainer(engine: engine);
    container.read(muteProvider.notifier).toggle();
    container.read(muteProvider.notifier).toggle();
    expect(container.read(muteProvider), isFalse);
    expect(engine.calls, contains('setMute:false'));
  });
}
