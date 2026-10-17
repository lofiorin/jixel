/// Setting controls shared by Convert, Web and Settings.
library;

import 'package:flutter/material.dart';

import '../../engine/models.dart';
import '../../l10n/app_localizations.dart';
import '../theme.dart';

/// Quality 0–100 with a plain-language label ("Visually lossless"…).
class QualitySlider extends StatelessWidget {
  const QualitySlider({super.key, required this.value, required this.format, required this.onChanged, this.min = 30});

  final double value;
  final ImageFormat format;
  final ValueChanged<double> onChanged;
  final double min;

  static String describe(AppLocalizations l, double q, ImageFormat format) {
    if (q >= 100 && format.hasLossless) return l.qualityLossless;
    if (q >= 90) return l.qualityVisuallyLossless;
    if (q >= 80) return l.qualityHigh;
    if (q >= 65) return l.qualityBalanced;
    return l.qualitySmall;
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final text = Theme.of(context).textTheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Text(l.qualityLabel, style: text.labelLarge),
            const Spacer(),
            Text(
              '${describe(l, value, format)} · ${value.round()}',
              style: text.labelLarge?.copyWith(color: Theme.of(context).colorScheme.primary),
            ),
          ],
        ),
        Slider(
          value: value.clamp(min, 100),
          min: min,
          max: 100,
          divisions: (100 - min).round(),
          label: value.round().toString(),
          onChanged: onChanged,
        ),
      ],
    );
  }
}

class SpeedPicker extends StatelessWidget {
  const SpeedPicker({super.key, required this.value, required this.onChanged});
  final SpeedPreset value;
  final ValueChanged<SpeedPreset> onChanged;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return _Labeled(
      label: l.speedLabel,
      child: SegmentedButton<SpeedPreset>(
        showSelectedIcon: false,
        segments: [
          ButtonSegment(value: SpeedPreset.fast, label: Text(l.speedFast)),
          ButtonSegment(value: SpeedPreset.balanced, label: Text(l.speedBalanced)),
          ButtonSegment(value: SpeedPreset.max, label: Text(l.speedMax)),
        ],
        selected: {value},
        onSelectionChanged: (s) => onChanged(s.first),
      ),
    );
  }
}

class MetadataPicker extends StatelessWidget {
  const MetadataPicker({super.key, required this.value, required this.onChanged});
  final MetadataPolicy value;
  final ValueChanged<MetadataPolicy> onChanged;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return _Labeled(
      label: l.metadataLabel,
      child: SegmentedButton<MetadataPolicy>(
        showSelectedIcon: false,
        segments: [
          ButtonSegment(value: MetadataPolicy.keep, label: Text(l.metadataKeep)),
          ButtonSegment(value: MetadataPolicy.stripGps, label: Text(l.metadataStripGps)),
          ButtonSegment(value: MetadataPolicy.stripAll, label: Text(l.metadataStripAll)),
        ],
        selected: {value},
        onSelectionChanged: (s) => onChanged(s.first),
      ),
    );
  }
}

/// Format chips, JPEG XL first.
class FormatPicker extends StatelessWidget {
  const FormatPicker({super.key, required this.value, required this.onChanged, this.formats = ImageFormat.targets});
  final ImageFormat value;
  final ValueChanged<ImageFormat> onChanged;
  final List<ImageFormat> formats;

  @override
  Widget build(BuildContext context) => Wrap(
    spacing: 8,
    runSpacing: 8,
    children: [
      for (final f in formats) ChoiceChip(label: Text(f.label), selected: f == value, onSelected: (_) => onChanged(f)),
    ],
  );
}

class _Labeled extends StatelessWidget {
  const _Labeled({required this.label, required this.child});
  final String label;
  final Widget child;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(label, style: Theme.of(context).textTheme.labelLarge),
      Gap.s,
      SingleChildScrollView(scrollDirection: Axis.horizontal, child: child),
    ],
  );
}
