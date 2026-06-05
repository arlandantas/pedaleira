import 'package:flutter_test/flutter_test.dart';
import 'package:pedaleira/src/domain/models.dart';
import 'package:pedaleira/src/domain/preset_io.dart';

void main() {
  group('parsePresetJson', () {
    test('returns Preset for valid JSON', () {
      final preset = Preset(
        name: 'Test',
        pedals: PedalSlot.values.map((s) => PedalState(
          slot: s,
          bypassed: false,
          params: Map.from(kDefaultParams[s]!),
        )).toList(),
      );
      final json = preset.toJsonString();
      final result = parsePresetJson(json);
      expect(result, isNotNull);
      expect(result!.name, 'Test');
    });

    test('returns null for invalid JSON', () {
      expect(parsePresetJson('not json'), isNull);
    });

    test('returns null for JSON missing required fields', () {
      expect(parsePresetJson('{"foo": 1}'), isNull);
    });
  });

  group('resolveImportName', () {
    test('returns name unchanged when no conflict', () {
      expect(resolveImportName('Clean', ['Crunch', 'Lead']), 'Clean');
    });

    test('appends - imported when name already exists', () {
      expect(resolveImportName('Clean', ['Clean', 'Lead']), 'Clean - imported');
    });

    test('returns name unchanged for empty list', () {
      expect(resolveImportName('Clean', []), 'Clean');
    });
  });
}
