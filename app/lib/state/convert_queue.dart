/// The list of files on the Convert screen and their progress.
library;

import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;

import '../engine/engine.dart';

enum ItemStatus { waiting, working, done, failed }

class QueueItem {
  QueueItem(this.input) : inputBytes = File(input).lengthSync();

  final String input;
  final int inputBytes;
  ItemStatus status = ItemStatus.waiting;
  String? output;
  ConvertResult? result;
  String? error;

  String get name => p.basename(input);
}

class ConvertQueue extends ChangeNotifier {
  final List<QueueItem> items = [];
  bool running = false;

  void add(Iterable<String> paths) {
    final known = items.map((i) => i.input).toSet();
    items.addAll(paths.where((path) => !known.contains(path)).map(QueueItem.new));
    notifyListeners();
  }

  void remove(QueueItem item) {
    items.remove(item);
    notifyListeners();
  }

  void clear() {
    if (running) return;
    items.clear();
    notifyListeners();
  }

  int get pending => items.where((i) => i.status != ItemStatus.done).length;

  /// Bytes before/after over the finished items.
  (int, int) get totals {
    var before = 0, after = 0;
    for (final i in items.where((i) => i.result != null)) {
      before += i.result!.inputBytes;
      after += i.result!.outputBytes;
    }
    return (before, after);
  }

  /// Converts every unfinished item, a few at a time.
  Future<void> run(ConvertSettings settings, String outputDir) async {
    if (running) return;
    running = true;
    notifyListeners();
    await Directory(outputDir).create(recursive: true);
    final todo = items.where((i) => i.status != ItemStatus.done).toList();
    // The engine is multi-threaded already; two files at a time keeps the
    // UI responsive and the progress readable.
    const parallel = 2;
    for (var start = 0; start < todo.length; start += parallel) {
      await Future.wait(todo.skip(start).take(parallel).map((item) => _convert(item, settings, outputDir)));
    }
    running = false;
    notifyListeners();
  }

  Future<void> _convert(QueueItem item, ConvertSettings settings, String outputDir) async {
    item
      ..status = ItemStatus.working
      ..error = null;
    notifyListeners();
    final output = uniquePath(outputDir, p.basenameWithoutExtension(item.input), settings.target.extension);
    try {
      item.result = await Engine.convert(item.input, output, settings);
      item
        ..output = output
        ..status = ItemStatus.done;
    } on EngineException catch (e) {
      item
        ..error = e.message
        ..status = ItemStatus.failed;
    }
    notifyListeners();
  }
}

/// `dir/name.ext`, or `dir/name-2.ext` … if it already exists.
String uniquePath(String dir, String name, String ext) {
  var candidate = p.join(dir, '$name.$ext');
  for (var n = 2; File(candidate).existsSync(); n++) {
    candidate = p.join(dir, '$name-$n.$ext');
  }
  return candidate;
}

/// `dir/name`, or `dir/name-2` … if a file or folder with that name exists.
String uniqueDir(String dir, String name) {
  var candidate = p.join(dir, name);
  for (var n = 2; FileSystemEntity.typeSync(candidate) != FileSystemEntityType.notFound; n++) {
    candidate = p.join(dir, '$name-$n');
  }
  return candidate;
}
