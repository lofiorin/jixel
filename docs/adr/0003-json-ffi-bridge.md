# ADR 0003: One JSON-over-C-ABI function between Dart and Rust

- Status: accepted
- Date: 2026-10-17

## Context

The Flutter app must call the Rust engine on five platforms. The common choice
is `flutter_rust_bridge` (FRB), which generates Dart and Rust glue code from
Rust signatures.

## Decision

The bridge (`app/rust`) exposes **one** function, `jixel_call(method,
args_json, buffers)`, plus `jixel_free`. Arguments and results are JSON; large
binary data (pixels, encoded files) travels in side buffers. The Dart side
(`engine/native.dart`) calls it with `dart:ffi` on a background isolate, and
`engine/engine.dart` offers a typed API on top.

It is compiled with **cargokit** (vendored), the same build integration FRB
uses, but without code generation.

## Why

- **Readable.** New contributors see the whole contract in two files instead
  of generated code.
- **Stable ABI.** Adding an operation never changes the C interface.
- **Simple builds.** No codegen step to keep in sync in CI on five platforms.
- **Cost is negligible.** JSON encoding takes microseconds; operations take
  milliseconds to seconds, and big data doesn't go through JSON.

## Consequences

- Argument names are checked at runtime, not compile time. Mitigated by:
  Dart models mirroring the Rust serde structs, bridge tests, and clear error
  messages (`missing argument "input"`).
- No streaming progress callbacks yet; long batches report progress per file.
