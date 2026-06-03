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
    final updatedPedals = _defaultPedals().map((p) =>
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
