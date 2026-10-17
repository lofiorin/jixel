/// Convert: a queue of files, one set of settings, one button.
library;

import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;

import '../../engine/engine.dart';
import '../../l10n/app_localizations.dart';
import '../../state/app_state.dart';
import '../../state/convert_queue.dart';
import '../format.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/controls.dart';
import '../widgets/file_input.dart';
import '../widgets/image_views.dart';

class ConvertScreen extends StatefulWidget {
  const ConvertScreen({super.key});

  @override
  State<ConvertScreen> createState() => _ConvertScreenState();
}

class _ConvertScreenState extends State<ConvertScreen> {
  final _queue = ConvertQueue();
  late ConvertSettings _settings;
  bool _advanced = false;
  AppState? _state;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final state = AppScope.of(context);
    if (_state == null) {
      final s = state.settings;
      _settings = ConvertSettings(quality: s.quality, effort: s.speed.effort, metadata: s.metadata);
    }
    _state = state;
    // Pick up files handed over from Home.
    final files = state.take(Section.convert);
    if (files.isNotEmpty) _queue.add(files);
  }

  @override
  void dispose() {
    _queue.dispose();
    super.dispose();
  }

  void _set(ConvertSettings s) => setState(() => _settings = s);

  Future<void> _run() async {
    await _queue.run(_settings, _state!.settings.outputDir);
  }

  Future<void> _showTrial(QueueItem item) async {
    await showDialog<void>(
      context: context,
      builder: (_) => _TrialDialog(input: item.input, settings: _settings),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return ListenableBuilder(
      listenable: _queue,
      builder: (context, _) => PageBody(
        title: l.navConvert,
        actions: [
          if (_queue.items.isNotEmpty && !_queue.running) TextButton(onPressed: _queue.clear, child: Text(l.clearList)),
        ],
        children: [
          if (_queue.items.isEmpty)
            DropZone(onFiles: _queue.add)
          else ...[
            DropZone(onFiles: _queue.add, compact: true),
            Gap.m,
            _QueueList(queue: _queue, onPreview: _showTrial),
            Gap.m,
            _Totals(queue: _queue),
          ],
          Gap.l,
          SectionCard(
            title: l.formatLabel,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                FormatPicker(
                  value: _settings.target,
                  onChanged: (f) => _set(_settings.copyWith(target: f)),
                ),
                if (!_settings.target.alwaysLossless) ...[
                  Gap.l,
                  QualitySlider(
                    value: _settings.quality,
                    format: _settings.target,
                    onChanged: (q) => _set(_settings.copyWith(quality: q)),
                  ),
                ],
                if (_settings.target == ImageFormat.jxl && _settings.jpegRecompression) ...[
                  Gap.s,
                  _Hint(icon: Icons.verified_outlined, text: l.jpegLosslessHint),
                ],
                Gap.m,
                Align(
                  alignment: Alignment.centerLeft,
                  child: TextButton.icon(
                    onPressed: () => setState(() => _advanced = !_advanced),
                    icon: Icon(_advanced ? Icons.expand_less : Icons.expand_more),
                    label: Text(l.advanced),
                  ),
                ),
                if (_advanced) _Advanced(settings: _settings, onChanged: _set),
              ],
            ),
          ),
          Gap.l,
          FilledButton.icon(
            onPressed: _queue.pending == 0 || _queue.running ? null : _run,
            icon: _queue.running
                ? const SizedBox.square(dimension: 18, child: CircularProgressIndicator(strokeWidth: 2))
                : const Icon(Icons.bolt),
            label: Text(_queue.items.isEmpty ? l.emptyQueue : l.convertCount(_queue.pending)),
          ),
        ],
      ),
    );
  }
}

class _Advanced extends StatelessWidget {
  const _Advanced({required this.settings, required this.onChanged});
  final ConvertSettings settings;
  final ValueChanged<ConvertSettings> onChanged;

  static const _widths = [null, 3840, 2560, 1920, 1280, 800];

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final isJxl = settings.target == ImageFormat.jxl;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Gap.s,
        if (isJxl) ...[
          SpeedPicker(
            value: SpeedPreset.values.firstWhere(
              (s) => s.effort == settings.effort,
              orElse: () => SpeedPreset.balanced,
            ),
            onChanged: (s) => onChanged(settings.copyWith(effort: s.effort)),
          ),
          Gap.m,
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(l.progressiveLabel),
            subtitle: Text(l.progressiveHint),
            value: settings.progressive,
            onChanged: (v) => onChanged(settings.copyWith(progressive: v)),
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(l.reencodeJpeg),
            value: !settings.jpegRecompression,
            onChanged: (v) => onChanged(settings.copyWith(jpegRecompression: !v)),
          ),
          Gap.s,
        ],
        Text(l.maxWidthLabel, style: Theme.of(context).textTheme.labelLarge),
        Gap.s,
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final w in _widths)
              ChoiceChip(
                label: Text(w == null ? l.noLimit : '$w px'),
                selected: settings.maxWidth == w,
                onSelected: (_) => onChanged(settings.copyWith(maxWidth: () => w)),
              ),
          ],
        ),
        Gap.m,
        MetadataPicker(
          value: settings.metadata,
          onChanged: (m) => onChanged(settings.copyWith(metadata: m)),
        ),
      ],
    );
  }
}

