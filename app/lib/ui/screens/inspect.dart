/// Inspect: what is inside a file, and what to do about it.
library;

import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;

import '../../engine/engine.dart';
import '../../l10n/app_localizations.dart';
import '../../state/app_state.dart';
import '../../state/convert_queue.dart' show uniquePath;
import '../format.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/file_input.dart';
import '../widgets/image_views.dart';

class InspectScreen extends StatefulWidget {
  const InspectScreen({super.key});

  @override
  State<InspectScreen> createState() => _InspectScreenState();
}

class _InspectScreenState extends State<InspectScreen> {
  String? _path;
  Inspection? _info;
  RgbaPreview? _preview;
  String? _restored;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final files = AppScope.of(context).take(Section.inspect);
    if (files.isNotEmpty) _load(files.first);
  }

  Future<void> _load(String path) async {
    setState(() {
      _path = path;
      _info = null;
      _preview = null;
      _restored = null;
    });
    try {
      final info = await Engine.inspect(path);
      if (mounted) setState(() => _info = info);
      final preview = await Engine.preview(path, maxSide: 800);
      if (mounted && _path == path) setState(() => _preview = preview);
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    }
  }

  Future<void> _restore() async {
    final dir = AppScope.read(context).settings.outputDir;
    final out = uniquePath(dir, p.basenameWithoutExtension(_path!), 'jpg');
    try {
      await Engine.reconstructJpeg(_path!, out);
      if (mounted) setState(() => _restored = out);
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    }
  }

  /// A tip code from the engine, in the UI language.
  String _tip(AppLocalizations l, Map<String, Object?> tip) => switch (tip['code']) {
    'jpeg' => l.tipJpeg,
    'lossless_png' => l.tipLosslessPng,
    'animated_gif' => l.tipAnimatedGif,
    'webp' => l.tipWebp,
    'gps' => l.tipGps,
    'opaque_alpha' => l.tipOpaqueAlpha,
    'reconstruct' => l.tipReconstruct,
    'hdr' => l.tipHdr,
    'large' => l.tipLarge(tip['width']!, tip['height']!),
    final other => '$other',
  };

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final info = _info;
    return PageBody(
      title: l.navInspect,
      actions: [
        if (_path != null)
          OutlinedButton(
            onPressed: () async {
              final files = await pickImages(context, multiple: false);
              if (files.isNotEmpty) _load(files.first);
            },
            child: Text(l.changeImage),
          ),
      ],
      children: [
        if (_path == null) ...[
          Text(l.inspectEmpty),
          Gap.m,
          DropZone(multiple: false, onFiles: (f) => _load(f.first)),
        ] else if (info == null)
          const Padding(
            padding: EdgeInsets.all(60),
            child: Center(child: CircularProgressIndicator()),
          )
        else
          LayoutBuilder(
            builder: (context, box) {
              final details = _details(l, info);
              final preview = _preview == null
                  ? const SizedBox(height: 200, child: Center(child: CircularProgressIndicator()))
                  : ClipRRect(
                      borderRadius: BorderRadius.circular(16),
                      child: PreviewImage(preview: _preview!, checkerboard: info.alphaUsed),
                    );
              final side = box.maxWidth >= 760;
              return Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  if (side)
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Expanded(child: preview),
                        Gap.l,
                        Expanded(child: details),
                      ],
                    )
                  else ...[
                    preview,
                    Gap.m,
                    details,
                  ],
                  if (info.tips.isNotEmpty) ...[
                    Gap.l,
                    SectionCard(
                      title: l.suggestions,
                      child: Column(
                        children: [
                          for (final tip in info.tips)
                            ListTile(
                              contentPadding: EdgeInsets.zero,
                              leading: Icon(Icons.lightbulb_outline, color: Theme.of(context).colorScheme.primary),
                              title: Text(_tip(l, tip)),
                            ),
                        ],
                      ),
                    ),
                  ],
                ],
              );
            },
          ),
      ],
    );
  }

  Widget _details(AppLocalizations l, Inspection info) {
    String yesNo(bool v) => v ? l.yes : l.no;
    final jxl = info.jxl;
    final rows = <(String, String)>[
      (l.formatLabel, info.format.label),
      (l.dimensions, '${info.width} × ${info.height}'),
      (l.fileSize, formatBytes(info.fileBytes)),
      (l.bitsPerPixel, info.bitsPerPixel.toStringAsFixed(2)),
      (l.bitDepth, '${info.bitsPerSample} bit'),
      (l.transparency, info.hasAlpha ? (info.alphaUsed ? l.yes : l.unused) : l.no),
      (l.colorProfile, info.hasIcc ? l.embedded : l.srgbDefault),
      (l.exif, yesNo(info.hasExif)),
      (l.location, yesNo(info.hasGps)),
      if (info.frames > 1) (l.frames, '${info.frames}'),
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SectionCard(
          title: p.basename(_path!),
          child: Column(children: [for (final (k, v) in rows) _Row(k, v)]),
        ),
        if (jxl != null) ...[
          Gap.m,
          SectionCard(
            title: l.jxlDetails,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _Row(l.jxlMode, jxl['uses_original_profile'] == true ? l.jxlLossless : l.jxlLossy),
                _Row(l.colorProfile, '${jxl['color_description']}'),
                _Row(
                  l.hdr,
                  yesNo(
                    ((jxl['intensity_target'] as num?) ?? 0) > 255 || '${jxl['color_description']}'.contains('HDR'),
                  ),
                ),
                _Row(l.jxlContainsJpeg, yesNo(jxl['jpeg_reconstruction'] == true)),
                if (jxl['jpeg_reconstruction'] == true) ...[
                  Gap.m,
                  FilledButton.tonalIcon(
                    onPressed: _restore,
                    icon: const Icon(Icons.restore),
                    label: Text(l.restoreJpeg),
                  ),
                ],
                if (_restored case final path?) ...[
                  Gap.s,
                  Row(
                    children: [
                      Icon(Icons.check_circle, color: successColor(context), size: 18),
                      Gap.s,
                      Expanded(child: Text(l.savedTo(p.basename(path)))),
                      TextButton(
                        onPressed: () => revealOrShare(path),
                        child: Text(isDesktop ? l.showInFolder : l.share),
                      ),
                    ],
                  ),
                ],
              ],
            ),
          ),
        ],
      ],
    );
  }
}

class _Row extends StatelessWidget {
  const _Row(this.label, this.value);
  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 6),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: Text(label, style: TextStyle(color: Theme.of(context).colorScheme.onSurfaceVariant)),
        ),
        Expanded(
          child: Text(
            value,
            textAlign: TextAlign.end,
            style: const TextStyle(fontWeight: FontWeight.w500),
          ),
        ),
      ],
    ),
  );
}
