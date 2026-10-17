/// Subject: find the main subject, load it first and give it more detail.
library;

import 'package:flutter/material.dart';

import '../../../engine/engine.dart';
import '../../../l10n/app_localizations.dart';
import '../../../state/app_state.dart';
import '../../../state/convert_queue.dart' show uniquePath;
import '../../format.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/rect_selector.dart';
import '../studio.dart';

class SubjectTool extends StatefulWidget {
  const SubjectTool({super.key, required this.image});
  final StudioImage image;

  @override
  State<SubjectTool> createState() => _SubjectToolState();
}

class _SubjectToolState extends State<SubjectTool> with AutomaticKeepAliveClientMixin {
  Subject? _subject;
  bool _detecting = false;
  bool _busy = false;
  BoostReport? _report;
  String? _saved;

  @override
  bool get wantKeepAlive => true;

  @override
  void initState() {
    super.initState();
    _detect();
  }

  Future<void> _detect() async {
    setState(() => _detecting = true);
    try {
      final s = await Engine.detectSubject(widget.image.path);
      if (mounted) {
        setState(() {
          _subject = s;
          _report = null;
        });
      }
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    } finally {
      if (mounted) setState(() => _detecting = false);
    }
  }

  /// The user drew a rectangle: it becomes a manual subject.
  void _manual(PixelRect r) {
    final info = widget.image.info;
    setState(() {
      _report = null;
      _subject = Subject({
        'bbox': r.toJson(),
        'focus': PixelPoint(r.x + r.width ~/ 2, r.y + r.height ~/ 2).toJson(),
        'coverage': r.width * r.height / (info.width * info.height),
        'source': 'manual',
      });
    });
  }

  Future<void> _boost() async {
    final dir = AppScope.read(context).settings.outputDir;
    final out = uniquePath(dir, '${widget.image.stem}-subject', 'jxl');
    setState(() => _busy = true);
    try {
      final report = await Engine.subjectBoost(widget.image.path, out, subject: _subject);
      if (mounted) {
        setState(() {
          _report = report;
          _saved = out;
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
    final report = _report;
    return ToolBody(
      intro: l.subjectIntro,
      children: [
        PreviewFrame(
          child: RectSelector(
            preview: widget.image.preview,
            imageWidth: widget.image.info.width,
            imageHeight: widget.image.info.height,
            rect: _subject?.bbox,
            onChanged: _manual,
          ),
        ),
        Gap.s,
        Text(l.cropHint, style: Theme.of(context).textTheme.bodySmall, textAlign: TextAlign.center),
        Gap.l,
        Wrap(
          spacing: 12,
          runSpacing: 12,
          alignment: WrapAlignment.center,
          children: [
            OutlinedButton.icon(
              onPressed: _detecting ? null : _detect,
              icon: const Icon(Icons.auto_fix_high),
              label: Text(l.detectSubject),
            ),
            BusyButton(busy: _busy, onPressed: _subject == null ? null : _boost, icon: Icons.bolt, label: l.boostRun),
          ],
        ),
        if (report != null) ...[
          Gap.l,
          Row(
            children: [
              Expanded(
                child: _Option(
                  title: l.boostStandard,
                  bytes: report.singleBytes,
                  score: report.singleSubjectScore,
                  chosen: !report.recommendBoost,
                ),
              ),
              Gap.m,
              Expanded(
                child: report.boostedBytes == null
                    ? SectionCard(title: l.boostLayered, child: Text(l.boostWholeImage))
                    : _Option(
                        title: l.boostLayered,
                        bytes: report.boostedBytes!,
                        score: report.boostedSubjectScore!,
                        chosen: report.recommendBoost,
                      ),
              ),
            ],
          ),
          Gap.m,
          Text(report.recommendBoost ? l.boostRecommended : l.boostNotWorth),
          if (_saved case final path?) ...[Gap.m, SavedFile(path: path)],
        ],
      ],
    );
  }
}

class _Option extends StatelessWidget {
  const _Option({required this.title, required this.bytes, required this.score, required this.chosen});
  final String title;
  final int bytes;
  final double score;
  final bool chosen;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final scheme = Theme.of(context).colorScheme;
    return Card(
      color: chosen ? scheme.primaryContainer : null,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(child: Text(title, style: Theme.of(context).textTheme.titleMedium)),
                if (chosen) Icon(Icons.check_circle, color: scheme.primary),
              ],
            ),
            Gap.s,
            Text(formatBytes(bytes), style: Theme.of(context).textTheme.headlineSmall),
            Text(l.boostSubjectScore(score.toStringAsFixed(1))),
          ],
        ),
      ),
    );
  }
}
