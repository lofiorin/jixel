/// The app's view of the Rust engine: one async method per operation.
///
/// Every method runs on a background isolate (see [callNative]) and throws
/// [EngineException] with a readable message on failure. Methods take and
/// return file paths; only on-screen previews travel as pixels.
library;

import 'dart:typed_data';

import 'models.dart';
import 'native.dart';

export 'models.dart';
export 'native.dart' show EngineException;

/// Engine version and capabilities.
class EngineInfo {
  EngineInfo(Map<String, Object?> j)
    : version = j['jixel']! as String,
      libjxl = j['libjxl']! as String,
      backgroundRemoval = j['background_removal'] == true;

  final String version;
  final String libjxl;
  final bool backgroundRemoval;
}

/// Result of a trial encode for the before/after view.
class Trial {
  Trial(this.result, this.score, this.preview);
  final ConvertResult result;

  /// SSIMULACRA2 against the original, when comparable.
  final double? score;
  final RgbaPreview preview;
}

/// Result of a web export.
class WebExportResult {
  WebExportResult(Map<String, Object?> j)
    : manifest = j['manifest']! as Map<String, Object?>,
      snippets = (j['snippets']! as Map<String, Object?>).cast<String, String>();

  final Map<String, Object?> manifest;

  /// `path → contents`, e.g. `snippets/picture.html`.
  final Map<String, String> snippets;

  /// Total bytes per format, JPEG XL first.
  List<(ImageFormat, int)> get totals => [
    for (final t in manifest['totals']! as List) (ImageFormat.parse((t as List)[0]), (t[1] as num).toInt()),
  ];

  int get fileCount => (manifest['files']! as List).length;
}

/// Subject boost outcome (Rust: `BoostResult`).
class BoostReport {
  BoostReport(Map<String, Object?> j)
    : subject = Subject(j['subject']! as Map<String, Object?>),
      singleBytes = ((j['result']! as Map)['single_bytes'] as num).toInt(),
      boostedBytes = ((j['result']! as Map)['boosted_bytes'] as num?)?.toInt(),
      singleSubjectScore = ((j['result']! as Map)['single_subject_score'] as num).toDouble(),
      boostedSubjectScore = ((j['result']! as Map)['boosted_subject_score'] as num?)?.toDouble(),
      recommendBoost = (j['result']! as Map)['recommend_boost'] == true;

  final Subject subject;
  final int singleBytes;
  final int? boostedBytes;
  final double singleSubjectScore;
  final double? boostedSubjectScore;
  final bool recommendBoost;
}

/// One step of the progressive-loading simulation.
class ProgressiveStep {
  ProgressiveStep(this.fraction, this.bytes, this.preview);
  final double fraction;
  final int bytes;
  final RgbaPreview? preview;
}

abstract final class Engine {
  static Future<EngineInfo> info() async => EngineInfo((await callNative('version', {})).map);

  static Future<Inspection> inspect(String path) async => Inspection((await callNative('inspect', {'path': path})).map);

  /// Decoded pixels of any supported file, at most [maxSide] px.
  static Future<RgbaPreview> preview(String path, {int maxSide = 2048}) async {
    final r = await callNative('preview', {'path': path, 'max_side': maxSide});
    return RgbaPreview.fromJson(r.map, r.buffers);
  }

  static Future<ConvertResult> convert(String input, String output, ConvertSettings s) async =>
      ConvertResult((await callNative('convert', {'input': input, 'output': output, 'options': s.toJson()})).map);

  /// Encodes in memory and returns what it would look like.
  static Future<Trial> trial(String input, ConvertSettings s, {int maxSide = 2048}) async {
    final r = await callNative('trial', {'input': input, 'options': s.toJson(), 'max_side': maxSide});
    return Trial(
      ConvertResult(r.map['result']! as Map<String, Object?>),
      (r.map['score'] as num?)?.toDouble(),
      RgbaPreview.fromJson(r.map['preview']! as Map<String, Object?>, r.buffers),
    );
  }

  static Future<WebExportResult> webExport(
    String input,
    String outputDir,
    Map<String, Object?> options, {
    bool autoFocus = false,
  }) async => WebExportResult(
    (await callNative('web_export', {
      'input': input,
      'output_dir': outputDir,
      'options': options,
      'auto_focus': autoFocus,
    })).map,
  );

