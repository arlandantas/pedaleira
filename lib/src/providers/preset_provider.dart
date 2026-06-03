import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../domain/models.dart';
import 'engine_provider.dart';

class PresetNotifier extends AsyncNotifier<List<Preset>> {
  @override
  Future<List<Preset>> build() async {
    return ref.read(presetRepositoryProvider).loadAll();
  }

  Future<void> saveCurrentAs(String name, List<PedalState> pedals) async {
    final preset = Preset(name: name, pedals: pedals);
    await ref.read(presetRepositoryProvider).save(preset);
    state = AsyncValue.data(
      await ref.read(presetRepositoryProvider).loadAll(),
    );
  }

  Future<void> delete(String name) async {
    await ref.read(presetRepositoryProvider).delete(name);
    state = AsyncValue.data(
      state.value?.where((p) => p.name != name).toList() ?? [],
    );
  }

  Future<void> duplicatePreset(
    String currentName,
    List<PedalState> pedals,
  ) async {
    final copyName = '$currentName - copy';
    final copy = Preset(name: copyName, pedals: pedals);
    await ref.read(presetRepositoryProvider).save(copy);
    final updated = await ref.read(presetRepositoryProvider).loadAll();
    state = AsyncValue.data(updated);
    final idx = updated.indexWhere((p) => p.name == copyName);
    if (idx >= 0) {
      ref.read(activePresetIndexProvider.notifier).state = idx;
    }
  }

  Future<void> renamePreset(
    String oldName,
    String newName,
    List<PedalState> pedals,
  ) async {
    await ref.read(presetRepositoryProvider).delete(oldName);
    final renamed = Preset(name: newName, pedals: pedals);
    await ref.read(presetRepositoryProvider).save(renamed);
    final updated = await ref.read(presetRepositoryProvider).loadAll();
    state = AsyncValue.data(updated);
    final idx = updated.indexWhere((p) => p.name == newName);
    if (idx >= 0) {
      ref.read(activePresetIndexProvider.notifier).state = idx;
    }
  }
}

final presetListProvider =
    AsyncNotifierProvider<PresetNotifier, List<Preset>>(
  PresetNotifier.new,
);

final activePresetIndexProvider = StateProvider<int>((ref) => 0);
