/// Jixel: JPEG XL for everyone.
///
/// Start here: [JixelApp] (app.dart) builds the UI; screens live in
/// `ui/screens/`; every image operation goes through `engine/engine.dart`,
/// which calls the Rust core (see docs/en/architecture.md).
library;

import 'package:flutter/material.dart';

import 'app.dart';
import 'engine/engine.dart';
import 'state/app_state.dart';
import 'state/settings.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final state = AppState(await SettingsController.load());
  runApp(JixelApp(state: state));
  // Version info is only shown in Settings: load it without delaying start-up.
  try {
    state.engineInfo = await Engine.info();
  } on Object catch (e) {
    debugPrint('Engine not available: $e');
  }
}
