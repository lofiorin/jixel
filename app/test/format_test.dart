import 'package:flutter_test/flutter_test.dart';
import 'package:jixel/engine/models.dart';
import 'package:jixel/ui/format.dart';
import 'package:jixel/ui/screens/web_export.dart' show slug;

void main() {
  test('formatBytes picks a readable unit', () {
    expect(formatBytes(512, 'en'), '512 B');
    expect(formatBytes(1536, 'en'), '1.5 KB');
    expect(formatBytes(5 * 1024 * 1024, 'en'), '5 MB');
    expect(formatBytes(1536, 'it'), '1,5 KB');
  });

  test('formatPercent signs savings and growth', () {
    expect(formatPercent(-35.2), '−35%');
    expect(formatPercent(12), '+12%');
    expect(formatPercent(0.2), '0%');
    expect(formatPercentAbs(-35.2), '35%');
  });

  test('slug makes URL-safe names', () {
    expect(slug('My Holiday Photo (1)'), 'my-holiday-photo-1');
    expect(slug('***'), 'image');
  });

  test('score bands follow SSIMULACRA2 guidance', () {
    expect(scoreBand(92), ScoreBand.visuallyLossless);
    expect(scoreBand(85), ScoreBand.veryHigh);
    expect(scoreBand(72), ScoreBand.high);
    expect(scoreBand(55), ScoreBand.medium);
    expect(scoreBand(10), ScoreBand.low);
  });

  test('settings serialise with the engine field names', () {
    final json = const ConvertSettings(maxWidth: 1920).toJson();
    expect(json['target'], 'jxl');
    expect(json['resize'], {'max_width': 1920, 'max_height': null});
    expect(json['metadata'], 'strip_gps');
    expect(const ConvertSettings().toJson()['resize'], isNull);
  });
}
