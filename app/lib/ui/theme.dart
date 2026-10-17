/// Visual identity: Material 3, generated from one seed colour, with a few
/// shape and spacing choices applied everywhere.
library;

import 'package:flutter/material.dart';

/// Jixel indigo.
const seed = Color(0xFF5B4BDB);

ThemeData buildTheme(Brightness brightness) {
  final scheme = ColorScheme.fromSeed(seedColor: seed, brightness: brightness);
  final base = ThemeData(colorScheme: scheme, useMaterial3: true, visualDensity: VisualDensity.standard);
  return base.copyWith(
    scaffoldBackgroundColor: scheme.surface,
    cardTheme: CardThemeData(
      elevation: 0,
      color: scheme.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(20)),
      margin: EdgeInsets.zero,
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        minimumSize: const Size(64, 48),
        padding: const EdgeInsets.symmetric(horizontal: 24),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
        textStyle: const TextStyle(fontWeight: FontWeight.w600, fontSize: 15),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        minimumSize: const Size(64, 44),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
      ),
    ),
    chipTheme: base.chipTheme.copyWith(shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10))),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: scheme.surfaceContainerHighest.withValues(alpha: 0.5),
      border: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide.none),
    ),
    sliderTheme: base.sliderTheme.copyWith(year2023: false),
    textTheme: base.textTheme.copyWith(
      displaySmall: base.textTheme.displaySmall?.copyWith(fontWeight: FontWeight.w700, letterSpacing: -0.5),
      headlineSmall: base.textTheme.headlineSmall?.copyWith(fontWeight: FontWeight.w700),
      titleMedium: base.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600),
    ),
  );
}

/// Standard gaps, so spacing stays consistent.
abstract final class Gap {
  static const xs = SizedBox(width: 4, height: 4);
  static const s = SizedBox(width: 8, height: 8);
  static const m = SizedBox(width: 16, height: 16);
  static const l = SizedBox(width: 24, height: 24);
  static const xl = SizedBox(width: 40, height: 40);
}

/// Colour for "good news" numbers (savings), readable in both themes.
Color successColor(BuildContext context) =>
    Theme.of(context).brightness == Brightness.dark ? const Color(0xFF7BE0A4) : const Color(0xFF1B7F4B);
