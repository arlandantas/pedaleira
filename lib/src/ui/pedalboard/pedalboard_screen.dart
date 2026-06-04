import 'package:flutter/material.dart';
import 'pedal_tile.dart';
import 'mute_bar.dart';
import '../preset_bar.dart';

class PedalboardScreen extends StatelessWidget {
  const PedalboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Pedaleira'),
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
                          Expanded(child: PedalTile(slot: 8)), // Reverb
                          const SizedBox(width: 12),
                          Expanded(child: PedalTile(slot: 9)), // Boost
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
}
