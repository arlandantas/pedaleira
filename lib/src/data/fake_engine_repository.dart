import '../domain/engine_repository.dart';

class FakeEngineRepository implements EngineRepository {
  final List<String> calls = [];

  @override
  void start(String wavPath) => calls.add('start:$wavPath');

  @override
  void stop() => calls.add('stop');

  @override
  void toggleBypass(int slot, bool bypassed) =>
      calls.add('toggle:$slot:$bypassed');

  @override
  void setParam(int slot, String json) => calls.add('setParam:$slot:$json');

  @override
  void setMute(bool muted) => calls.add('setMute:$muted');
}
