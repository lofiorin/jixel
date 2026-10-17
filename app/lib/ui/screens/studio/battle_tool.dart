/// Format battle: every format tuned to the same visual quality.
library;

import 'package:flutter/material.dart';

import '../../../engine/engine.dart';
import '../../../l10n/app_localizations.dart';
import '../../format.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../studio.dart';

class BattleTool extends StatefulWidget {
  const BattleTool({super.key, required this.image});
  final StudioImage image;

  @override
  State<BattleTool> createState() => _BattleToolState();
}

class _BattleToolState extends State<BattleTool> with AutomaticKeepAliveClientMixin {
  double _target = 80;
  bool _busy = false;
  List<BattleEntry>? _entries;

  @override
  bool get wantKeepAlive => true;

  Future<void> _run() async {
    setState(() => _busy = true);
    try {
      final (_, _, entries) = await Engine.battle(widget.image.path, targetScore: _target);
      if (mounted) setState(() => _entries = entries);
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
    final entries = _entries;
    return ToolBody(
      intro: l.battleIntro,
      children: [
        SectionCard(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Text(l.battleTarget(_target.round()), style: Theme.of(context).textTheme.labelLarge),
                  const Spacer(),
                  Text(scoreText(l, _target), style: TextStyle(color: Theme.of(context).colorScheme.primary)),
                ],
              ),
              Slider(
                value: _target,
                min: 60,
                max: 95,
                divisions: 7,
                label: '${_target.round()}',
                onChanged: _busy ? null : (v) => setState(() => _target = v),
              ),
              Text(l.scoreExplain, style: Theme.of(context).textTheme.bodySmall),
              Gap.m,
              BusyButton(busy: _busy, onPressed: _run, icon: Icons.emoji_events_outlined, label: l.battleRun),
            ],
          ),
        ),
        if (entries != null && entries.isNotEmpty) ...[Gap.l, _Results(entries: entries)],
      ],
    );
  }
}

class _Results extends StatelessWidget {
  const _Results({required this.entries});
  final List<BattleEntry> entries;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final scheme = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;
    final scored = entries.where((e) => e.score != null).toList();
    final winner = scored.isEmpty ? null : scored.reduce((a, b) => a.bytes <= b.bytes ? a : b);
    final largest = entries.map((e) => e.bytes).reduce((a, b) => a > b ? a : b);
    return SectionCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (final e in entries) ...[
            Row(
              children: [
                SizedBox(width: 90, child: Text(e.format.label, style: text.titleSmall)),
                Expanded(
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(6),
                    child: LinearProgressIndicator(
                      value: e.bytes / largest,
                      minHeight: 14,
                      color: e == winner ? scheme.primary : scheme.outline,
                      backgroundColor: scheme.surfaceContainerHighest,
                    ),
                  ),
                ),
                Gap.m,
                SizedBox(
                  width: 80,
                  child: Text(formatBytes(e.bytes), textAlign: TextAlign.end, style: text.titleSmall),
                ),
              ],
            ),
            Padding(
              padding: const EdgeInsets.only(left: 90, top: 4, bottom: 14),
              child: Wrap(
                spacing: 8,
                runSpacing: 4,
                children: [
                  if (e == winner) Pill(l.winner, icon: Icons.emoji_events, color: successColor(context)),
                  Text(
                    e.score == null ? l.notScored : '${l.scoreLabel} ${e.score!.toStringAsFixed(1)}',
                    style: text.bodySmall,
                  ),
                  if (e.vsJpegPercent case final pct? when e.format != ImageFormat.jpeg)
                    Text(l.vsJpeg(formatPercent(pct)), style: text.bodySmall),
                  if (e.note case final note?)
                    Text(note, style: text.bodySmall?.copyWith(color: scheme.onSurfaceVariant)),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}
