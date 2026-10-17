/// Number formatting shared by the screens.
library;

import 'package:intl/intl.dart';

/// 1536 → "1.5 KB". Uses the decimal separator of [locale].
String formatBytes(int bytes, [String? locale]) {
  final f = NumberFormat('#,##0.#', locale);
  if (bytes < 1024) return '$bytes B';
  if (bytes < 1024 * 1024) return '${f.format(bytes / 1024)} KB';
  return '${NumberFormat('#,##0.##', locale).format(bytes / 1024 / 1024)} MB';
}

/// −35.2 → "−35%", 12 → "+12%".
String formatPercent(double value) {
  final rounded = value.round();
  if (rounded == 0) return '0%';
  return rounded < 0 ? '−${-rounded}%' : '+$rounded%';
}

/// 35.2 → "35%" (no sign).
String formatPercentAbs(double value) => '${value.abs().round()}%';
