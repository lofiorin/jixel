// End-to-end tests on a real device or desktop: the native engine is loaded
// and every main operation runs on a generated image.
//
//   flutter test integration_test -d macos      (or windows, linux, an emulator)
//
// Set JIXEL_SHOTS=1 (--dart-define) to also save PNG renders of each screen
// to the temporary folder, for a quick visual check.
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:jixel/app.dart';
import 'package:jixel/engine/engine.dart';
import 'package:jixel/state/app_state.dart';
import 'package:jixel/state/settings.dart';
import 'package:path/path.dart' as p;
import 'package:shared_preferences/shared_preferences.dart';

const _shots = bool.fromEnvironment('JIXEL_SHOTS');

/// A 640×480 test picture with gradients, shapes and text-like detail.
Future<String> _samplePng(Directory dir) async {
  final recorder = ui.PictureRecorder();
  final canvas = Canvas(recorder);
  const size = Size(640, 480);
  canvas.drawRect(
    Offset.zero & size,
    Paint()..shader = ui.Gradient.linear(Offset.zero, const Offset(640, 480), [Colors.indigo, Colors.orange]),
  );
  canvas.drawCircle(const Offset(320, 240), 120, Paint()..color = Colors.white);
  for (var i = 0; i < 40; i++) {
    canvas.drawLine(Offset(i * 16.0, 0), Offset(640 - i * 16.0, 480), Paint()..color = Colors.black26);
  }
  final image = await recorder.endRecording().toImage(640, 480);
  final png = await image.toByteData(format: ui.ImageByteFormat.png);
  final path = p.join(dir.path, 'sample.png');
  await File(path).writeAsBytes(png!.buffer.asUint8List());
  return path;
}

Future<void> _shot(WidgetTester tester, Directory dir, String name) async {
  if (!_shots) return;
  final boundary = tester.renderObject<RenderRepaintBoundary>(find.byKey(const ValueKey('root')));
  final image = await boundary.toImage(pixelRatio: 1);
  final png = await image.toByteData(format: ui.ImageByteFormat.png);
  final path = p.join(dir.path, 'shot-$name.png');
  await File(path).writeAsBytes(png!.buffer.asUint8List());
  debugPrint('screenshot: $path');
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  late Directory dir;
  late String sample;

  setUpAll(() async {
    dir = await Directory.systemTemp.createTemp('jixel-it-');
    sample = await _samplePng(dir);
    debugPrint('work dir: ${dir.path}');
  });

  test('engine info', () async {
    final info = await Engine.info();
    expect(info.libjxl, startsWith('0.12'));
    expect(info.backgroundRemoval, isTrue);
  });

  test('PNG → lossless JPEG XL → PNG keeps every pixel', () async {
    final jxl = p.join(dir.path, 'a.jxl');
    final r = await Engine.convert(sample, jxl, const ConvertSettings(quality: 100));
    expect(r.fidelity, Fidelity.pixelExact);
    expect(r.outputBytes, lessThan(r.inputBytes));
    final back = await Engine.convert(jxl, p.join(dir.path, 'a.png'), const ConvertSettings(target: ImageFormat.png));
    expect(back.width, 640);
    final info = await Engine.inspect(jxl);
    expect(info.format, ImageFormat.jxl);
    expect(info.jxl!['uses_original_profile'], isTrue);
  });

  test('trial, battle, subject, background, crop, tiles, progressive', () async {
    final trial = await Engine.trial(sample, const ConvertSettings(quality: 85), maxSide: 320);
    expect(trial.score, greaterThan(70));
    expect(trial.preview.width, 320);

    final (_, _, entries) = await Engine.battle(sample, targetScore: 80);
    expect(entries.map((e) => e.format), containsAll([ImageFormat.jxl, ImageFormat.webp, ImageFormat.jpeg]));

    final subject = await Engine.detectSubject(sample);
    expect(subject.bbox.width, greaterThan(0));
    final boost = await Engine.subjectBoost(sample, p.join(dir.path, 'boost.jxl'));
    expect(boost.singleBytes, greaterThan(0));

    final (bytes, cut) = await Engine.removeBackground(sample, p.join(dir.path, 'cut.png'), maxSide: 256);
    expect(bytes, greaterThan(0));
    expect(cut.width, 256);

    final (_, fidelity) = await Engine.crop(sample, p.join(dir.path, 'crop.jxl'), const PixelRect(10, 10, 200, 100));
    expect(fidelity, Fidelity.pixelExact);
    expect(await Engine.tiles(sample, p.join(dir.path, 'tiles'), cols: 2, rows: 2), 4);

    final jxl = await Engine.encodeJxl(sample, const ConvertSettings(quality: 85, progressive: true));
    final steps = await Engine.progressive(jxl, [0.5, 1.0], maxSide: 320);
    expect(steps.last.preview, isNotNull);
  });

  test('web export writes the files and snippets', () async {
    final out = p.join(dir.path, 'web');
    final r = await Engine.webExport(sample, out, {
      'name': 'sample',
      'widths': [320, 640],
      'fallbacks': ['webp', 'jpeg'],
    });
    expect(r.fileCount, 6);
    expect(r.snippets.keys, contains('snippets/picture.html'));
    expect(File(p.join(out, 'manifest.json')).existsSync(), isTrue);
  });

  testWidgets('screens render with the real engine', (tester) async {
    SharedPreferences.setMockInitialValues({'locale': 'en', 'outputDir': dir.path});
    final state = AppState(SettingsController.withPrefs(await SharedPreferences.getInstance(), dir.path));
    state.engineInfo = await Engine.info();
    await tester.binding.setSurfaceSize(const Size(1280, 860));
    await tester.pumpWidget(
      RepaintBoundary(
        key: const ValueKey('root'),
        child: JixelApp(state: state),
      ),
    );
    await tester.pumpAndSettle();
    await _shot(tester, dir, 'home');

    Future<void> open(Section section, String name, {int? tool}) async {
      state.open(section, [sample], tool: tool);
      // Engine work happens on other isolates: let it finish, then settle.
      for (var i = 0; i < 40; i++) {
        await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 150)));
        await tester.pump();
      }
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await _shot(tester, dir, name);
    }

    await open(Section.convert, 'convert');
    await open(Section.web, 'web');
    await open(Section.studio, 'studio-compare', tool: 0);
    await open(Section.inspect, 'inspect');
    state.go(Section.settings);
    await tester.pumpAndSettle();
    await _shot(tester, dir, 'settings');
  });
}
