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
