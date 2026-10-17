/// Compare: see an encode next to the original, with a quality score.
library;

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../engine/engine.dart';
import '../../../l10n/app_localizations.dart';
import '../../../state/app_state.dart';
import '../../../state/convert_queue.dart' show uniquePath;
import '../../format.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/controls.dart';
import '../../widgets/image_views.dart';
import '../studio.dart';

class CompareTool extends StatefulWidget {
  const CompareTool({super.key, required this.image});
  final StudioImage image;

  @override
  State<CompareTool> createState() => _CompareToolState();
}

class _CompareToolState extends State<CompareTool> with AutomaticKeepAliveClientMixin {
  ImageFormat _format = ImageFormat.jxl;
  double _quality = 85;
  Trial? _trial;
  bool _busy = false;
  String? _saved;
  Timer? _debounce;

  /// Bumped on every request, so a slow old result never replaces a new one.
  int _generation = 0;

  static const _formats = [ImageFormat.jxl, ImageFormat.avif, ImageFormat.webp, ImageFormat.jpeg];

  @override
  bool get wantKeepAlive => true;

  @override
  void initState() {
    super.initState();
    _run();
  }

  @override
  void dispose() {
    _debounce?.cancel();
    super.dispose();
  }

  ConvertSettings get _settings => ConvertSettings(target: _format, quality: _quality, jpegRecompression: false);

  void _schedule() {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 350), _run);
  }

  Future<void> _run() async {
    final generation = ++_generation;
    setState(() {
      _busy = true;
      _saved = null;
    });
    try {
      final trial = await Engine.trial(widget.image.path, _settings, maxSide: 1600);
      if (mounted && generation == _generation) setState(() => _trial = trial);
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    } finally {
      if (mounted && generation == _generation) setState(() => _busy = false);
    }
  }

  Future<void> _save() async {
    final dir = AppScope.read(context).settings.outputDir;
    final out = uniquePath(dir, widget.image.stem, _format.extension);
    try {
      await Engine.convert(widget.image.path, out, _settings);
      if (mounted) setState(() => _saved = out);
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    final l = AppLocalizations.of(context);
    final trial = _trial;
    return ToolBody(
      children: [
        PreviewFrame(
          child: trial == null
              ? AspectRatio(
                  aspectRatio: widget.image.preview.width / widget.image.preview.height,
                  child: const Center(child: CircularProgressIndicator()),
                )
              : CompareSlider(
                  before: widget.image.preview,
                  after: trial.preview,
                  beforeLabel: l.original,
                  afterLabel: _format.label,
                ),
        ),
        Gap.m,
        if (trial != null)
          Wrap(
            spacing: 12,
            runSpacing: 8,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              Text(
                '${formatBytes(trial.result.outputBytes)} (${formatPercent(trial.result.savingPercent)})',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              if (trial.score case final score?)
                Tooltip(
                  message: l.scoreExplain,
                  child: Pill(
                    '${l.scoreLabel} ${score.toStringAsFixed(1)} · ${scoreText(l, score)}',
                    icon: Icons.insights,
                  ),
                ),
              if (_busy) const SizedBox.square(dimension: 16, child: CircularProgressIndicator(strokeWidth: 2)),
            ],
          ),
        Gap.l,
        SectionCard(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              FormatPicker(
                value: _format,
                formats: _formats,
                onChanged: (f) {
                  setState(() => _format = f);
                  _run();
                },
              ),
              Gap.m,
              QualitySlider(
                value: _quality,
                format: _format,
                min: 10,
                onChanged: (q) {
                  setState(() => _quality = q);
                  _schedule();
                },
              ),
              Gap.s,
              Align(
                alignment: Alignment.centerRight,
                child: FilledButton.tonalIcon(
                  onPressed: _save,
                  icon: const Icon(Icons.save_alt),
                  label: Text(l.saveResult),
                ),
              ),
            ],
          ),
        ),
        if (_saved case final path?) ...[Gap.m, SavedFile(path: path)],
      ],
    );
  }
}
