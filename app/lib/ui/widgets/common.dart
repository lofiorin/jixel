/// Small building blocks used across screens.
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:share_plus/share_plus.dart';

import '../../engine/models.dart';
import '../../l10n/app_localizations.dart';
import '../theme.dart';

/// A titled card section.
class SectionCard extends StatelessWidget {
  const SectionCard({super.key, this.title, this.trailing, required this.child, this.padding = 20});

  final String? title;
  final Widget? trailing;
  final Widget child;
  final double padding;

  @override
  Widget build(BuildContext context) => Card(
    child: Padding(
      padding: EdgeInsets.all(padding),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (title != null) ...[
            Row(
              children: [
                Expanded(child: Text(title!, style: Theme.of(context).textTheme.titleMedium)),
                ?trailing,
              ],
            ),
            Gap.m,
          ],
          child,
        ],
      ),
    ),
  );
}

/// Page scaffold: centred, width-limited content with a big title.
class PageBody extends StatelessWidget {
  const PageBody({super.key, required this.title, this.subtitle, this.actions = const [], required this.children});

  final String title;
  final String? subtitle;
  final List<Widget> actions;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    return SafeArea(
      child: ListView(
        padding: const EdgeInsets.fromLTRB(20, 24, 20, 40),
        children: [
          Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1000),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      Expanded(child: Text(title, style: text.headlineSmall)),
                      ...actions,
                    ],
                  ),
                  if (subtitle != null) ...[
                    Gap.s,
                    Text(
                      subtitle!,
                      style: text.bodyLarge?.copyWith(color: Theme.of(context).colorScheme.onSurfaceVariant),
                    ),
                  ],
                  Gap.l,
                  ...children,
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// A rounded label like "Lossless".
class Pill extends StatelessWidget {
  const Pill(this.text, {super.key, this.icon, this.color});

  final String text;
  final IconData? icon;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final fg = color ?? scheme.onSecondaryContainer;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: (color ?? scheme.secondaryContainer).withValues(alpha: color == null ? 1 : 0.14),
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (icon != null) ...[Icon(icon, size: 14, color: fg), const SizedBox(width: 4)],
          Text(
            text,
            style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: fg),
          ),
        ],
      ),
    );
  }
}

/// The honesty badge for a result.
class FidelityPill extends StatelessWidget {
  const FidelityPill(this.fidelity, {super.key});
  final Fidelity fidelity;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return switch (fidelity) {
      Fidelity.reversible => Pill(l.fidelityReversible, icon: Icons.verified_outlined, color: successColor(context)),
      Fidelity.pixelExact => Pill(l.fidelityPixelExact, icon: Icons.verified_outlined, color: successColor(context)),
      Fidelity.lossy => Pill(l.fidelityLossy, icon: Icons.tune),
    };
  }
}

/// Localised name of a quality score band.
String scoreText(AppLocalizations l, double score) => switch (scoreBand(score)) {
  ScoreBand.visuallyLossless => l.scoreVisuallyLossless,
  ScoreBand.veryHigh => l.scoreVeryHigh,
  ScoreBand.high => l.scoreHigh,
  ScoreBand.medium => l.scoreMedium,
  ScoreBand.low => l.scoreLow,
};

/// Reveals [path] in the file manager (desktop) or shares it (mobile).
Future<void> revealOrShare(String path) async {
  if (Platform.isMacOS) {
    await Process.run('open', ['-R', path]);
  } else if (Platform.isWindows) {
    await Process.run('explorer', ['/select,', path]);
  } else if (Platform.isLinux) {
    await Process.run('xdg-open', [FileSystemEntity.isDirectorySync(path) ? path : p.dirname(path)]);
  } else {
    await SharePlus.instance.share(ShareParams(files: [XFile(path)]));
  }
}

/// Shows an error in a snack bar.
void showError(BuildContext context, Object error) {
  if (!context.mounted) return;
  ScaffoldMessenger.of(
    context,
  ).showSnackBar(SnackBar(content: Text(AppLocalizations.of(context).errorGeneric(error.toString()))));
}
