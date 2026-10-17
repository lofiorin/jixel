/// Typed views over the JSON the engine returns.
///
/// Field names mirror the Rust structs in `core/crates/jixel-core` (serde
/// snake_case), so the Rust docs describe these too.
library;

import 'dart:async';
import 'dart:typed_data';
import 'dart:ui' as ui;

/// Image formats, named as the engine names them.
enum ImageFormat {
  jxl('JPEG XL', 'jxl'),
  avif('AVIF', 'avif'),
  webp('WebP', 'webp'),
  jpeg('JPEG', 'jpg'),
  png('PNG', 'png'),
  gif('GIF', 'gif'),
  tiff('TIFF', 'tiff'),
  bmp('BMP', 'bmp'),
  qoi('QOI', 'qoi'),
  ico('ICO', 'ico'),
  tga('TGA', 'tga'),
  pnm('PNM', 'ppm'),
  heic('HEIC', 'heic');

  const ImageFormat(this.label, this.extension);
  final String label;
  final String extension;

  static ImageFormat parse(Object? name) => values.firstWhere((f) => f.name == name, orElse: () => jxl);

  /// Formats offered as conversion targets, most useful first.
  static const targets = [jxl, avif, webp, jpeg, png, gif, tiff, bmp, qoi, ico, tga, pnm];

  bool get supportsAlpha => this != jpeg && this != bmp && this != pnm;

  /// Formats where 100 means "lossless" (otherwise 100 is just "best").
  bool get hasLossless => const {jxl, webp, png, gif, tiff, bmp, qoi, ico, tga, pnm, avif}.contains(this);

  /// Formats with no quality setting at all.
  bool get alwaysLossless => const {png, gif, tiff, bmp, qoi, ico, tga, pnm}.contains(this);

  /// File extensions the engine can read.
  static const readableExtensions = [
    'jxl', 'jpg', 'jpeg', 'png', 'webp', 'gif', 'tif', 'tiff', 'bmp', //
    'qoi', 'ico', 'tga', 'ppm', 'pgm', 'pbm', 'pam', 'pnm',
  ];
}

/// How faithful an output is to its input (see `convert.rs`).
enum Fidelity {
  /// The original file can be restored bit for bit.
  reversible,

  /// Identical pixels.
  pixelExact,

  /// Re-encoded with some loss.
  lossy;

  static Fidelity parse(Object? s) => switch (s) {
    'reversible' => reversible,
    'pixel_exact' => pixelExact,
    _ => lossy,
  };
}

/// What to do with EXIF/XMP metadata.
enum MetadataPolicy {
  keep('keep'),
  stripGps('strip_gps'),
  stripAll('strip_all');

  const MetadataPolicy(this.wire);
  final String wire;

  static MetadataPolicy parse(String? s) => values.firstWhere((m) => m.wire == s, orElse: () => stripGps);
}

/// Encoder speed presets, mapped to JPEG XL effort.
enum SpeedPreset {
  fast(3),
  balanced(7),
  max(9);

  const SpeedPreset(this.effort);
  final int effort;
}

/// A point in image pixels.
class PixelPoint {
  const PixelPoint(this.x, this.y);
  final int x;
  final int y;

  Map<String, Object?> toJson() => {'x': x, 'y': y};
  factory PixelPoint.fromJson(Map<String, Object?> j) => PixelPoint((j['x']! as num).toInt(), (j['y']! as num).toInt());
}

/// A rectangle in image pixels.
class PixelRect {
  const PixelRect(this.x, this.y, this.width, this.height);
  final int x;
  final int y;
  final int width;
  final int height;

  Map<String, Object?> toJson() => {'x': x, 'y': y, 'width': width, 'height': height};
  factory PixelRect.fromJson(Map<String, Object?> j) => PixelRect(
    (j['x']! as num).toInt(),
    (j['y']! as num).toInt(),
    (j['width']! as num).toInt(),
    (j['height']! as num).toInt(),
  );
}

/// Conversion settings (Rust: `ConvertOptions`).
class ConvertSettings {
  const ConvertSettings({
    this.target = ImageFormat.jxl,
    this.quality = 90,
    this.jpegRecompression = true,
    this.effort = 7,
    this.progressive = false,
    this.focus,
    this.maxWidth,
    this.maxHeight,
    this.metadata = MetadataPolicy.stripGps,
  });

  final ImageFormat target;

  /// 0–100; 100 = lossless where possible.
  final double quality;
  final bool jpegRecompression;
  final int effort;
  final bool progressive;
  final PixelPoint? focus;
  final int? maxWidth;
  final int? maxHeight;
  final MetadataPolicy metadata;

  bool get lossless => quality >= 100;

  ConvertSettings copyWith({
    ImageFormat? target,
    double? quality,
    bool? jpegRecompression,
    int? effort,
    bool? progressive,
    PixelPoint? Function()? focus,
    int? Function()? maxWidth,
    MetadataPolicy? metadata,
  }) => ConvertSettings(
    target: target ?? this.target,
    quality: quality ?? this.quality,
    jpegRecompression: jpegRecompression ?? this.jpegRecompression,
    effort: effort ?? this.effort,
    progressive: progressive ?? this.progressive,
    focus: focus != null ? focus() : this.focus,
    maxWidth: maxWidth != null ? maxWidth() : this.maxWidth,
    maxHeight: maxHeight,
    metadata: metadata ?? this.metadata,
  );

