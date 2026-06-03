import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/domain/models.dart';

void main() {
  group('PedalState', () {
    test('JSON round-trip', () {
      final state = PedalState(
        slot: PedalSlot.compressor,
        bypassed: false,
        params: {'threshold_db': -18.0, 'ratio': 4.0, 'attack': 0.01, 'release': 0.1},
      );
      final restored = PedalState.fromJson(state.toJson());
      expect(restored.slot, PedalSlot.compressor);
      expect(restored.bypassed, false);
      expect(restored.params['ratio'], 4.0);
    });

    test('copyWith only changes specified fields', () {
      final original = PedalState(
        slot: PedalSlot.delay,
        bypassed: true,
        params: {'time_ms': 300.0},
      );
      final copy = original.copyWith(bypassed: false);
      expect(copy.bypassed, false);
      expect(copy.slot, PedalSlot.delay);
      expect(copy.params['time_ms'], 300.0);
    });
  });

  group('Preset', () {
    test('JSON round-trip', () {
      final pedals = PedalSlot.values.map((s) => PedalState(
        slot: s,
        bypassed: true,
        params: Map.from(kDefaultParams[s]!),
      )).toList();
      final preset = Preset(name: 'Clean', pedals: pedals);
      final restored = Preset.fromJson(preset.toJson());
      expect(restored.name, 'Clean');
      expect(restored.pedals.length, 9);
      expect(restored.pedals[1].slot, PedalSlot.compressor);
    });
  });
}
