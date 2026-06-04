abstract class EngineRepository {
  void start(String wavPath);
  void stop();
  void toggleBypass(int slot, bool bypassed);
  void setParam(int slot, String json);
  void setMute(bool muted);
}
