/// Home: drop images, then pick what to do with them.
library;

import 'package:flutter/material.dart';

import '../../l10n/app_localizations.dart';
import '../../state/app_state.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/file_input.dart';
import 'studio.dart' show StudioTool;

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  /// Files dropped here, waiting for the user to choose a task.
  List<String> _files = const [];

  void _open(Section section, {StudioTool? tool}) {
    final state = AppScope.read(context);
    if (_files.isEmpty) {
      state.open(section, const [], tool: tool?.index);
    } else {
      // Single-image tools get the first file.
      final files = section == Section.convert ? _files : [_files.first];
      state.open(section, files, tool: tool?.index);
    }
    setState(() => _files = const []);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final text = Theme.of(context).textTheme;
    final scheme = Theme.of(context).colorScheme;
    final tasks = [
      (Icons.swap_horiz, l.taskConvertTitle, l.taskConvertBody, () => _open(Section.convert)),
      (Icons.language, l.taskWebTitle, l.taskWebBody, () => _open(Section.web)),
      (
        Icons.content_cut,
        l.taskBackgroundTitle,
        l.taskBackgroundBody,
        () => _open(Section.studio, tool: StudioTool.background),
      ),
      (Icons.auto_awesome, l.taskStudioTitle, l.taskStudioBody, () => _open(Section.studio)),
    ];

    return SafeArea(
      child: ListView(
        padding: const EdgeInsets.fromLTRB(20, 32, 20, 40),
        children: [
          Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 900),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(l.homeHeadline, style: text.displaySmall, textAlign: TextAlign.center),
                  Gap.s,
                  Text(
                    l.homeSubtitle,
                    style: text.bodyLarge?.copyWith(color: scheme.onSurfaceVariant),
                    textAlign: TextAlign.center,
                  ),
                  Gap.xl,
                  if (_files.isEmpty)
                    DropZone(onFiles: (f) => setState(() => _files = f))
                  else
                    SectionCard(
                      title: l.webResultTitle(_files.length),
                      trailing: IconButton(
                        icon: const Icon(Icons.close),
                        onPressed: () => setState(() => _files = const []),
                      ),
                      child: Wrap(
                        spacing: 8,
                        runSpacing: 8,
                        children: [
                          for (final f in _files.take(8)) Pill(f.split(RegExp(r'[\\/]')).last),
                          if (_files.length > 8) Pill('+${_files.length - 8}'),
                        ],
                      ),
                    ),
                  Gap.xl,
                  Text(l.whatToDo, style: text.titleLarge),
                  Gap.m,
                  LayoutBuilder(
                    builder: (context, box) {
                      final columns = box.maxWidth >= 640 ? 2 : 1;
                      final width = (box.maxWidth - (columns - 1) * 16) / columns;
                      return Wrap(
                        spacing: 16,
                        runSpacing: 16,
                        children: [
                          for (final (icon, title, body, onTap) in tasks)
                            SizedBox(
                              width: width,
                              child: _TaskCard(icon: icon, title: title, body: body, onTap: onTap),
                            ),
                        ],
                      );
                    },
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _TaskCard extends StatelessWidget {
  const _TaskCard({required this.icon, required this.title, required this.body, required this.onTap});

  final IconData icon;
  final String title;
  final String body;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;
    return Card(
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(color: scheme.primaryContainer, borderRadius: BorderRadius.circular(14)),
                child: Icon(icon, color: scheme.onPrimaryContainer),
              ),
              Gap.m,
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(title, style: text.titleMedium),
                    Gap.xs,
                    Text(body, style: text.bodyMedium?.copyWith(color: scheme.onSurfaceVariant)),
                  ],
                ),
              ),
              Icon(Icons.chevron_right, color: scheme.onSurfaceVariant),
            ],
          ),
        ),
      ),
    );
  }
}
