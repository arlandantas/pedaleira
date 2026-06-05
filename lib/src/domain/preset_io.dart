import 'dart:convert';
import 'models.dart';

/// Returns null if [json] is not a valid Preset.
Preset? parsePresetJson(String json) {
  try {
    final map = jsonDecode(json) as Map<String, dynamic>;
    return Preset.fromJson(map);
  } catch (_) {
    return null;
  }
}

/// Returns [name] if not in [existing]; otherwise returns '[name] - imported'.
String resolveImportName(String name, List<String> existing) {
  if (!existing.contains(name)) return name;
  return '$name - imported';
}
