/// Crop & tiles: pixel-exact crops, grids and Deep Zoom pyramids.
library;

import 'package:flutter/material.dart';

import '../../../engine/engine.dart';
import '../../../l10n/app_localizations.dart';
import '../../../state/app_state.dart';
import '../../../state/convert_queue.dart' show uniqueDir, uniquePath;
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/controls.dart';
import '../../widgets/rect_selector.dart';
import '../studio.dart';

class CropTool extends StatefulWidget {
  const CropTool({super.key, required this.image});
  final StudioImage image;

  @override
  State<CropTool> createState() => _CropToolState();
}

class _CropToolState extends State<CropTool> with AutomaticKeepAliveClientMixin {
  PixelRect? _rect;
  int _cols = 2;
  int _rows = 2;
  ImageFormat _format = ImageFormat.jxl;
  String? _busy;
  String? _saved;
  String? _detail;

  @override
  bool get wantKeepAlive => true;

  String get _dir => AppScope.read(context).settings.outputDir;

  Future<void> _do(String what, Future<(String, String?)> Function() job) async {
    setState(() => _busy = what);
    try {
      final (path, detail) = await job();
      if (mounted) {
        setState(() {
          _saved = path;
          _detail = detail;
        });
      }
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    } finally {
      if (mounted) setState(() => _busy = null);
    }
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    final l = AppLocalizations.of(context);
    final image = widget.image;
    return ToolBody(
      intro: l.cropIntro,
      children: [
        PreviewFrame(
          child: RectSelector(
            preview: image.preview,
            imageWidth: image.info.width,
            imageHeight: image.info.height,
            rect: _rect,
            onChanged: (r) => setState(() => _rect = r),
          ),
        ),
        Gap.s,
        Text(
          _rect == null ? l.cropHint : '${_rect!.width} × ${_rect!.height} px',
          style: Theme.of(context).textTheme.bodySmall,
          textAlign: TextAlign.center,
        ),
        Gap.l,
        SectionCard(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              FormatPicker(
                value: _format,
                formats: const [ImageFormat.jxl, ImageFormat.png, ImageFormat.webp],
                onChanged: (f) => setState(() => _format = f),
              ),
              Gap.m,
              BusyButton(
                busy: _busy == 'crop',
                icon: Icons.crop,
                label: l.cropButton,
                onPressed: _rect == null || _busy != null
                    ? null
                    : () => _do('crop', () async {
                        final out = uniquePath(_dir, '${image.stem}-crop', _format.extension);
                        await Engine.crop(image.path, out, _rect!);
                        return (out, null);
                      }),
              ),
            ],
          ),
        ),
        Gap.m,
        SectionCard(
          title: l.gridLabel(_cols, _rows),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _Stepper(icon: Icons.view_column_outlined, value: _cols, onChanged: (v) => setState(() => _cols = v)),
              _Stepper(icon: Icons.table_rows_outlined, value: _rows, onChanged: (v) => setState(() => _rows = v)),
              Gap.s,
              BusyButton(
                busy: _busy == 'tiles',
                icon: Icons.grid_view,
                label: l.tilesButton,
                onPressed: _busy != null
                    ? null
                    : () => _do('tiles', () async {
                        final dir = uniqueDir(_dir, '${image.stem}-tiles');
                        final n = await Engine.tiles(image.path, dir, cols: _cols, rows: _rows, format: _format);
                        return (dir, l.tilesDone(n));
                      }),
              ),
            ],
          ),
        ),
        Gap.m,
        SectionCard(
          title: l.deepZoomButton,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(l.deepZoomHint),
              Gap.m,
              BusyButton(
                busy: _busy == 'dzi',
                icon: Icons.zoom_in_map,
                label: l.deepZoomButton,
                onPressed: _busy != null
                    ? null
                    : () => _do('dzi', () async {
                        final dir = uniqueDir(_dir, '${image.stem}-deepzoom');
                        final (_, levels, tiles) = await Engine.deepZoom(image.path, dir, format: _format);
                        return (dir, l.deepZoomDone(levels, tiles));
                      }),
              ),
            ],
          ),
        ),
        if (_saved case final path?) ...[Gap.m, SavedFile(path: path, detail: _detail)],
      ],
    );
  }
}

class _Stepper extends StatelessWidget {
  const _Stepper({required this.icon, required this.value, required this.onChanged});
  final IconData icon;
  final int value;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) => Row(
    children: [
      Icon(icon),
      Expanded(
        child: Slider(
          value: value.toDouble(),
          min: 1,
          max: 8,
          divisions: 7,
          label: '$value',
          onChanged: (v) => onChanged(v.round()),
        ),
      ),
      SizedBox(width: 24, child: Text('$value', textAlign: TextAlign.end)),
    ],
  );
}
