import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'engine_provider.dart';

class MuteNotifier extends Notifier<bool> {
  @override
  bool build() => false;

  void toggle() {
    state = !state;
    ref.read(engineRepositoryProvider).setMute(state);
  }
}

final muteProvider = NotifierProvider<MuteNotifier, bool>(MuteNotifier.new);