  Map<String, Object?> toJson() => {
    'target': target.name,
    'quality': quality,
    'jpeg_recompression': jpegRecompression,
    'effort': effort,
    'progressive': progressive,
    'focus': focus?.toJson(),
    'resize': (maxWidth == null && maxHeight == null) ? null : {'max_width': maxWidth, 'max_height': maxHeight},
    'metadata': metadata.wire,
  };
}

/// The outcome of a conversion (Rust: `ConvertOutput`).
class ConvertResult {
  ConvertResult(Map<String, Object?> j)
    : format = ImageFormat.parse(j['format']),
      sourceFormat = ImageFormat.parse(j['source_format']),
      width = (j['width']! as num).toInt(),
      height = (j['height']! as num).toInt(),
      frames = (j['frames']! as num).toInt(),
      inputBytes = (j['input_bytes']! as num).toInt(),
      outputBytes = (j['output_bytes']! as num).toInt(),
      fidelity = Fidelity.parse(j['fidelity']),
      notes = [for (final n in j['notes']! as List) n as String];

  final ImageFormat format;
  final ImageFormat sourceFormat;
  final int width;
  final int height;
  final int frames;
  final int inputBytes;
  final int outputBytes;
  final Fidelity fidelity;
  final List<String> notes;

  /// −35.0 means 35% smaller.
  double get savingPercent => inputBytes == 0 ? 0 : (outputBytes / inputBytes - 1) * 100;
}

/// Decoded pixels for display (RGBA, 8 bits).
class RgbaPreview {
  RgbaPreview(this.pixels, this.width, this.height);

  /// Builds a preview from a `{buffer, width, height}` JSON entry.
  factory RgbaPreview.fromJson(Map<String, Object?> j, List<Uint8List> buffers) =>
      RgbaPreview(buffers[(j['buffer']! as num).toInt()], (j['width']! as num).toInt(), (j['height']! as num).toInt());

  final Uint8List pixels;
  final int width;
  final int height;

  /// A GPU-ready image for `RawImage`.
  Future<ui.Image> toImage() {
    final completer = Completer<ui.Image>();
    ui.decodeImageFromPixels(pixels, width, height, ui.PixelFormat.rgba8888, completer.complete);
    return completer.future;
  }
}

/// What is inside a file (Rust: `Inspection`).
class Inspection {
  Inspection(this.raw);
  final Map<String, Object?> raw;

  ImageFormat get format => ImageFormat.parse(raw['format']);
  int get fileBytes => (raw['file_bytes']! as num).toInt();
  int get width => (raw['width']! as num).toInt();
  int get height => (raw['height']! as num).toInt();
  int get bitsPerSample => (raw['bits_per_sample']! as num).toInt();
  bool get hasAlpha => raw['has_alpha'] == true;
  bool get alphaUsed => raw['alpha_used'] == true;
  int get frames => (raw['frames']! as num).toInt();
  bool get hasIcc => raw['has_icc'] == true;
  bool get hasExif => raw['has_exif'] == true;
  bool get hasXmp => raw['has_xmp'] == true;
  bool get hasGps => raw['has_gps'] == true;
  double get bitsPerPixel => (raw['bits_per_pixel']! as num).toDouble();

  /// English advice (see [tips] for localisable codes).
  List<String> get suggestions => [for (final s in raw['suggestions']! as List) s as String];

  /// Advice as `{code: …}` maps (Rust: `Tip`), shown in the UI language.
  List<Map<String, Object?>> get tips => [for (final t in raw['tips']! as List) t as Map<String, Object?>];

  /// JPEG XL details (Rust: `JxlProbe`), or null for other formats.
  Map<String, Object?>? get jxl => raw['jxl'] as Map<String, Object?>?;
}

/// The main subject of an image (Rust: `Subject`).
class Subject {
  Subject(Map<String, Object?> j)
    : bbox = PixelRect.fromJson(j['bbox']! as Map<String, Object?>),
      focus = PixelPoint.fromJson(j['focus']! as Map<String, Object?>),
      coverage = (j['coverage']! as num).toDouble(),
      source = j['source']! as String,
      raw = j;

  final PixelRect bbox;
  final PixelPoint focus;
  final double coverage;

  /// `detected`, `manual` or `center`.
  final String source;
  final Map<String, Object?> raw;
}

/// One contender in a format battle.
class BattleEntry {
  BattleEntry(Map<String, Object?> j)
    : format = ImageFormat.parse(j['format']),
      quality = (j['quality']! as num).toDouble(),
      bytes = (j['bytes']! as num).toInt(),
      score = (j['score'] as num?)?.toDouble(),
      bpp = (j['bpp']! as num).toDouble(),
      vsJpegPercent = (j['vs_jpeg_percent'] as num?)?.toDouble(),
      note = j['note'] as String?;

  final ImageFormat format;
  final double quality;
  final int bytes;
  final double? score;
  final double bpp;
  final double? vsJpegPercent;
  final String? note;
}

/// Plain-language description of a SSIMULACRA2 score.
enum ScoreBand { visuallyLossless, veryHigh, high, medium, low }

ScoreBand scoreBand(double score) => switch (score) {
  >= 90 => ScoreBand.visuallyLossless,
  >= 80 => ScoreBand.veryHigh,
  >= 70 => ScoreBand.high,
  >= 50 => ScoreBand.medium,
  _ => ScoreBand.low,
};
