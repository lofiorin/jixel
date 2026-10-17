/// Optimize for web: responsive JPEG XL + fallbacks + code for the site.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;

import '../../engine/engine.dart';
import '../../l10n/app_localizations.dart';
import '../../state/app_state.dart';
import '../../state/convert_queue.dart' show uniqueDir;
import '../format.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/controls.dart';
import '../widgets/file_input.dart';

/// Lower-case, dash-separated, safe for URLs and file names.
String slug(String s) {
  final out = s.toLowerCase().replaceAll(RegExp('[^a-z0-9]+'), '-').replaceAll(RegExp(r'^-+|-+$'), '');
  return out.isEmpty ? 'image' : out;
}

class WebExportScreen extends StatefulWidget {
  const WebExportScreen({super.key});

  @override
  State<WebExportScreen> createState() => _WebExportScreenState();
}

class _WebExportScreenState extends State<WebExportScreen> {
  static const _allWidths = [480, 768, 1080, 1440, 1920, 2560, 3840];

  String? _input;
  Inspection? _info;
  final _name = TextEditingController();
  final _alt = TextEditingController();
  final _baseUrl = TextEditingController(text: '/images/');
  Set<int> _widths = {480, 768, 1080, 1440, 1920, 2560};
  Set<ImageFormat> _fallbacks = {ImageFormat.avif, ImageFormat.jpeg};
  double _quality = 85;
  bool _hero = false;
  bool _autoFocus = true;
  bool _busy = false;
  WebExportResult? _result;
  String? _outputDir;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final files = AppScope.of(context).take(Section.web);
    if (files.isNotEmpty) _load(files.first);
  }

  @override
  void dispose() {
    _name.dispose();
    _alt.dispose();
    _baseUrl.dispose();
    super.dispose();
  }

  Future<void> _load(String path) async {
    setState(() {
      _input = path;
      _info = null;
      _result = null;
      _name.text = slug(p.basenameWithoutExtension(path));
    });
    try {
      final info = await Engine.inspect(path);
      if (mounted && _input == path) setState(() => _info = info);
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    }
  }

  Future<void> _export() async {
    final settings = AppScope.read(context).settings;
    final name = slug(_name.text);
    // A fresh folder per export, named after the image.
    final dir = uniqueDir(settings.outputDir, name);
    setState(() => _busy = true);
    try {
      final result = await Engine.webExport(_input!, dir, {
        'name': name,
        'widths': (_widths.toList()..sort()),
        'quality': _quality,
        'fallbacks': [
          for (final f in [ImageFormat.avif, ImageFormat.webp, ImageFormat.jpeg])
            if (_fallbacks.contains(f)) f.name,
        ],
        'effort': settings.speed.effort,
        'progressive': true,
        'base_url': _baseUrl.text.endsWith('/') ? _baseUrl.text : '${_baseUrl.text}/',
        'alt': _alt.text,
        'hero': _hero,
        'metadata': settings.metadata.wire,
      }, autoFocus: _autoFocus);
      if (mounted) {
        setState(() {
          _result = result;
          _outputDir = dir;
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
    final l = AppLocalizations.of(context);
    final input = _input;
    return PageBody(
      title: l.taskWebTitle,
      subtitle: l.webIntro,
      children: [
        if (input == null)
          DropZone(multiple: false, onFiles: (f) => _load(f.first))
        else ...[
          SectionCard(
            child: Row(
              children: [
                const Icon(Icons.image_outlined),
                Gap.m,
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(p.basename(input), style: Theme.of(context).textTheme.titleMedium),
                      if (_info case final info?)
                        Text('${info.width} × ${info.height} · ${formatBytes(info.fileBytes)}'),
                    ],
                  ),
                ),
                OutlinedButton(
                  onPressed: () async {
                    final files = await pickImages(context, multiple: false);
                    if (files.isNotEmpty) _load(files.first);
                  },
                  child: Text(l.changeImage),
                ),
              ],
            ),
          ),
          Gap.m,
          _options(l),
          Gap.l,
          FilledButton.icon(
            onPressed: _busy || _widths.isEmpty ? null : _export,
            icon: _busy
                ? const SizedBox.square(dimension: 18, child: CircularProgressIndicator(strokeWidth: 2))
                : const Icon(Icons.rocket_launch_outlined),
            label: Text(_busy ? l.exporting : l.exportButton),
          ),
          if (_result case final result?) ...[Gap.l, _ResultCard(result: result, outputDir: _outputDir!)],
        ],
      ],
    );
  }

  Widget _options(AppLocalizations l) {
    final maxWidth = _info?.width;
    return SectionCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _name,
                  decoration: InputDecoration(labelText: l.nameLabel),
                ),
              ),
              Gap.m,
              Expanded(
                child: TextField(
                  controller: _baseUrl,
                  decoration: InputDecoration(labelText: l.baseUrlLabel),
                ),
              ),
            ],
          ),
          Gap.m,
          TextField(
            controller: _alt,
            decoration: InputDecoration(labelText: l.altLabel),
          ),
          Gap.l,
          Text(l.widthsLabel, style: Theme.of(context).textTheme.labelLarge),
          Gap.s,
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final w in _allWidths)
                FilterChip(
                  label: Text('$w'),
                  selected: _widths.contains(w),
                  // Sizes larger than the original are skipped by the engine.
                  onSelected: maxWidth != null && w > maxWidth && !_widths.contains(w)
                      ? null
                      : (on) => setState(() => _widths = on ? ({..._widths, w}) : (_widths.toSet()..remove(w))),
                ),
            ],
          ),
          Gap.l,
          QualitySlider(
            value: _quality,
            format: ImageFormat.jxl,
            min: 50,
            onChanged: (q) => setState(() => _quality = q),
          ),
          Gap.m,
          Text(l.fallbacksLabel, style: Theme.of(context).textTheme.labelLarge),
          Gap.s,
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final f in [ImageFormat.avif, ImageFormat.webp, ImageFormat.jpeg])
                FilterChip(
                  label: Text(f.label),
                  selected: _fallbacks.contains(f),
                  onSelected: (on) =>
                      setState(() => _fallbacks = on ? ({..._fallbacks, f}) : (_fallbacks.toSet()..remove(f))),
                ),
            ],
          ),
          Gap.m,
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(l.heroLabel),
            subtitle: Text(l.heroHint),
            value: _hero,
            onChanged: (v) => setState(() => _hero = v),
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(l.autoFocusLabel),
            subtitle: Text(l.autoFocusHint),
            value: _autoFocus,
            onChanged: (v) => setState(() => _autoFocus = v),
          ),
        ],
      ),
    );
  }
}

