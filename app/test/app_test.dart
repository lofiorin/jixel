// UI smoke tests. They do not load the native engine: they check that the
// screens build, navigate and translate.
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jixel/app.dart';
import 'package:jixel/state/app_state.dart';
import 'package:jixel/state/settings.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<AppState> _state([Map<String, Object> prefs = const {}]) async {
  SharedPreferences.setMockInitialValues(prefs);
  return AppState(SettingsController.withPrefs(await SharedPreferences.getInstance(), '/tmp/jixel'));
}

void main() {
  testWidgets('home shows the main tasks', (tester) async {
    tester.view.physicalSize = const Size(1280, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(JixelApp(state: await _state({'locale': 'en'})));
    await tester.pumpAndSettle();
    expect(find.text('Smaller images. Same quality.'), findsOneWidget);
    expect(find.text('Optimize for web'), findsWidgets);
    expect(find.byType(NavigationRail), findsOneWidget);
  });

  testWidgets('phones get a bottom navigation bar', (tester) async {
    tester.view.physicalSize = const Size(390 * 3, 844 * 3);
    tester.view.devicePixelRatio = 3;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(JixelApp(state: await _state({'locale': 'en'})));
    await tester.pumpAndSettle();
    expect(find.byType(NavigationBar), findsOneWidget);
    expect(find.byType(NavigationRail), findsNothing);
  });

  testWidgets('Italian is used when chosen', (tester) async {
    tester.view.physicalSize = const Size(1280, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(JixelApp(state: await _state({'locale': 'it'})));
    await tester.pumpAndSettle();
    expect(find.text('Converti'), findsWidgets);
  });

  testWidgets('settings change the theme', (tester) async {
    tester.view.physicalSize = const Size(1280, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final state = await _state({'locale': 'en'});
    await tester.pumpWidget(JixelApp(state: state));
    state.go(Section.settings);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Dark'));
    await tester.pumpAndSettle();
    expect(state.settings.themeMode, ThemeMode.dark);
  });
}
