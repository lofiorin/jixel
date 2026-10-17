# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org).

## [Unreleased]

## [0.1.0] — 2026-10-17

First version.

### Engine (`jixel-core`)
- JPEG XL encode/decode on libjxl 0.12 with hand-written bindings.
- Conversion between JPEG XL, JPEG, PNG, WebP, AVIF (write only), GIF
  (animated), TIFF, BMP, QOI, ICO, TGA, PNM, with a `Fidelity` for every result.
- Lossless JPEG recompression and bit-exact reconstruction.
- Progressive encoding and partial-download previews.
- Subject detection, subject-first group order, and *subject boost* (layered
  encoding, keeps the better result).
- On-device background removal (U²-Netp, 4.6 MB) with guided-filter edges.
- Pixel-exact crops, tiles and Deep Zoom pyramids.
- SSIMULACRA2 scoring and the same-quality format battle.
- Inspection with translatable tips.
- Responsive web export with manifest and snippets for HTML, React/Next.js,
  WordPress, nginx and Apache.

### CLI (`jixel`)
- `convert`, `web`, `info`, `compare`, `battle`, `crop`, `tiles`, `dzi`,
  `rmbg`, `subject`, `reconstruct`, `preview`; `--json` everywhere.

### App
- Flutter app for iOS, Android, macOS, Windows and Linux: Home, Convert,
  Optimize for web, Studio (Compare, Subject, Background, Crop & tiles,
  Loading, Format battle), Inspect, Settings.
- English and Italian.

### Known issues
- AVIF files cannot be opened (decoding not available yet).
- Android build not yet verified with Android Gradle Plugin 9.
- iOS is not built in CI yet. Windows and Linux are built in CI, and macOS is
  also tested end to end; Windows and Linux have not been tried by hand.
