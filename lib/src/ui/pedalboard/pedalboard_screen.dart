import 'package:flutter/material.dart';
import 'pedal_tile.dart';
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
          final crossAxisCount =
              orientation == Orientation.portrait ? 2 : 4;
          return Column(
            children: [
              Expanded(
                child: GridView.builder(
                  padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: crossAxisCount,
                    crossAxisSpacing: 12,
                    mainAxisSpacing: 12,
                    childAspectRatio: 1.2,
                  ),
                  itemCount: 8,
                  itemBuilder: (_, i) => PedalTile(slot: i),
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                child: SizedBox(
                  height: 72,
                  child: PedalTile(slot: 8),
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}
