import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'src/rust/frb_generated.dart';
import 'src/ui/app.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const ProviderScope(child: App()));
}