class _QueueList extends StatelessWidget {
  const _QueueList({required this.queue, required this.onPreview});
  final ConvertQueue queue;
  final ValueChanged<QueueItem> onPreview;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final scheme = Theme.of(context).colorScheme;
    return Card(
      child: Column(
        children: [
          for (final (i, item) in queue.items.indexed) ...[
            if (i > 0) const Divider(height: 1, indent: 16, endIndent: 16),
            ListTile(
              leading: switch (item.status) {
                ItemStatus.waiting => Icon(Icons.image_outlined, color: scheme.onSurfaceVariant),
                ItemStatus.working => const SizedBox.square(
                  dimension: 22,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
                ItemStatus.done => Icon(Icons.check_circle, color: successColor(context)),
                ItemStatus.failed => Icon(Icons.error_outline, color: scheme.error),
              },
              title: Text(item.name, maxLines: 1, overflow: TextOverflow.ellipsis),
              subtitle: switch (item.status) {
                ItemStatus.waiting => Text('${formatBytes(item.inputBytes)} · ${l.statusWaiting}'),
                ItemStatus.working => Text(l.statusWorking),
                ItemStatus.failed => Text(item.error ?? l.statusFailed, style: TextStyle(color: scheme.error)),
                ItemStatus.done => _ResultLine(result: item.result!),
              },
              trailing: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (item.status == ItemStatus.done)
                    IconButton(
                      tooltip: isDesktop ? l.showInFolder : l.share,
                      icon: Icon(isDesktop ? Icons.folder_open_outlined : Icons.ios_share),
                      onPressed: () => revealOrShare(item.output!),
                    )
                  else if (!queue.running) ...[
                    IconButton(
                      tooltip: l.beforeAfter,
                      icon: const Icon(Icons.compare_outlined),
                      onPressed: () => onPreview(item),
                    ),
                    IconButton(icon: const Icon(Icons.close), onPressed: () => queue.remove(item)),
                  ],
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _ResultLine extends StatelessWidget {
  const _ResultLine({required this.result});
  final ConvertResult result;

  @override
  Widget build(BuildContext context) {
    final saving = result.savingPercent;
    return Padding(
      padding: const EdgeInsets.only(top: 4),
      child: Wrap(
        spacing: 8,
        runSpacing: 4,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          Text('${formatBytes(result.inputBytes)} → ${formatBytes(result.outputBytes)}'),
          Text(
            formatPercent(saving),
            style: TextStyle(
              fontWeight: FontWeight.w600,
              color: saving < 0 ? successColor(context) : Theme.of(context).colorScheme.error,
            ),
          ),
          FidelityPill(result.fidelity),
        ],
      ),
    );
  }
}

class _Totals extends StatelessWidget {
  const _Totals({required this.queue});
  final ConvertQueue queue;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final (before, after) = queue.totals;
    if (before == 0) return const SizedBox.shrink();
    final diff = after - before;
    final percent = formatPercentAbs((after / before - 1) * 100);
    final dir = queue.items.firstWhere((i) => i.output != null).output!;
    return Row(
      children: [
        Expanded(
          child: Text(
            diff <= 0 ? l.totalSaving(formatBytes(-diff), percent) : l.totalGrowth(formatBytes(diff), percent),
            style: Theme.of(context).textTheme.titleMedium?.copyWith(color: diff <= 0 ? successColor(context) : null),
          ),
        ),
        if (isDesktop)
          TextButton.icon(
            onPressed: () => revealOrShare(p.dirname(dir)),
            icon: const Icon(Icons.folder_open_outlined),
            label: Text(l.showInFolder),
          ),
      ],
    );
  }
}

class _Hint extends StatelessWidget {
  const _Hint({required this.icon, required this.text});
  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) {
    final color = successColor(context);
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(icon, size: 18, color: color),
        Gap.s,
        Expanded(
          child: Text(text, style: Theme.of(context).textTheme.bodySmall?.copyWith(color: color)),
        ),
      ],
    );
  }
}

/// Encodes one file in memory and shows it next to the original.
class _TrialDialog extends StatefulWidget {
  const _TrialDialog({required this.input, required this.settings});
  final String input;
  final ConvertSettings settings;

  @override
  State<_TrialDialog> createState() => _TrialDialogState();
}

class _TrialDialogState extends State<_TrialDialog> {
  late final Future<(RgbaPreview, Trial)> _work = () async {
    final results = await Future.wait([
      Engine.preview(widget.input, maxSide: 1600),
      Engine.trial(widget.input, widget.settings, maxSide: 1600),
    ]);
    return (results[0] as RgbaPreview, results[1] as Trial);
  }();

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return Dialog(
      insetPadding: const EdgeInsets.all(24),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 1100),
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: FutureBuilder(
            future: _work,
            builder: (context, snap) {
              if (snap.hasError) return Text(l.errorGeneric(snap.error.toString()));
              if (!snap.hasData) {
                return const SizedBox(height: 240, child: Center(child: CircularProgressIndicator()));
              }
              final (original, trial) = snap.data!;
              return Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      Expanded(child: Text(l.beforeAfter, style: Theme.of(context).textTheme.titleLarge)),
                      IconButton(icon: const Icon(Icons.close), onPressed: () => Navigator.pop(context)),
                    ],
                  ),
                  Gap.m,
                  Flexible(
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(12),
                      child: CompareSlider(
                        before: original,
                        after: trial.preview,
                        beforeLabel: l.original,
                        afterLabel: '${trial.result.format.label} · ${formatBytes(trial.result.outputBytes)}',
                      ),
                    ),
                  ),
                  Gap.m,
                  Wrap(
                    spacing: 12,
                    runSpacing: 8,
                    crossAxisAlignment: WrapCrossAlignment.center,
                    children: [
                      _ResultLine(result: trial.result),
                      if (trial.score case final score?)
                        Tooltip(
                          message: l.scoreExplain,
                          child: Pill(
                            '${l.scoreLabel}: ${score.toStringAsFixed(1)} · ${scoreText(l, score)}',
                            icon: Icons.insights,
                          ),
                        ),
                    ],
                  ),
                ],
              );
            },
          ),
        ),
      ),
    );
  }
}
