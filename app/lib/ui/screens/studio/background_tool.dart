/// Background: on-device background removal (U²-Netp, ~4.6 MB).
library;

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

enum _Fill { transparent, white, black }

class BackgroundTool extends StatefulWidget {
  const BackgroundTool({super.key, required this.image});
  final StudioImage image;

  @override
  State<BackgroundTool> createState() => _BackgroundToolState();
}

class _BackgroundToolState extends State<BackgroundTool> with AutomaticKeepAliveClientMixin {
  _Fill _fill = _Fill.transparent;
  bool _hard = false;
  ImageFormat _format = ImageFormat.png;
  bool _busy = false;
  RgbaPreview? _result;
  String? _saved;
  int? _bytes;

  @override
  bool get wantKeepAlive => true;

  List<ImageFormat> get _formats => _fill == _Fill.transparent
      ? const [ImageFormat.png, ImageFormat.jxl, ImageFormat.webp]
      : const [ImageFormat.jpeg, ImageFormat.png, ImageFormat.jxl, ImageFormat.webp];

  Future<void> _run() async {
    final dir = AppScope.read(context).settings.outputDir;
    final out = uniquePath(dir, '${widget.image.stem}-cutout', _format.extension);
    setState(() => _busy = true);
    try {
      final (bytes, preview) = await Engine.removeBackground(
        widget.image.path,
        out,
        format: _format,
        hardEdges: _hard,
        fill: switch (_fill) {
          _Fill.transparent => null,
          _Fill.white => [255, 255, 255],
          _Fill.black => [0, 0, 0],
        },
        maxSide: 1600,
      );
      if (mounted) {
        setState(() {
          _result = preview;
          _saved = out;
          _bytes = bytes;
        });
      }
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    final l = AppLocalizations.of(context);
    final available = AppScope.of(context).engineInfo?.backgroundRemoval ?? true;
    if (!available) return ToolBody(children: [Center(child: Text(l.bgUnavailable))]);
    final result = _result;
    return ToolBody(
      intro: l.taskBackgroundBody,
      children: [
        PreviewFrame(
          child: result == null
              ? PreviewImage(preview: widget.image.preview)
              : CompareSlider(
                  before: widget.image.preview,
                  after: result,
                  beforeLabel: l.original,
                  afterLabel: l.result,
                ),
        ),
        Gap.l,
        SectionCard(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SegmentedButton<_Fill>(
                segments: [
                  ButtonSegment(
                    value: _Fill.transparent,
                    label: Text(l.bgTransparent),
                    icon: const Icon(Icons.grid_4x4),
                  ),
                  ButtonSegment(value: _Fill.white, label: Text(l.bgWhite), icon: const Icon(Icons.square_outlined)),
                  ButtonSegment(value: _Fill.black, label: Text(l.bgBlack), icon: const Icon(Icons.square)),
                ],
                selected: {_fill},
                onSelectionChanged: (s) => setState(() {
                  _fill = s.first;
                  if (!_formats.contains(_format)) _format = _formats.first;
                }),
              ),
              Gap.m,
              FormatPicker(value: _format, formats: _formats, onChanged: (f) => setState(() => _format = f)),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(l.hardEdges),
                value: _hard,
                onChanged: (v) => setState(() => _hard = v),
              ),
              Gap.s,
              BusyButton(busy: _busy, onPressed: _run, icon: Icons.content_cut, label: l.removeBackgroundButton),
            ],
          ),
        ),
        if (result != null) ...[
          Gap.l,
          // The cut-out alone, on a checkerboard so transparency is visible.
          PreviewFrame(maxHeight: 360, child: PreviewImage(preview: result, checkerboard: true)),
          Gap.m,
          SavedFile(path: _saved!, detail: formatBytes(_bytes!)),
        ],
      ],
    );
  }
}
