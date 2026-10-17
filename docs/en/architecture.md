# Architecture

A 10-minute map of the code. If you are about to change something, this tells
you where it lives and why it is built that way.

## The big picture

```
┌──────────────────────────── Flutter app (app/lib) ───────────────────────────┐
│ ui/screens  ──►  state/  ──►  engine/engine.dart  (typed async API)          │
│                                     │  JSON + byte buffers, background isolate│
│                                     ▼                                         │
│                         engine/native.dart  (dart:ffi)                        │
└─────────────────────────────────────┬─────────────────────────────────────────┘
                                      │  C ABI: jixel_call / jixel_free
┌─────────────────────────────────────▼─────────────────────────────────────────┐
│ app/rust (jixel_bridge)  — decodes the JSON, calls jixel-core, encodes result │
└─────────────────────────────────────┬─────────────────────────────────────────┘
                                      │  plain Rust calls
┌────────────── core/ (Rust workspace) ▼───────────────────────────────────────┐
│ jixel-core: raster · format · codec · convert · ops · analyze · web          │
│        │                                                                      │
│        ▼                                                                      │
│ jixel-libjxl-sys: libjxl 0.12 built from source + hand-written FFI            │
│ jixel-cli: the `jixel` command, same engine                                   │
└───────────────────────────────────────────────────────────────────────────────┘
```

**Rule of thumb:** image logic lives in `jixel-core`, *only* there. The CLI and
the bridge translate arguments and print or return results; the Flutter app
only shows things. This keeps the engine testable without a UI and identical
everywhere.

## core/crates/jixel-libjxl-sys

- `build.rs` builds libjxl and its three bundled dependencies (brotli, highway,
  skcms) with CMake as static libraries, for the *target* platform (so
  cross-compiling to iOS/Android works). libjxl is always built in Release:
  debug libjxl is ~20× slower.
- `src/lib.rs` declares the subset of the libjxl C API we use. Hand-written
  instead of using `jpegxl-sys` because that crate is GPL-licensed and Jixel
  is Apache-2.0 ([ADR 0002](../adr/0002-own-libjxl-bindings.md)).

## core/crates/jixel-core

| Module | Responsibility |
|---|---|
| `raster` | `Raster`: width, height, channels, 8/16/32f samples, ICC, frames. Every module passes this around. |
| `format` | `Format` detection from bytes (magic numbers) and capabilities (alpha, lossless, animation). |
| `codec::jxl` | Safe encoder/decoder over libjxl: distance from quality, effort, progressive, focus point (group order), layers, JPEG recompression & reconstruction, partial-file decode (progressive preview), metadata boxes. |
| `codec::raster` | Every other format through the `image` crate (+ `ravif` for AVIF encode). |
| `metadata` | EXIF/XMP extraction, GPS stripping, orientation. |
| `convert` | `convert(bytes, options) -> ConvertOutput` — one call for any → any, with a `Fidelity` (`Reversible`, `PixelExact`, `Lossy`). |
| `ops::resize` | High-quality downscale (Lanczos3 via `fast_image_resize`). |
| `ops::crop` | Pixel-exact crops. |
| `ops::tiles` | Grid/size tiling and Deep Zoom pyramids. |
| `ops::subject` | Subject detection (from the background mask), focus point, *subject boost* (layered encode, keep the better). |
| `ops::background` | U²-Netp inference with `tract-onnx`, guided-filter edge refinement. Behind the `background-removal` feature (default on). |
| `analyze::inspect` | `Inspection` + `Tip` codes (translatable). |
| `analyze::compare` | SSIMULACRA2 scoring and the format battle (quality search per format). |
| `analyze::progressive` | What a JPEG XL looks like after N% of its bytes. |
| `web` | Responsive export, `Manifest`, and `snippets` (HTML, React, WordPress, nginx, Apache). |

Errors use one `Error` enum (`thiserror`), with messages written for users.

### Key design choices

- **Honest results.** Every conversion reports its `Fidelity`. The UI shows it
  as a badge; nothing is called "lossless" unless it is.
