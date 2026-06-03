import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/data/fake_engine_repository.dart';
import 'package:pedaleira/src/data/memory_preset_repository.dart';
import 'package:pedaleira/src/domain/models.dart';
import 'package:pedaleira/src/providers/engine_provider.dart';
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

  test('duplicatePreset saves copy with " - copy" suffix and navigates to it', () async {
    final container = makeContainer();
    await container.read(presetListProvider.future);
    final notifier = container.read(presetListProvider.notifier);

    await notifier.saveCurrentAs('Lead', _defaultPedals());
    await notifier.duplicatePreset('Lead', _defaultPedals());

    final presets = container.read(presetListProvider).value!;
    expect(presets.length, 2);
    expect(presets.any((p) => p.name == 'Lead - copy'), isTrue);
    expect(container.read(activePresetIndexProvider), 1);
  });

  test('renamePreset replaces old name with new name and navigates to it', () async {
    final container = makeContainer();
    await container.read(presetListProvider.future);
    final notifier = container.read(presetListProvider.notifier);

    await notifier.saveCurrentAs('Old', _defaultPedals());
    await notifier.renamePreset('Old', 'New', _defaultPedals());

    final presets = container.read(presetListProvider).value!;
    expect(presets.length, 1);
    expect(presets.first.name, 'New');
    expect(container.read(activePresetIndexProvider), 0);
  });
}
