import '../domain/models.dart';
import '../domain/preset_repository.dart';

class MemoryPresetRepository implements PresetRepository {
  final Map<String, Preset> _store = {};

  @override
  Future<List<Preset>> loadAll() async => _store.values.toList();

  @override
  Future<void> save(Preset preset) async => _store[preset.name] = preset;

  @override
  Future<void> delete(String name) async => _store.remove(name);
}
