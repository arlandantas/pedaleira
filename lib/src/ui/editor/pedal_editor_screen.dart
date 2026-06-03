import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../domain/models.dart';
import '../../providers/pedalboard_provider.dart';
import 'knob_widget.dart';

class PedalEditorScreen extends ConsumerWidget {
  final int slot;
  const PedalEditorScreen({super.key, required this.slot});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final pedal = ref.watch(
      pedalboardProvider.select((s) => s[slot]),
    );
    return Scaffold(
      appBar: AppBar(title: Text(kPedalNames[pedal.slot]!)),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(32),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Enabled', style: TextStyle(fontSize: 16)),
                Switch(
                  value: !pedal.bypassed,
                  onChanged: (_) => ref
                      .read(pedalboardProvider.notifier)
                      .toggleBypass(slot),
                ),
              ],
            ),
            const Divider(height: 32),
            Wrap(
              spacing: 32,
              runSpacing: 32,
              children: pedal.params.entries.map((entry) {
                final range = kParamRanges[entry.key] ?? (0.0, 1.0);
                return KnobWidget(
                  label: entry.key,
                  value: entry.value,
                  min: range.$1,
                  max: range.$2,
                  onChanged: (v) => ref
                      .read(pedalboardProvider.notifier)
                      .updateParam(slot, entry.key, v),
                );
              }).toList(),
            ),
          ],
        ),
      ),
    );
  }
}
