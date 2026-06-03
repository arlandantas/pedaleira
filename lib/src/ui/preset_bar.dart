import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../domain/models.dart';
import '../providers/pedalboard_provider.dart';
import '../providers/preset_provider.dart';

class PresetBar extends ConsumerWidget {
  const PresetBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final presetsAsync = ref.watch(presetListProvider);
    final activeIndex = ref.watch(activePresetIndexProvider);
    final pedalboard = ref.watch(pedalboardProvider);

    return presetsAsync.when(
      data: (presets) {
        final hasPresets = presets.isNotEmpty;
        final clampedIndex =
            hasPresets ? activeIndex.clamp(0, presets.length - 1) : 0;
        final name = hasPresets ? presets[clampedIndex].name : '—';

        return Row(
          children: [
            IconButton(
              icon: const Icon(Icons.chevron_left),
              onPressed: hasPresets && activeIndex > 0
                  ? () => _navigate(ref, presets, activeIndex - 1)
                  : null,
            ),
            Expanded(
              child: Text(
                name,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  fontWeight: FontWeight.bold,
                  fontSize: 14,
                ),
              ),
            ),
            IconButton(
              icon: const Icon(Icons.chevron_right),
              onPressed: hasPresets && activeIndex < presets.length - 1
                  ? () => _navigate(ref, presets, activeIndex + 1)
                  : null,
            ),
            IconButton(
              icon: const Icon(Icons.save),
              tooltip: 'Save',
              onPressed: () =>
                  _save(context, ref, presets, clampedIndex, pedalboard),
            ),
            IconButton(
              icon: const Icon(Icons.copy),
              tooltip: 'Duplicate preset',
              onPressed: hasPresets
                  ? () => _duplicate(ref, presets[clampedIndex], pedalboard)
                  : null,
            ),
            IconButton(
              icon: const Icon(Icons.edit),
              tooltip: 'Rename preset',
              onPressed: hasPresets
                  ? () => _rename(
                        context,
                        ref,
                        presets[clampedIndex],
                        pedalboard,
                      )
                  : null,
            ),
          ],
        );
      },
      loading: () => const SizedBox.shrink(),
      error: (err, st) => const SizedBox.shrink(),
    );
  }

  void _navigate(WidgetRef ref, List<Preset> presets, int idx) {
    ref.read(activePresetIndexProvider.notifier).state = idx;
    ref.read(pedalboardProvider.notifier).applyPreset(presets[idx]);
  }

  Future<void> _save(
    BuildContext context,
    WidgetRef ref,
    List<Preset> presets,
    int activeIndex,
    List<PedalState> pedalboard,
  ) async {
    if (presets.isNotEmpty) {
      final currentName = presets[activeIndex].name;
      await ref
          .read(presetListProvider.notifier)
          .saveCurrentAs(currentName, pedalboard);
    } else {
      if (!context.mounted) return;
      final name = await _promptName(context, title: 'Save Preset', initial: '');
      if (name != null && name.isNotEmpty && context.mounted) {
        await ref
            .read(presetListProvider.notifier)
            .saveCurrentAs(name, pedalboard);
      }
    }
  }

  Future<void> _duplicate(
    WidgetRef ref,
    Preset current,
    List<PedalState> pedalboard,
  ) async {
    await ref
        .read(presetListProvider.notifier)
        .duplicatePreset(current.name, pedalboard);
  }

  Future<void> _rename(
    BuildContext context,
    WidgetRef ref,
    Preset current,
    List<PedalState> pedalboard,
  ) async {
    final newName = await _promptName(
      context,
      title: 'Rename Preset',
      initial: current.name,
    );
    if (newName == null || newName.isEmpty || newName == current.name) return;
    if (!context.mounted) return;
    await ref
        .read(presetListProvider.notifier)
        .renamePreset(current.name, newName, pedalboard);
  }

  Future<String?> _promptName(
    BuildContext context, {
    required String title,
    required String initial,
  }) {
    final controller = TextEditingController(text: initial);
    return showDialog<String>(
      context: context,
      builder: (_) => AlertDialog(
        title: Text(title),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(labelText: 'Name'),
          autofocus: true,
          onSubmitted: (v) => Navigator.pop(context, v.trim()),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, controller.text.trim()),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }
}
