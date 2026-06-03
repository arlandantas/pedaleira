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
}

final presetListProvider =
    AsyncNotifierProvider<PresetNotifier, List<Preset>>(
  PresetNotifier.new,
);

final activePresetIndexProvider = StateProvider<int>((ref) => 0);
