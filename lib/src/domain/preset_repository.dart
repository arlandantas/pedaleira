import '../domain/models.dart';

abstract class PresetRepository {
  Future<List<Preset>> loadAll();
  Future<void> save(Preset preset);
  Future<void> delete(String name);
}
