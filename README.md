<p align="center">
  <b>Jixel</b> — JPEG XL for everyone<br>
  <i>Smaller images. Same quality. Every platform.</i>
</p>

<p align="center">
  <a href="README.it.md">🇮🇹 Leggi in italiano</a> ·
  <a href="docs/en/user-guide.md">User guide</a> ·
  <a href="docs/en/web-delivery.md">Web delivery</a> ·
  <a href="docs/en/cli.md">CLI</a> ·
  <a href="docs/en/architecture.md">Architecture</a>
</p>

---

Jixel is a free, open-source app for **iOS, Android, macOS, Windows and Linux**
that makes [JPEG XL](docs/en/jpeg-xl.md) easy to use — above all for
**publishing lighter, sharper images on websites**.

- **Convert anything ↔ JPEG XL.** JPEG, PNG, WebP, AVIF, GIF, TIFF, BMP, QOI,
  ICO, TGA, PNM in; any of them out. Batches too.
- **JPEGs shrink ~20% with zero loss**, and the original JPEG can be restored
  bit for bit.
- **Optimize for web** in one click: every screen size, fallbacks for older
  browsers, and the code for **HTML, Next.js/React, WordPress, nginx, Apache**.
- **Subject first.** Jixel finds the main subject, makes it load first and can
  give it more detail than the background — in one ordinary `.jxl` file.
- **Lossless crop & tiles**, **Deep Zoom** pyramids for zoomable viewers.
- **Background removal**, fully on-device, with a 4.6 MB model that runs on
  practically any phone or PC.
- **Format battle**: JPEG XL vs AVIF vs WebP vs JPEG tuned to the *same visual
  quality* (SSIMULACRA2), on *your* photos.
- **Inspect** any file: what's inside, privacy (GPS), and what to do about it.
- Honest by design: every result says whether it is **reversible**,
  **pixel-exact** or **lossy**.
- Your images never leave your device.

## Why JPEG XL?

On a 3840×2160 photo, tuned so every format reaches the *same* visual quality
(SSIMULACRA2 80, "very high"):

| Format | Size | vs JPEG |
|---|---:|---:|
| **JPEG XL** | **194 KB** | **−41%** |
| WebP | 280 KB | −15% |
| JPEG | 329 KB | — |

And for an existing JPEG, lossless recompression saved 21% with a
bit-identical round trip. Run `jixel battle your-photo.jpg` to see your own
numbers. The [JPEG XL guide](docs/en/jpeg-xl.md) explains where the gains come
from, and where JPEG XL is *not* the best choice.

Browser support (October 2026): Safari 17+, Chrome 155+ and Firefox 157+
decode JPEG XL. Jixel always generates a fallback for everything else —
see [Web delivery](docs/en/web-delivery.md).

## Quick start

### App

Download a build from Releases (coming with the first public release) or
[build it yourself](docs/en/building.md):

```bash
scripts/bootstrap.sh
cd app && flutter run
```

### CLI

```bash
scripts/bootstrap.sh
cargo install --path core/crates/jixel-cli

jixel convert photos/*.jpg -o out/          # lossless JPEG → JPEG XL
jixel web hero.png -o public/images --hero  # responsive set + snippets
jixel battle photo.jpg                      # compare formats
```

Full reference: [docs/en/cli.md](docs/en/cli.md).

## Project layout

```
core/                   Rust workspace — the engine
  crates/jixel-libjxl-sys   libjxl built from source + hand-written FFI
  crates/jixel-core         every image operation, as plain functions
  crates/jixel-cli          the `jixel` command
app/                    Flutter app (all five platforms)
  rust/                     C ABI bridge between Dart and jixel-core
  rust_builder/             Flutter plugin that builds the bridge (cargokit)
  lib/                      UI: engine/, state/, ui/screens, ui/widgets
models/                 U²-Netp background-removal model (4.6 MB)
docs/                   documentation (en, it), ADRs, design spec
scripts/                bootstrap and helper scripts
```

Start with [architecture.md](docs/en/architecture.md): it is a 10-minute read
that maps every folder to a responsibility.

## Documentation

| | English | Italiano |
|---|---|---|
| What JPEG XL is and why it matters | [jpeg-xl.md](docs/en/jpeg-xl.md) | [jpeg-xl.md](docs/it/jpeg-xl.md) |
| Using the app | [user-guide.md](docs/en/user-guide.md) | [guida-utente.md](docs/it/guida-utente.md) |
| Shipping JPEG XL on websites | [web-delivery.md](docs/en/web-delivery.md) | [web.md](docs/it/web.md) |
| Command line | [cli.md](docs/en/cli.md) | [cli.md](docs/it/cli.md) |
| How the code is organised | [architecture.md](docs/en/architecture.md) | [architettura.md](docs/it/architettura.md) |
| Building from source | [building.md](docs/en/building.md) | [compilazione.md](docs/it/compilazione.md) |
| Questions | [faq.md](docs/en/faq.md) | [faq.md](docs/it/faq.md) |
| Design decisions | [docs/adr](docs/adr) | |

## Contributing

Contributions are welcome — code, translations, docs, bug reports with sample
images. Read [CONTRIBUTING.md](CONTRIBUTING.md) first. Please follow the
[Code of Conduct](CODE_OF_CONDUCT.md) and report vulnerabilities as described in
[SECURITY.md](SECURITY.md).

## License

Jixel is licensed under the [Apache License 2.0](LICENSE). It bundles
libjxl (BSD-3-Clause), the U²-Netp model (Apache-2.0) and other open-source
components listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
