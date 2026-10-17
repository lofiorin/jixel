/// The app frame: navigation rail on wide windows, bottom bar on phones.
library;

import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../state/app_state.dart';
import 'screens/convert.dart';
import 'screens/home.dart';
import 'screens/inspect.dart';
import 'screens/settings.dart';
import 'screens/studio.dart';
import 'screens/web_export.dart';

/// Windows at least this wide get the side rail.
const wideBreakpoint = 840.0;

class AppShell extends StatelessWidget {
  const AppShell({super.key});

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final l = AppLocalizations.of(context);
    final destinations = [
      (Section.home, Icons.home_outlined, Icons.home, l.navHome),
      (Section.convert, Icons.swap_horiz_outlined, Icons.swap_horiz, l.navConvert),
      (Section.web, Icons.language_outlined, Icons.language, l.navWeb),
      (Section.studio, Icons.auto_awesome_outlined, Icons.auto_awesome, l.navStudio),
      (Section.inspect, Icons.info_outline, Icons.info, l.navInspect),
      (Section.settings, Icons.settings_outlined, Icons.settings, l.navSettings),
    ];
    // Screens stay alive in an IndexedStack, so switching keeps their state.
    final body = IndexedStack(
      index: state.section.index,
      children: const [
        HomeScreen(),
        ConvertScreen(),
        WebExportScreen(),
        StudioScreen(),
        InspectScreen(),
        SettingsScreen(),
      ],
    );

    return LayoutBuilder(
      builder: (context, box) {
        if (box.maxWidth >= wideBreakpoint) {
          return Scaffold(
            body: Row(
              children: [
                NavigationRail(
                  selectedIndex: state.section.index,
                  onDestinationSelected: (i) => state.go(Section.values[i]),
                  labelType: NavigationRailLabelType.all,
                  groupAlignment: -0.85,
                  leading: Padding(padding: const EdgeInsets.symmetric(vertical: 12), child: _Logo()),
                  destinations: [
                    for (final (_, icon, selected, label) in destinations)
                      NavigationRailDestination(icon: Icon(icon), selectedIcon: Icon(selected), label: Text(label)),
                  ],
                ),
                const VerticalDivider(width: 1),
                Expanded(child: body),
              ],
            ),
          );
        }
        // Phones: five tabs at the bottom, settings in the top bar.
        final compact = destinations.where((d) => d.$1 != Section.settings).toList();
        final index = state.section == Section.settings ? null : state.section.index;
        return Scaffold(
          appBar: AppBar(
            title: Text(l.appTitle),
            actions: [
              IconButton(
                tooltip: l.navSettings,
                isSelected: state.section == Section.settings,
                icon: const Icon(Icons.settings_outlined),
                selectedIcon: const Icon(Icons.settings),
                onPressed: () => state.go(Section.settings),
              ),
            ],
          ),
          body: body,
          bottomNavigationBar: NavigationBar(
            selectedIndex: index ?? 0,
            indicatorColor: index == null ? Colors.transparent : null,
            onDestinationSelected: (i) => state.go(compact[i].$1),
            destinations: [
              for (final (_, icon, selected, label) in compact)
                NavigationDestination(icon: Icon(icon), selectedIcon: Icon(selected), label: label),
            ],
          ),
        );
      },
    );
  }
}

class _Logo extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Container(
      width: 44,
      height: 44,
      decoration: BoxDecoration(
        gradient: LinearGradient(colors: [scheme.primary, scheme.tertiary]),
        borderRadius: BorderRadius.circular(14),
      ),
      alignment: Alignment.center,
      child: Text(
        'J',
        style: TextStyle(color: scheme.onPrimary, fontSize: 22, fontWeight: FontWeight.w800),
      ),
    );
  }
}
