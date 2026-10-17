# Jixel app

The Flutter app for iOS, Android, macOS, Windows and Linux.

```bash
../scripts/bootstrap.sh       # once
flutter run -d macos          # or windows, linux, an iOS simulator, an Android device
flutter test                  # unit + widget tests (no native engine needed)
flutter test integration_test -d macos   # real engine, on a desktop or device
```

| Folder | What |
|---|---|
| `lib/engine/` | typed Dart API over the Rust engine (`Engine.convert(...)` …) |
| `lib/state/` | settings, app state, the Convert queue |
| `lib/ui/` | shell, theme, widgets, screens |
| `lib/l10n/` | translations, generated from `tool/strings.py` |
| `rust/` | the C ABI bridge (`jixel_call`) to `core/crates/jixel-core` |
| `rust_builder/` | the `jixel_native` plugin that builds the bridge with cargokit |

Read [../docs/en/architecture.md](../docs/en/architecture.md) for how the
pieces fit, and [../docs/en/building.md](../docs/en/building.md) for
platform requirements.
