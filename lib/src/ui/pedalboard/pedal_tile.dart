import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../domain/models.dart';
import '../../providers/pedalboard_provider.dart';
import '../editor/pedal_editor_screen.dart';

class PedalTile extends ConsumerWidget {
  final int slot;
  const PedalTile({super.key, required this.slot});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final pedal = ref.watch(
      pedalboardProvider.select((s) => s[slot]),
    );
    final isActive = !pedal.bypassed;
    final theme = Theme.of(context);
    final activeColor = theme.colorScheme.primary;

    return GestureDetector(
      onTap: () => ref.read(pedalboardProvider.notifier).toggleBypass(slot),
      onLongPress: () => _openEditor(context),
      child: Container(
        decoration: BoxDecoration(
          color: theme.colorScheme.surface,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: isActive ? activeColor : Colors.grey.shade800,
            width: isActive ? 1.5 : 1,
          ),
        ),
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Container(
                  width: 10,
                  height: 10,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: isActive ? activeColor : Colors.grey.shade700,
                  ),
                ),
                GestureDetector(
                  onTap: () => _openEditor(context),
                  child: Icon(
                    Icons.settings,
                    size: 16,
                    color: Colors.grey.shade500,
                  ),
                ),
              ],
            ),
            const Spacer(),
            Text(
              kPedalNames[pedal.slot]!,
              style: theme.textTheme.labelMedium?.copyWith(
                color: isActive ? Colors.white : Colors.grey.shade600,
                fontWeight: FontWeight.bold,
                letterSpacing: 0.5,
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _openEditor(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => PedalEditorScreen(slot: slot),
      ),
    );
  }
}
