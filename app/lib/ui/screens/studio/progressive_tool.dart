/// Loading: what a visitor sees while a progressive JPEG XL downloads.
library;

import 'package:flutter/material.dart';

import '../../../engine/engine.dart';
import '../../../l10n/app_localizations.dart';
import '../../format.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/image_views.dart';
import '../studio.dart';

class ProgressiveTool extends StatefulWidget {
  const ProgressiveTool({super.key, required this.image});
  final StudioImage image;

  @override
  State<ProgressiveTool> createState() => _ProgressiveToolState();
}

class _ProgressiveToolState extends State<ProgressiveTool> with AutomaticKeepAliveClientMixin {
  static const _fractions = [0.05, 0.10, 0.15, 0.20, 0.30, 0.50, 0.75, 1.0];

  List<ProgressiveStep>? _steps;
  int _index = 3;
  Object? _error;

  @override
  bool get wantKeepAlive => true;

  @override
  void initState() {
    super.initState();
    _run();
  }

  Future<void> _run() async {
    try {
      final jxl = await Engine.encodeJxl(
        widget.image.path,
        const ConvertSettings(quality: 85, progressive: true, jpegRecompression: false),
      );
      final steps = await Engine.progressive(jxl, _fractions);
      if (mounted) setState(() => _steps = steps);
    } on EngineException catch (e) {
      if (mounted) setState(() => _error = e.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    final l = AppLocalizations.of(context);
    final steps = _steps;
    if (_error != null) return ToolBody(children: [Text(l.errorGeneric(_error.toString()))]);
    if (steps == null) {
      return const ToolBody(
        children: [
          Padding(
            padding: EdgeInsets.all(60),
            child: Center(child: CircularProgressIndicator()),
          ),
        ],
      );
    }
    final step = steps[_index];
    final aspect = widget.image.preview.width / widget.image.preview.height;
    return ToolBody(
      intro: l.progressiveIntro,
      children: [
        PreviewFrame(
          child: AspectRatio(
            aspectRatio: aspect,
            child: step.preview == null
                ? ColoredBox(
                    color: Theme.of(context).colorScheme.surfaceContainerHighest,
                    child: Center(child: Text(l.nothingYet)),
                  )
                : PreviewImage(preview: step.preview!, fit: BoxFit.fill),
          ),
        ),
        Gap.m,
        Text(
          l.downloaded(formatPercentAbs(step.fraction * 100), formatBytes(step.bytes)),
          style: Theme.of(context).textTheme.titleMedium,
          textAlign: TextAlign.center,
        ),
        Slider(
          value: _index.toDouble(),
          max: steps.length - 1,
          divisions: steps.length - 1,
          label: formatPercentAbs(step.fraction * 100),
          onChanged: (v) => setState(() => _index = v.round()),
        ),
        Gap.s,
        Center(child: Pill(l.progressiveHint, icon: Icons.downloading)),
      ],
    );
  }
}
