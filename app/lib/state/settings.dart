/// User preferences, persisted with `shared_preferences`.
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../engine/models.dart';

class SettingsController extends ChangeNotifier {
  SettingsController._(this._prefs, this._defaultOutput);

  /// Loads saved settings (or defaults).
  static Future<SettingsController> load() async {
    final prefs = await SharedPreferences.getInstance();
    return SettingsController._(prefs, await _defaultOutputDir());
  }

  /// Settings over [prefs] with a fixed default folder (for tests).
  @visibleForTesting
  SettingsController.withPrefs(SharedPreferences prefs, String defaultOutput) : this._(prefs, defaultOutput);

  final SharedPreferences _prefs;
  final String _defaultOutput;

  ThemeMode get themeMode => ThemeMode.values.byName(_prefs.getString('theme') ?? 'system');
  set themeMode(ThemeMode v) => _set('theme', v.name);

  /// `null` follows the system language.
  Locale? get locale => switch (_prefs.getString('locale')) {
    null || '' => null,
    final l => Locale(l),
  };
  set locale(Locale? v) => _set('locale', v?.languageCode ?? '');

  /// Where results are written.
  String get outputDir => _prefs.getString('outputDir') ?? _defaultOutput;
  set outputDir(String v) => _set('outputDir', v);

  double get quality => _prefs.getDouble('quality') ?? 90;
  set quality(double v) => _setDouble('quality', v);

  SpeedPreset get speed => SpeedPreset.values.byName(_prefs.getString('speed') ?? 'balanced');
  set speed(SpeedPreset v) => _set('speed', v.name);

  MetadataPolicy get metadata => MetadataPolicy.parse(_prefs.getString('metadata'));
  set metadata(MetadataPolicy v) => _set('metadata', v.wire);

  void _set(String key, String value) {
    _prefs.setString(key, value);
    notifyListeners();
  }

  void _setDouble(String key, double value) {
    _prefs.setDouble(key, value);
    notifyListeners();
  }

  /// Desktop: ~/Downloads/Jixel. Mobile: the app's documents folder (files
  /// are then shared with the system share sheet).
  static Future<String> _defaultOutputDir() async {
    if (Platform.isMacOS || Platform.isWindows || Platform.isLinux) {
      final downloads = await getDownloadsDirectory();
      if (downloads != null) return p.join(downloads.path, 'Jixel');
    }
    return p.join((await getApplicationDocumentsDirectory()).path, 'Jixel');
  }
}
