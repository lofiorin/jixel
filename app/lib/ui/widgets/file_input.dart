/// Getting files into the app: a drop zone (desktop) and the system picker.
library;

import 'dart:io';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;

import '../../engine/models.dart';
import '../../l10n/app_localizations.dart';
import '../theme.dart';

final bool isDesktop = Platform.isMacOS || Platform.isWindows || Platform.isLinux;

/// Keeps only files the engine can read; reports how many were skipped.
List<String> supportedOnly(BuildContext context, Iterable<String> paths) {
  final ok = <String>[];
  var skipped = 0;
  for (final path in paths) {
    final ext = p.extension(path).replaceFirst('.', '').toLowerCase();
    if (ImageFormat.readableExtensions.contains(ext) && File(path).existsSync()) {
      ok.add(path);
    } else {
      skipped++;
    }
  }
  if (skipped > 0 && context.mounted) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(AppLocalizations.of(context).unsupportedFiles(skipped))));
  }
  return ok;
}

/// Opens the system picker for images.
Future<List<String>> pickImages(BuildContext context, {bool multiple = true}) async {
  final files = multiple
      ? await FilePicker.pickFiles(type: FileType.custom, allowedExtensions: ImageFormat.readableExtensions)
      : [?await FilePicker.pickFile(type: FileType.custom, allowedExtensions: ImageFormat.readableExtensions)];
  if (!context.mounted) return const [];
  return supportedOnly(context, files.map((f) => f.path).whereType<String>());
}

/// A large, friendly target to drop files on or click to pick them.
class DropZone extends StatefulWidget {
  const DropZone({super.key, required this.onFiles, this.multiple = true, this.compact = false});

  final ValueChanged<List<String>> onFiles;
  final bool multiple;
  final bool compact;

  @override
  State<DropZone> createState() => _DropZoneState();
}

class _DropZoneState extends State<DropZone> {
  bool _hover = false;

  Future<void> _pick() async {
    final files = await pickImages(context, multiple: widget.multiple);
    if (files.isNotEmpty) widget.onFiles(files);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final scheme = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;
    final content = AnimatedContainer(
      duration: const Duration(milliseconds: 180),
      padding: EdgeInsets.symmetric(horizontal: 24, vertical: widget.compact ? 20 : 40),
      decoration: BoxDecoration(
        color: _hover ? scheme.primaryContainer : scheme.surfaceContainerLow,
        borderRadius: BorderRadius.circular(24),
        border: Border.all(color: _hover ? scheme.primary : scheme.outlineVariant, width: _hover ? 2 : 1),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.add_photo_alternate_outlined, size: widget.compact ? 32 : 48, color: scheme.primary),
          Gap.m,
          if (isDesktop) ...[
            Text(l.dropTitle, style: text.titleLarge, textAlign: TextAlign.center),
            Gap.s,
            Text(l.dropOr, style: text.bodyMedium?.copyWith(color: scheme.onSurfaceVariant)),
            Gap.s,
          ],
          FilledButton.tonalIcon(
            onPressed: _pick,
            icon: const Icon(Icons.folder_open_outlined),
            label: Text(widget.multiple ? l.chooseFiles : l.chooseImage),
          ),
          if (!widget.compact) ...[
            Gap.m,
            Text(
              l.supportedFormats,
              style: text.bodySmall?.copyWith(color: scheme.onSurfaceVariant),
              textAlign: TextAlign.center,
            ),
          ],
        ],
      ),
    );
    if (!isDesktop) return content;
    return DropTarget(
      onDragEntered: (_) => setState(() => _hover = true),
      onDragExited: (_) => setState(() => _hover = false),
      onDragDone: (details) {
        setState(() => _hover = false);
        var files = supportedOnly(context, details.files.map((f) => f.path));
        if (!widget.multiple && files.length > 1) files = files.take(1).toList();
        if (files.isNotEmpty) widget.onFiles(files);
      },
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(onTap: _pick, child: content),
      ),
    );
  }
}