class _ResultCard extends StatelessWidget {
  const _ResultCard({required this.result, required this.outputDir});
  final WebExportResult result;
  final String outputDir;

  static const _tabs = [
    ('HTML', 'snippets/picture.html'),
    ('Next.js / React', 'snippets/JxlPicture.tsx'),
    ('WordPress', 'snippets/wordpress.php'),
    ('nginx', 'snippets/nginx.conf'),
    ('Apache', 'snippets/apache.htaccess'),
  ];

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final totals = result.totals;
    final jxl = totals.firstWhere((t) => t.$1 == ImageFormat.jxl, orElse: () => totals.first);
    final tabs = _tabs.where((t) => result.snippets.containsKey(t.$2)).toList();
    return SectionCard(
      title: l.webResultTitle(result.fileCount),
      trailing: TextButton.icon(
        onPressed: () => revealOrShare(outputDir),
        icon: Icon(isDesktop ? Icons.folder_open_outlined : Icons.ios_share),
        label: Text(isDesktop ? l.showInFolder : l.share),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              for (final (format, bytes) in totals)
                _Stat(
                  label: format.label,
                  value: formatBytes(bytes),
                  detail: format == ImageFormat.jxl || bytes == 0
                      ? null
                      : () {
                          final pct = (jxl.$2 / bytes - 1) * 100;
                          return pct <= 0
                              ? l.smallerThan(formatPercentAbs(pct), format.label)
                              : l.largerThan(formatPercentAbs(pct), format.label);
                        }(),
                  highlight: format == ImageFormat.jxl,
                ),
            ],
          ),
          Gap.l,
          Text(l.webHowTo, style: Theme.of(context).textTheme.bodyMedium),
          Gap.m,
          DefaultTabController(
            length: tabs.length,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                TabBar(
                  isScrollable: true,
                  tabAlignment: TabAlignment.start,
                  tabs: [for (final t in tabs) Tab(text: t.$1)],
                ),
                SizedBox(
                  height: 320,
                  child: TabBarView(children: [for (final t in tabs) _CodeView(code: result.snippets[t.$2]!)]),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _Stat extends StatelessWidget {
  const _Stat({required this.label, required this.value, this.detail, this.highlight = false});
  final String label;
  final String value;
  final String? detail;
  final bool highlight;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;
    return Container(
      constraints: const BoxConstraints(minWidth: 150),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: highlight ? scheme.primaryContainer : scheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(14),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: text.labelMedium),
          Text(value, style: text.titleLarge?.copyWith(fontWeight: FontWeight.w700)),
          if (detail != null) Text(detail!, style: text.bodySmall),
        ],
      ),
    );
  }
}

/// Monospace, selectable code with a copy button.
class _CodeView extends StatelessWidget {
  const _CodeView({required this.code});
  final String code;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final scheme = Theme.of(context).colorScheme;
    return Container(
      margin: const EdgeInsets.only(top: 12),
      decoration: BoxDecoration(color: scheme.surfaceContainerHighest, borderRadius: BorderRadius.circular(12)),
      child: Stack(
        children: [
          Positioned.fill(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(16),
              child: SelectableText(
                code,
                style: const TextStyle(fontFamily: 'monospace', fontSize: 12.5, height: 1.45),
              ),
            ),
          ),
          Positioned(
            right: 8,
            top: 8,
            child: FilledButton.tonalIcon(
              onPressed: () async {
                await Clipboard.setData(ClipboardData(text: code));
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(l.copied)));
                }
              },
              icon: const Icon(Icons.copy, size: 18),
              label: Text(l.copy),
            ),
          ),
        ],
      ),
    );
  }
}
