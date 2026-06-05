import 'dart:convert';
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';
import '../../domain/preset_io.dart';
import '../../providers/preset_provider.dart';
import 'pedal_tile.dart';
import 'mute_bar.dart';
import '../preset_bar.dart';

class PedalboardScreen extends ConsumerStatefulWidget {
  const PedalboardScreen({super.key});

  @override
  ConsumerState<PedalboardScreen> createState() => _PedalboardScreenState();
}

class _PedalboardScreenState extends ConsumerState<PedalboardScreen> {
  @override
  Widget build(BuildContext context) {
    final presetsAsync = ref.watch(presetListProvider);
    final hasPresets = presetsAsync.valueOrNull?.isNotEmpty ?? false;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Pedaleira'),
        actions: [
          IconButton(
            icon: const Icon(Icons.upload_file),
            tooltip: 'Import preset',
            onPressed: _importPreset,
          ),
          IconButton(
            icon: const Icon(Icons.ios_share),
            tooltip: 'Export preset',
            onPressed: hasPresets ? _exportPreset : null,
          ),
        ],
        bottom: const PreferredSize(
          preferredSize: Size.fromHeight(48),
          child: PresetBar(),
        ),
      ),
      body: OrientationBuilder(
        builder: (context, orientation) {
          final crossAxisCount = orientation == Orientation.portrait ? 2 : 4;
          final rowCount = orientation == Orientation.portrait ? 4 : 2;
          return LayoutBuilder(
            builder: (context, constraints) {
              const reverbRowHeight = 72.0;
              const reverbRowBottomPad = 8.0;
              const muteBarHeight = 56.0;
              const muteBarBottomPad = 16.0;
              const gridPadTop = 16.0;
              const gridPadBottom = 8.0;
              const tileSpacing = 12.0;
              final gridContentHeight = constraints.maxHeight
                  - reverbRowHeight
                  - reverbRowBottomPad
                  - muteBarHeight
                  - muteBarBottomPad
                  - gridPadTop
                  - gridPadBottom;
              final tileHeight =
                  (gridContentHeight - tileSpacing * (rowCount - 1)) / rowCount;
              return Column(
                children: [
                  Expanded(
                    child: GridView.builder(
                      padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                      physics: const NeverScrollableScrollPhysics(),
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: crossAxisCount,
                        crossAxisSpacing: 12,
                        mainAxisSpacing: 12,
                        mainAxisExtent: tileHeight,
                      ),
                      itemCount: 8,
                      itemBuilder: (_, i) => PedalTile(slot: i),
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 0, 16, reverbRowBottomPad),
                    child: SizedBox(
                      height: reverbRowHeight,
                      child: Row(
                        children: [
                          Expanded(child: PedalTile(slot: 8)),
                          const SizedBox(width: 12),
                          Expanded(child: PedalTile(slot: 9)),
                        ],
                      ),
                    ),
                  ),
                  const MuteBar(),
                ],
              );
            },
          );
        },
      ),
    );
  }

  Future<void> _exportPreset() async {
    final presets = ref.read(presetListProvider).valueOrNull ?? [];
    final idx = ref.read(activePresetIndexProvider);
    if (presets.isEmpty) return;
    final preset = presets[idx.clamp(0, presets.length - 1)];

    final dir = await getTemporaryDirectory();
    final file = File('${dir.path}/${preset.name}.json');
    await file.writeAsString(preset.toJsonString());
    await Share.shareXFiles(
      [XFile(file.path)],
      subject: preset.name,
    );
    await file.delete();
  }

  Future<void> _importPreset() async {
    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['json'],
      withData: true,
    );
    if (result == null || !mounted) return;

    final pickedFile = result.files.single;
    final String jsonString;
    if (pickedFile.bytes != null) {
      jsonString = utf8.decode(pickedFile.bytes!);
    } else if (pickedFile.path != null) {
      jsonString = await File(pickedFile.path!).readAsString();
    } else {
      _showSnackBar('Could not read file.');
      return;
    }

    final preset = parsePresetJson(jsonString);
    if (!mounted) return;
    if (preset == null) {
      _showSnackBar('Invalid preset file.');
      return;
    }

    final existing = (ref.read(presetListProvider).valueOrNull ?? [])
        .map((p) => p.name)
        .toList();

    final String finalName;
    if (existing.contains(preset.name)) {
      final overwrite = await showDialog<bool>(
        context: context,
        builder: (_) => AlertDialog(
          title: const Text('Name conflict'),
          content: Text('A preset named "${preset.name}" already exists.'),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Save as copy'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('Overwrite'),
            ),
          ],
        ),
      );
      if (!mounted) return;
      if (overwrite == null) return;
      finalName = overwrite
          ? preset.name
          : resolveImportName(preset.name, existing);
    } else {
      finalName = preset.name;
    }

    await ref.read(presetListProvider.notifier).saveCurrentAs(
          finalName,
          preset.pedals,
        );

    if (!mounted) return;
    final updated = ref.read(presetListProvider).valueOrNull ?? [];
    final newIdx = updated.indexWhere((p) => p.name == finalName);
    if (newIdx >= 0) {
      ref.read(activePresetIndexProvider.notifier).state = newIdx;
    }
    _showSnackBar('Preset "$finalName" imported.');
  }

  void _showSnackBar(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }
}