  /// Same visual quality ([targetScore]) or same setting ([quality]).
  static Future<(int, int, List<BattleEntry>)> battle(
    String input, {
    double? targetScore = 80,
    double? quality,
    int effort = 7,
    String? outputDir,
  }) async {
    final goal = quality != null
        ? {'mode': 'same_setting', 'quality': quality}
        : {'mode': 'same_score', 'score': targetScore};
    final r = (await callNative('battle', {
      'input': input,
      'goal': goal,
      'effort': effort,
      'output_dir': outputDir,
    })).map;
    return (
      (r['width']! as num).toInt(),
      (r['height']! as num).toInt(),
      [for (final e in r['entries']! as List) BattleEntry(e as Map<String, Object?>)],
    );
  }

  static Future<Subject> detectSubject(String path) async =>
      Subject((await callNative('detect_subject', {'path': path})).map);

  static Future<BoostReport> subjectBoost(
    String input,
    String output, {
    Subject? subject,
    double quality = 85,
    int effort = 7,
  }) async => BoostReport(
    (await callNative('subject_boost', {
      'input': input,
      'output': output,
      'subject': subject?.raw,
      'options': {'quality': quality, 'effort': effort},
    })).map,
  );

  static Future<(int, RgbaPreview)> removeBackground(
    String input,
    String output, {
    ImageFormat format = ImageFormat.png,
    List<int>? fill,
    bool hardEdges = false,
    int maxSide = 2048,
  }) async {
    final r = await callNative('remove_background', {
      'input': input,
      'output': output,
      'format': format.name,
      'options': {'refine_edges': true, 'hard_edges': hardEdges, 'fill': fill},
      'max_side': maxSide,
    });
    return (
      (r.map['bytes']! as num).toInt(),
      RgbaPreview.fromJson(r.map['preview']! as Map<String, Object?>, r.buffers),
    );
  }

  /// Pixel-exact unless [quality] is given.
  static Future<(int, Fidelity)> crop(String input, String output, PixelRect rect, {double? quality}) async {
    final r = (await callNative('crop', {
      'input': input,
      'output': output,
      'rect': rect.toJson(),
      'quality': quality,
    })).map;
    return ((r['bytes']! as num).toInt(), Fidelity.parse(r['fidelity']));
  }

  static Future<int> tiles(
    String input,
    String outputDir, {
    required int cols,
    required int rows,
    ImageFormat format = ImageFormat.jxl,
    double quality = 100,
  }) async {
    final r = (await callNative('tiles', {
      'input': input,
      'output_dir': outputDir,
      'split': {'mode': 'grid', 'cols': cols, 'rows': rows},
      'format': format.name,
      'quality': quality,
    })).map;
    return (r['files']! as List).length;
  }

  static Future<(String, int, int)> deepZoom(
    String input,
    String outputDir, {
    int tileSize = 254,
    int overlap = 1,
    ImageFormat format = ImageFormat.jxl,
    double quality = 85,
  }) async {
    final r = (await callNative('deep_zoom', {
      'input': input,
      'output_dir': outputDir,
      'tile_size': tileSize,
      'overlap': overlap,
      'format': format.name,
      'quality': quality,
    })).map;
    return (r['dzi']! as String, (r['levels']! as num).toInt(), (r['tiles']! as num).toInt());
  }

  static Future<int> reconstructJpeg(String input, String output) async =>
      ((await callNative('reconstruct', {'input': input, 'output': output})).map['bytes']! as num).toInt();

  /// Encodes [input] as JPEG XL in memory (for the progressive viewer).
  static Future<Uint8List> encodeJxl(String input, ConvertSettings s) async {
    final r = await callNative('encode_jxl', {'input': input, 'options': s.toJson()});
    return r.buffers[(r.map['buffer']! as num).toInt()];
  }

  /// What [jxl] looks like after each fraction of the file has arrived.
  static Future<List<ProgressiveStep>> progressive(Uint8List jxl, List<double> fractions, {int maxSide = 1600}) async {
    final r = await callNative('progressive', {'fractions': fractions, 'max_side': maxSide}, [jxl]);
    return [
      for (final s in r.value! as List)
        ProgressiveStep(
          ((s as Map)['fraction'] as num).toDouble(),
          (s['bytes'] as num).toInt(),
          s['preview'] == null ? null : RgbaPreview.fromJson(s['preview'] as Map<String, Object?>, r.buffers),
        ),
    ];
  }
}
