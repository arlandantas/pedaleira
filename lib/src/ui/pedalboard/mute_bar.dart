import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/mute_provider.dart';

class MuteBar extends ConsumerWidget {
  const MuteBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final muted = ref.watch(muteProvider);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      child: GestureDetector(
        onTap: () => ref.read(muteProvider.notifier).toggle(),
        child: Container(
          height: 56,
          decoration: BoxDecoration(
            color: muted ? Colors.red.shade900 : theme.colorScheme.surface,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: muted ? Colors.red : Colors.grey.shade800,
              width: muted ? 1.5 : 1,
            ),
          ),
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                children: [
                  Icon(
                    muted ? Icons.volume_off : Icons.volume_up,
                    color: muted ? Colors.red : Colors.grey.shade500,
                    size: 18,
                  ),
                  const SizedBox(width: 8),
                  Text(
                    muted ? 'MUTED' : 'Output',
                    style: theme.textTheme.labelMedium?.copyWith(
                      color: muted ? Colors.red : Colors.grey.shade500,
                      fontWeight: FontWeight.bold,
                      letterSpacing: 0.5,
                    ),
                  ),
                ],
              ),
              Switch(
                value: !muted,
                onChanged: (_) => ref.read(muteProvider.notifier).toggle(),
                activeColor: theme.colorScheme.primary,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
