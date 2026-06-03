import 'dart:convert';
import 'dart:io';
import 'package:path_provider/path_provider.dart';
import '../domain/models.dart';
import '../domain/preset_repository.dart';

class FilePresetRepository implements PresetRepository {
  final String? dirOverride;

  const FilePresetRepository({this.dirOverride});

  Future<Directory> _presetsDir() async {
    final basePath = dirOverride ??
        (await getApplicationDocumentsDirectory()).path;
    final dir = Directory('$basePath/presets');
    if (!await dir.exists()) await dir.create(recursive: true);
    return dir;
  }

  @override
  Future<List<Preset>> loadAll() async {
    final dir = await _presetsDir();
    final files = dir
        .listSync()
        .whereType<File>()
        .where((f) => f.path.endsWith('.json'));
    return files.map((f) {
      final json = jsonDecode(f.readAsStringSync()) as Map<String, dynamic>;
      return Preset.fromJson(json);
    }).toList();
  }

  @override
  Future<void> save(Preset preset) async {
    final dir = await _presetsDir();
    final file = File('${dir.path}/${preset.name}.json');
    await file.writeAsString(jsonEncode(preset.toJson()));
  }

  @override
  Future<void> delete(String name) async {
    final dir = await _presetsDir();
    final file = File('${dir.path}/$name.json');
    if (await file.exists()) await file.delete();
  }
}
