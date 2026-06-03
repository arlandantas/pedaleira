import '../domain/engine_repository.dart';
import '../rust/api/engine_api.dart' as bridge;

class RustEngineRepository implements EngineRepository {
  @override
  void start(String wavPath) {
    bridge.startEngine(
      wavPath: wavPath,
      playOutput: true,
      writeOutput: false,
      outputPath: '',
    );
  }

  @override
  void stop() => bridge.stopEngine();

  @override
  void toggleBypass(int slot, bool bypassed) =>
      bridge.toggleBypass(slot: slot, bypass: bypassed);

  @override
  void setParam(int slot, String json) =>
      bridge.setParam(slot: slot, json: json);
}
