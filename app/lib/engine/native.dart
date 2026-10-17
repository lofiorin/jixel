/// Low-level access to the Rust engine through `dart:ffi`.
///
/// The Rust side exposes a single C function (see `app/rust/src/lib.rs`):
///
/// ```c
/// JixelResponse* jixel_call(const char* method, const char* args_json,
///                           const JixelBuffer* inputs, size_t n_inputs);
/// void jixel_free(JixelResponse*);
/// ```
///
/// [callNative] wraps it: it runs on a background isolate so the UI never
/// stutters, converts arguments to JSON, and copies any binary outputs into
/// Dart memory before freeing the native response.
library;

import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

final class _JixelBuffer extends Struct {
  external Pointer<Uint8> ptr;
  @Size()
  external int len;
}

final class _JixelResponse extends Struct {
  external Pointer<Utf8> json;
  external Pointer<_JixelBuffer> buffers;
  @Size()
  external int nBuffers;
}

typedef _CallC = Pointer<_JixelResponse> Function(Pointer<Utf8>, Pointer<Utf8>, Pointer<_JixelBuffer>, Size);
typedef _CallDart = Pointer<_JixelResponse> Function(Pointer<Utf8>, Pointer<Utf8>, Pointer<_JixelBuffer>, int);
typedef _FreeC = Void Function(Pointer<_JixelResponse>);
typedef _FreeDart = void Function(Pointer<_JixelResponse>);

/// Opens the library built by the `jixel_native` plugin.
DynamicLibrary _open() {
  if (Platform.isIOS || Platform.isMacOS) {
    // Statically linked into the plugin framework, which the app loads.
    try {
      return DynamicLibrary.open('jixel_native.framework/jixel_native');
    } on ArgumentError {
      return DynamicLibrary.process();
    }
  }
  if (Platform.isWindows) return DynamicLibrary.open('jixel_bridge.dll');
  return DynamicLibrary.open('libjixel_bridge.so');
}

// One instance per isolate, created on first use.
final DynamicLibrary _lib = _open();
final _CallDart _call = _lib.lookupFunction<_CallC, _CallDart>('jixel_call');
final _FreeDart _free = _lib.lookupFunction<_FreeC, _FreeDart>('jixel_free');

/// An error reported by the engine, with a message meant for people.
class EngineException implements Exception {
  EngineException(this.message);
  final String message;

  @override
  String toString() => message;
}

/// The result of a native call: decoded JSON plus binary outputs.
class NativeResult {
  NativeResult(this.value, this.buffers);

  /// The `ok` payload (a map, list or scalar).
  final Object? value;

  /// Binary outputs, referenced by index from [value].
  final List<Uint8List> buffers;

  Map<String, Object?> get map => value! as Map<String, Object?>;
}

/// Synchronous call; use [callNative] from UI code.
NativeResult callNativeSync(String method, Map<String, Object?> args, [List<Uint8List> inputs = const []]) {
  return using((arena) {
    final m = method.toNativeUtf8(allocator: arena);
    final a = jsonEncode(args).toNativeUtf8(allocator: arena);
    final bufs = arena<_JixelBuffer>(inputs.isEmpty ? 1 : inputs.length);
    for (var i = 0; i < inputs.length; i++) {
      final data = arena<Uint8>(inputs[i].length);
      data.asTypedList(inputs[i].length).setAll(0, inputs[i]);
      bufs[i]
        ..ptr = data
        ..len = inputs[i].length;
    }
    final response = _call(m, a, bufs, inputs.length);
    try {
      final r = response.ref;
      final decoded = jsonDecode(r.json.toDartString()) as Map<String, Object?>;
      final buffers = [
        for (var i = 0; i < r.nBuffers; i++) Uint8List.fromList(r.buffers[i].ptr.asTypedList(r.buffers[i].len)),
      ];
      if (decoded['error'] case final String message) {
        throw EngineException(message);
      }
      return NativeResult(decoded['ok'], buffers);
    } finally {
      _free(response);
    }
  });
}

/// Runs [method] on a background isolate.
Future<NativeResult> callNative(String method, Map<String, Object?> args, [List<Uint8List> inputs = const []]) {
  return Isolate.run(() => callNativeSync(method, args, inputs), debugName: 'jixel:$method');
}
