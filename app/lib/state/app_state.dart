/// App-wide state shared by the screens: which section is open and which
/// files the user handed over from the home screen.
library;

import 'package:flutter/widgets.dart';

import '../engine/engine.dart';
import 'settings.dart';

/// The sections of the app, in navigation order.
enum Section { home, convert, web, studio, inspect, settings }

class AppState extends ChangeNotifier {
  AppState(this.settings);

  final SettingsController settings;

  Section _section = Section.home;
  Section get section => _section;

  /// Engine version info, loaded once at start-up.
  EngineInfo? engineInfo;

  /// Files waiting to be picked up by a screen (see [take]).
  final Map<Section, List<String>> _handoff = {};

  /// Studio tool to open with the next hand-off (e.g. background removal).
  int? studioTool;

  void go(Section s) {
    _section = s;
    notifyListeners();
  }

  /// Opens [section] with [files] ready to use.
  void open(Section section, List<String> files, {int? tool}) {
    _handoff[section] = files;
    studioTool = tool;
    go(section);
  }

  /// Returns (and forgets) files handed to [section].
  List<String> take(Section section) => _handoff.remove(section) ?? const [];
}

/// Makes [AppState] available to the widget tree.
class AppScope extends InheritedNotifier<AppState> {
  const AppScope({super.key, required AppState state, required super.child}) : super(notifier: state);

  static AppState of(BuildContext context) => context.dependOnInheritedWidgetOfExactType<AppScope>()!.notifier!;

  /// Reads the state without rebuilding when it changes.
  static AppState read(BuildContext context) => context.getInheritedWidgetOfExactType<AppScope>()!.notifier!;
}
