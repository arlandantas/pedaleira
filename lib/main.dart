import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'src/data/rust_engine_repository.dart';
import 'src/providers/engine_provider.dart';
import 'src/rust/frb_generated.dart';
import 'src/ui/app.dart';

// Path to a WAV file for the audio engine.
// Convert the sample MP3 first: ffmpeg -i sample_audios/*.mp3 sample_audios/guitar_di.wav
const _kWavPath = 'sample_audios/guitar_di.wav';

Future<void> main() async {
  await RustLib.init();
  final engine = RustEngineRepository();
  engine.start(_kWavPath);
  runApp(ProviderScope(
    overrides: [
      engineRepositoryProvider.overrideWithValue(engine),
    ],
    child: const App(),
  ));
}
