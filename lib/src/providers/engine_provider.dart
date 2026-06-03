import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../data/fake_engine_repository.dart';
import '../data/memory_preset_repository.dart';
import '../domain/engine_repository.dart';
import '../domain/preset_repository.dart';

// Override these in main.dart (real impls) and tests (fakes).
final engineRepositoryProvider = Provider<EngineRepository>(
  (ref) => FakeEngineRepository(),
);

final presetRepositoryProvider = Provider<PresetRepository>(
  (ref) => MemoryPresetRepository(),
);