- **JPEG → JPEG XL defaults to lossless recompression.** Re-encoding is opt-in.
- **Fair comparisons.** The battle tunes each format to the *same SSIMULACRA2
  score* by binary search, instead of comparing "quality 80" settings that mean
  different things per format.
- **Subject boost keeps the winner.** Layered encoding is not always better,
  so both are encoded and scored; the better file is written.
- **Model embedded** (4.6 MB). No downloads at runtime, works offline.

## core/crates/jixel-cli

`clap` derive. One subcommand per feature. `--json` everywhere. Batches run in
parallel with `rayon`. No image logic beyond argument parsing.

## app/rust (the bridge)

A single C function:

```c
JixelResponse* jixel_call(const char* method, const char* args_json,
                          const JixelBuffer* buffers, size_t n_buffers);
void jixel_free(JixelResponse*);
```

`method` is e.g. `"convert"`, args are JSON, large inputs/outputs (pixels,
encoded files) travel as raw buffers next to the JSON. The response is
`{"ok": …}` or `{"error": "…"}`; panics are caught. Every method is
documented at the top of its `match` arm in `src/lib.rs`.

Why not flutter_rust_bridge? See [ADR 0003](../adr/0003-json-ffi-bridge.md):
one stable function is easier to read, debug and build on five platforms than
generated bindings, and the per-call JSON cost is negligible next to image
work.

The bridge is compiled by **cargokit** (vendored in `app/rust_builder/cargokit`)
through the `jixel_native` Flutter plugin: CocoaPods script phase on
iOS/macOS, Gradle on Android, CMake on Windows/Linux.

## app/lib (Flutter)

```
main.dart            start-up: load settings, run app, fetch engine info
app.dart             MaterialApp: themes, locales, AppScope
engine/
  native.dart        dart:ffi bindings, callNative() on a background isolate
  models.dart        typed views over the engine JSON (mirror Rust structs)
  engine.dart        Engine.convert(), Engine.webExport(), … — use this
state/
  settings.dart      persisted preferences (shared_preferences)
  app_state.dart     current section, files handed between sections
  convert_queue.dart the Convert list and its progress
ui/
  shell.dart         navigation rail (≥ 840 px) or bottom bar
  theme.dart         Material 3 theme from one seed colour, spacing
  format.dart        bytes and percent formatting
  widgets/           drop zone, previews, compare slider, rect selector, controls
  screens/           home, convert, web_export, studio (+ studio/*), inspect, settings
l10n/                app_en.arb, app_it.arb (generated from tool/strings.py)
```

- **No state-management package.** `ChangeNotifier` + one `InheritedNotifier`
  (`AppScope`) are enough and need no explanation.
- **All engine calls are async and off the UI thread** (`Isolate.run`).
- **Files, not bytes.** Operations take paths and write files; only previews
  (RGBA, at most 1600–2048 px) cross into Dart.

### Translations

Strings live in one table, `app/tool/strings.py` (key → English, Italian).
Run `python3 tool/strings.py` then `flutter gen-l10n` and `dart format --line-length 120 lib/l10n`. To add a language, add
a column and a locale (see [CONTRIBUTING](../../CONTRIBUTING.md)).

## Tests

| Where | What | Run |
|---|---|---|
| `core/**` | unit + round-trip tests (encode/decode, lossless guarantees, JPEG reconstruction bit-exactness, tiles reassembly, web export, snippets) | `cargo test --release` in `core/` |
| `app/rust` | bridge round trips | `cargo test --release` in `app/rust/` |
| `app/test` | formatting, settings JSON, widget tests (layout, navigation, i18n) | `flutter test` in `app/` |
| `app/integration_test` | the real engine on a device/desktop, screen renders | `flutter test integration_test -d macos` |

## Further reading

- [Design specification](../superpowers/specs/2026-10-17-jixel-design.md)
- [Implementation plan](../superpowers/plans/2026-10-17-jixel-v0.1.md)
- [ADRs](../adr)
