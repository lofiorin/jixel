# Compilare dai sorgenti

## Requisiti

| Strumento | Versione | Serve per |
|---|---|---|
| git | qualsiasi | submodule |
| Rust (tramite **rustup**) | 1.80+ | tutto |
| CMake | 3.16+ | libjxl |
| Un compilatore C/C++ | Clang 14+, GCC 11+, MSVC 2022 | libjxl |
| Flutter | 3.44+ | l'app |
| Xcode | 16+ | iOS, macOS |
| Android Studio + NDK | NDK 27+ | Android |
| Python 3 | qualsiasi | rigenerare le traduzioni |

`rustup` è importante per l'app: cargokit lo usa per aggiungere da solo i
target mobile (`aarch64-apple-ios`, `aarch64-linux-android`, …).
Su macOS con Homebrew: `brew install rustup cmake` e poi `rustup default stable`;
il `rustup` di Homebrew è keg-only, quindi aggiungilo al PATH:
`export PATH="$(brew --prefix rustup)/bin:$PATH"`.

Per le build desktop su Linux servono anche:
`sudo apt install clang cmake ninja-build pkg-config libgtk-3-dev`.

## La prima volta

```bash
git clone https://github.com/lofiorin/jixel.git
cd jixel
scripts/bootstrap.sh
```

Lo script scarica libjxl e solo tre dei suoi submodule (brotli, highway,
skcms), e prepara l'app Flutter.

## Motore e CLI

```bash
cd core
cargo test --release           # la prima build compila libjxl: qualche minuto
cargo build --release -p jixel-cli
./target/release/jixel --help
```

Usa sempre `--release` con immagini vere: le build debug del codice per
immagini sono molto lente.

Senza rimozione dello sfondo (binario più piccolo, senza `tract`):
`cargo build --release -p jixel-cli --no-default-features`.

## L'app

```bash
cd app
flutter run -d macos        # oppure: windows, linux, un simulatore iOS, un dispositivo Android
flutter build macos         # build di release: macos, windows, linux, ios, apk, appbundle
```

La prima build compila il ponte Rust e libjxl per la piattaforma (qualche
minuto). Le successive sono incrementali.

### Note per piattaforma

- **macOS / iOS**: il ponte è una libreria statica collegata al pod
  `jixel_native`. Minimo macOS 10.15, iOS 13. L'app macOS è in sandbox con i
  permessi di lettura/scrittura per *file scelti dall'utente* e *Download*.
- **Android**: il plugin Gradle di cargokit compila `libjixel_bridge.so` per
  ogni ABI. Imposta `ANDROID_NDK_HOME` se Gradle non trova l'NDK. *Stato: non
  ancora verificato con AGP 9; vedi Problemi noti.*
- **Windows**: compila da un "Developer PowerShell for VS 2022" così CMake
  trova MSVC.
- **Linux**: app desktop GTK 3.

### Rigenerare le traduzioni

```bash
cd app
python3 tool/strings.py && flutter gen-l10n && dart format --line-length 120 lib/l10n
```

## Controlli prima di una pull request

```bash
(cd core && cargo fmt --check && cargo clippy --release --all-targets -- -D warnings && cargo test --release)
(cd app/rust && cargo fmt --check && cargo clippy --release -- -D warnings)
(cd app && dart format --line-length 120 --set-exit-if-changed lib test && flutter analyze && flutter test)
```

La CI esegue gli stessi comandi (`.github/workflows/ci.yml`).

## Spazio su disco

Una build completa richiede circa 3–4 GB per cartella di target (libjxl, tract
e i codec sono grandi). `core/target` e la cartella di build dell'app sono
separate; `cargo clean` in `core/` e `flutter clean` in `app/` le liberano.

## Problemi noti

- **Lettura AVIF** non disponibile (AVIF si scrive, non si legge).
- **Android con AGP 9** non ancora verificato: il plugin Gradle di cargokit usa
  `libraryVariants`, che AGP 9 potrebbe aver rimosso. Se la build Android
  fallisce, fissa AGP 8.x in `app/android/settings.gradle.kts`.
