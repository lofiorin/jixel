/// Studio: one image, six tools that show off what JPEG XL can do.
library;

import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;

import '../../engine/engine.dart';
import '../../l10n/app_localizations.dart';
import '../../state/app_state.dart';
import '../format.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/file_input.dart';
import 'studio/background_tool.dart';
import 'studio/battle_tool.dart';
import 'studio/compare_tool.dart';
import 'studio/crop_tool.dart';
import 'studio/progressive_tool.dart';
import 'studio/subject_tool.dart';

/// The Studio tabs, in display order.
enum StudioTool { compare, subject, background, crop, progressive, battle }

/// What every tool receives: the file, its details and a screen preview.
class StudioImage {
  StudioImage(this.path, this.info, this.preview);
  final String path;
  final Inspection info;
  final RgbaPreview preview;

  String get stem => p.basenameWithoutExtension(path);
}

class StudioScreen extends StatefulWidget {
  const StudioScreen({super.key});

  @override
  State<StudioScreen> createState() => _StudioScreenState();
}

class _StudioScreenState extends State<StudioScreen> with SingleTickerProviderStateMixin {
  late final TabController _tabs;
  StudioImage? _image;
  bool _loading = false;

  @override
  void initState() {
    super.initState();
    _tabs = TabController(length: StudioTool.values.length, vsync: this);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final state = AppScope.of(context);
    final files = state.take(Section.studio);
    if (state.studioTool case final tool?) {
      _tabs.index = tool;
      state.studioTool = null;
    }
    if (files.isNotEmpty) _load(files.first);
  }

  @override
  void dispose() {
    _tabs.dispose();
    super.dispose();
  }

  Future<void> _load(String path) async {
    setState(() => _loading = true);
    try {
      final (info, preview) = (await Engine.inspect(path), await Engine.preview(path, maxSide: 1600));
      if (mounted) setState(() => _image = StudioImage(path, info, preview));
    } on EngineException catch (e) {
      if (mounted) showError(context, e.message);
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  Future<void> _change() async {
    final files = await pickImages(context, multiple: false);
    if (files.isNotEmpty) _load(files.first);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final image = _image;
    final labels = {
      StudioTool.compare: (Icons.compare_outlined, l.toolCompare),
      StudioTool.subject: (Icons.center_focus_strong_outlined, l.toolSubject),
      StudioTool.background: (Icons.content_cut, l.toolBackground),
      StudioTool.crop: (Icons.grid_view, l.toolCrop),
      StudioTool.progressive: (Icons.downloading, l.toolProgressive),
      StudioTool.battle: (Icons.emoji_events_outlined, l.toolBattle),
    };
    final header = Padding(
      padding: const EdgeInsets.fromLTRB(20, 24, 20, 0),
      child: Row(
        children: [
          Expanded(child: Text(l.navStudio, style: Theme.of(context).textTheme.headlineSmall)),
          if (image != null)
            OutlinedButton.icon(onPressed: _change, icon: const Icon(Icons.swap_horiz), label: Text(l.changeImage)),
        ],
      ),
    );

    if (image == null) {
      return SafeArea(
        child: ListView(
          padding: const EdgeInsets.only(bottom: 40),
          children: [
            header,
            Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                children: [
                  Text(l.studioEmpty, style: Theme.of(context).textTheme.bodyLarge),
                  Gap.l,
                  if (_loading)
                    const Padding(padding: EdgeInsets.all(40), child: CircularProgressIndicator())
                  else
                    ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 900),
                      child: DropZone(multiple: false, onFiles: (f) => _load(f.first)),
                    ),
                ],
              ),
            ),
          ],
        ),
      );
    }

    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          header,
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 4, 20, 0),
            child: Text(
              '${p.basename(image.path)} · ${image.info.width} × ${image.info.height} · ${formatBytes(image.info.fileBytes)}',
              style: Theme.of(
                context,
              ).textTheme.bodyMedium?.copyWith(color: Theme.of(context).colorScheme.onSurfaceVariant),
            ),
          ),
          TabBar(
            controller: _tabs,
            isScrollable: true,
            tabAlignment: TabAlignment.start,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            tabs: [for (final t in StudioTool.values) Tab(icon: Icon(labels[t]!.$1, size: 20), text: labels[t]!.$2)],
          ),
          Expanded(
            child: _loading
                ? const Center(child: CircularProgressIndicator())
                : TabBarView(
                    controller: _tabs,
                    // Each tool is keyed by the file, so it resets on a new image.
                    children: [
                      CompareTool(key: ValueKey(('c', image.path)), image: image),
                      SubjectTool(key: ValueKey(('s', image.path)), image: image),
                      BackgroundTool(key: ValueKey(('b', image.path)), image: image),
                      CropTool(key: ValueKey(('t', image.path)), image: image),
                      ProgressiveTool(key: ValueKey(('p', image.path)), image: image),
                      BattleTool(key: ValueKey(('f', image.path)), image: image),
                    ],
                  ),
          ),
        ],
      ),
    );
  }
}

/// Common layout for a tool tab: scrollable, centred, width-limited.
class ToolBody extends StatelessWidget {
  const ToolBody({super.key, this.intro, required this.children});
  final String? intro;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => ListView(
    padding: const EdgeInsets.fromLTRB(20, 20, 20, 40),
    children: [
      Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 1000),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              if (intro != null) ...[Text(intro!, style: Theme.of(context).textTheme.bodyLarge), Gap.l],
              ...children,
            ],
          ),
        ),
      ),
    ],
  );
}

/// A rounded frame around a preview.
class PreviewFrame extends StatelessWidget {
  const PreviewFrame({super.key, required this.child, this.maxHeight = 560});
  final Widget child;
  final double maxHeight;

  @override
  Widget build(BuildContext context) => Center(
    child: ConstrainedBox(
      constraints: BoxConstraints(maxHeight: maxHeight),
      child: ClipRRect(borderRadius: BorderRadius.circular(16), child: child),
    ),
  );
}

/// A primary action that shows a spinner while [busy].
class BusyButton extends StatelessWidget {
  const BusyButton({super.key, required this.busy, required this.onPressed, required this.icon, required this.label});
  final bool busy;
  final VoidCallback? onPressed;
  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) => FilledButton.icon(
    onPressed: busy ? null : onPressed,
    icon: busy ? const SizedBox.square(dimension: 18, child: CircularProgressIndicator(strokeWidth: 2)) : Icon(icon),
    label: Text(busy ? AppLocalizations.of(context).working : label),
  );
}

/// A saved file with a Show/Share button.
class SavedFile extends StatelessWidget {
  const SavedFile({super.key, required this.path, this.detail});
  final String path;
  final String? detail;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return Card(
      color: Theme.of(context).colorScheme.secondaryContainer,
      child: ListTile(
        leading: Icon(Icons.check_circle, color: successColor(context)),
        title: Text(l.savedTo(p.basename(path))),
        subtitle: detail == null ? null : Text(detail!),
        trailing: TextButton.icon(
          onPressed: () => revealOrShare(path),
          icon: Icon(isDesktop ? Icons.folder_open_outlined : Icons.ios_share),
          label: Text(isDesktop ? l.showInFolder : l.share),
        ),
      ),
    );
  }
}
