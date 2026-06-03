import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../domain/models.dart';
import 'engine_provider.dart';

class PedalboardNotifier extends Notifier<List<PedalState>> {
  @override
  List<PedalState> build() {
    return PedalSlot.values.map((slot) => PedalState(
      slot: slot,
      bypassed: true,
      params: Map.from(kDefaultParams[slot]!),
    )).toList();
  }

  void toggleBypass(int slot) {
    final current = state[slot];
    final updated = current.copyWith(bypassed: !current.bypassed);
    state = [...state]..[slot] = updated;
    ref.read(engineRepositoryProvider).toggleBypass(slot, updated.bypassed);
  }

  void updateParam(int slot, String key, double value) {
    final current = state[slot];
    final newParams = Map<String, double>.from(current.params)..[key] = value;
    state = [...state]..[slot] = current.copyWith(params: newParams);
    ref.read(engineRepositoryProvider).setParam(slot, jsonEncode(newParams));
  }

  void applyPreset(Preset preset) {
    state = List.from(preset.pedals);
    final engine = ref.read(engineRepositoryProvider);
    for (final pedal in preset.pedals) {
      final idx = pedal.slot.index;
      engine.toggleBypass(idx, pedal.bypassed);
      engine.setParam(idx, jsonEncode(pedal.params));
    }
  }
}

final pedalboardProvider =
    NotifierProvider<PedalboardNotifier, List<PedalState>>(
  PedalboardNotifier.new,
);
