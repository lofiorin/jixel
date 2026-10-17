/// Preferences: appearance, language, output folder and default quality.
library;

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';

import '../../engine/models.dart';
import '../../l10n/app_localizations.dart';
import '../../state/app_state.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/controls.dart';
import '../widgets/file_input.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final state = AppScope.of(context);
    final settings = state.settings;
    return ListenableBuilder(
      listenable: settings,
      builder: (context, _) => PageBody(
        title: l.navSettings,
        children: [
          SectionCard(
            title: l.appearance,
            child: SegmentedButton<ThemeMode>(
              segments: [
                ButtonSegment(
                  value: ThemeMode.system,
                  label: Text(l.themeSystem),
                  icon: const Icon(Icons.brightness_auto),
                ),
                ButtonSegment(value: ThemeMode.light, label: Text(l.themeLight), icon: const Icon(Icons.light_mode)),
                ButtonSegment(value: ThemeMode.dark, label: Text(l.themeDark), icon: const Icon(Icons.dark_mode)),
              ],
              selected: {settings.themeMode},
              onSelectionChanged: (s) => settings.themeMode = s.first,
            ),
          ),
          Gap.m,
          SectionCard(
            title: l.language,
            child: SegmentedButton<String>(
              segments: [
                ButtonSegment(value: '', label: Text(l.languageSystem)),
                const ButtonSegment(value: 'en', label: Text('English')),
                const ButtonSegment(value: 'it', label: Text('Italiano')),
              ],
              selected: {settings.locale?.languageCode ?? ''},
              onSelectionChanged: (s) => settings.locale = s.first.isEmpty ? null : Locale(s.first),
            ),
          ),
          if (isDesktop) ...[
            Gap.m,
            SectionCard(
              title: l.outputFolder,
              child: Row(
                children: [
                  Expanded(child: SelectableText(settings.outputDir)),
                  Gap.s,
                  OutlinedButton(
                    onPressed: () async {
                      final dir = await FilePicker.getDirectoryPath();
                      if (dir != null) settings.outputDir = dir;
                    },
                    child: Text(l.change),
                  ),
                ],
              ),
            ),
          ],
          Gap.m,
          SectionCard(
            title: l.defaults,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                QualitySlider(value: settings.quality, format: ImageFormat.jxl, onChanged: (v) => settings.quality = v),
                Gap.m,
                SpeedPicker(value: settings.speed, onChanged: (v) => settings.speed = v),
                Gap.m,
                MetadataPicker(value: settings.metadata, onChanged: (v) => settings.metadata = v),
              ],
            ),
          ),
          Gap.m,
          SectionCard(
            title: l.about,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(l.aboutBody),
                if (state.engineInfo case final info?) ...[
                  Gap.s,
                  Text(l.engineVersion(info.version, info.libjxl), style: Theme.of(context).textTheme.bodySmall),
                ],
                Gap.m,
                OutlinedButton(
                  onPressed: () => showLicensePage(context: context, applicationName: l.appTitle),
                  child: Text(l.licenses),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
