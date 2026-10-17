# Building from source

## Requirements

| Tool | Version | Needed for |
|---|---|---|
| git | any | submodules |
| Rust (via **rustup**) | 1.80+ | everything |
| CMake | 3.16+ | libjxl |
| A C/C++ compiler | Clang 14+, GCC 11+, MSVC 2022 | libjxl |
| Flutter | 3.44+ | the app |
| Xcode | 16+ | iOS, macOS |
| Android Studio + NDK | NDK 27+ | Android |
| Python 3 | any | regenerating translations |

`rustup` matters for the app: cargokit uses it to add mobile targets
(`aarch64-apple-ios`, `aarch64-linux-android`, …) automatically.
On macOS with Homebrew: `brew install rustup cmake` then `rustup default stable`;
Homebrew's `rustup` is keg-only, so put it on your PATH:
`export PATH="$(brew --prefix rustup)/bin:$PATH"`.

Linux desktop builds also need: `sudo apt install clang cmake ninja-build pkg-config libgtk-3-dev`.

## First time

```bash
git clone https://github.com/lofiorin/jixel.git
cd jixel
scripts/bootstrap.sh
```

This fetches libjxl and only the three of its submodules we need (brotli,
highway, skcms), and prepares the Flutter app.

## Core and CLI

```bash
cd core
cargo test --release           # first build compiles libjxl: a few minutes
cargo build --release -p jixel-cli
./target/release/jixel --help
```

Always use `--release` for anything involving real images; debug builds of
image code are very slow (dependencies are optimised even in debug, but our
own code is not).

Build without background removal (smaller binary, no `tract`):
`cargo build --release -p jixel-cli --no-default-features`.

## The app

```bash
cd app
flutter run -d macos        # or: windows, linux, an iOS simulator, an Android device
flutter build macos         # release builds: macos, windows, linux, ios, apk, appbundle
```

The first build compiles the Rust bridge and libjxl for the target (a few
minutes). Later builds are incremental.

### Platform notes

- **macOS / iOS**: the bridge is a static library linked into the
  `jixel_native` pod. Minimum macOS 10.15, iOS 13. The macOS app is sandboxed
  with *user-selected files* and *Downloads* read/write entitlements.
- **Android**: cargokit's Gradle plugin builds `libjixel_bridge.so` for each
  ABI. Set `ANDROID_NDK_HOME` if Gradle can't find the NDK. *Status: not yet
  verified with AGP 9; see Known issues.*
- **Windows**: build from a "Developer PowerShell for VS 2022" so CMake finds
  MSVC.
- **Linux**: GTK 3 desktop app.

### Regenerating translations

```bash
cd app
python3 tool/strings.py && flutter gen-l10n && dart format --line-length 120 lib/l10n
```

## Checks before a pull request

```bash
(cd core && cargo fmt --check && cargo clippy --release --all-targets -- -D warnings && cargo test --release)
(cd app/rust && cargo fmt --check && cargo clippy --release -- -D warnings)
(cd app && dart format --line-length 120 --set-exit-if-changed lib test && flutter analyze && flutter test)
```

CI runs the same commands (`.github/workflows/ci.yml`).

## Disk space

A full build needs about 3–4 GB per target directory (libjxl, tract and the
codecs are big). `core/target` and the app's build folder are separate;
`cargo clean` in `core/` and `flutter clean` in `app/` free them.

## Known issues

- **AVIF decoding** isn't available (AVIF can be written, not read).
- **Android with AGP 9** has not been verified yet: cargokit's Gradle plugin
  uses `libraryVariants`, which AGP 9 may have removed. If the Android build
  fails there, pin AGP 8.x in `app/android/settings.gradle.kts`.
