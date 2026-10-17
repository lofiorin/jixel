# Third-party notices

Jixel is licensed under Apache-2.0 (see [LICENSE](LICENSE)). It includes or
links the following components, each under its own licence. The app also shows
the Flutter/Dart package licences under Settings → About → Open-source
licences.

## Bundled in every build

| Component | Use | Licence |
|---|---|---|
| [libjxl](https://github.com/libjxl/libjxl) 0.12 | JPEG XL encoder/decoder | BSD-3-Clause (+ patent grant, see `PATENTS` in libjxl) |
| [Brotli](https://github.com/google/brotli) | used by libjxl | MIT |
| [Highway](https://github.com/google/highway) | SIMD, used by libjxl | Apache-2.0 or BSD-3-Clause |
| [skcms](https://skia.googlesource.com/skcms) | colour management, used by libjxl | BSD-3-Clause |
| [U²-Netp](https://github.com/xuebinqin/U-2-Net) model | background removal | Apache-2.0 — Qin, X. et al., *U²-Net: Going deeper with nested U-structure for salient object detection*, Pattern Recognition 106 (2020) |
| [libwebp](https://chromium.googlesource.com/webm/libwebp) via `webp` crate | WebP encode | BSD-3-Clause |
| [ravif](https://github.com/kornelski/cavif-rs) / [rav1e](https://github.com/xiph/rav1e) | AVIF encode | BSD-3-Clause / BSD-2-Clause |
| [image](https://github.com/image-rs/image) | PNG, JPEG, GIF, TIFF, BMP, ICO, TGA, QOI, PNM, WebP decode | MIT or Apache-2.0 |
| [tract](https://github.com/sonos/tract) | ONNX inference | MIT or Apache-2.0 |
| [ssimulacra2](https://github.com/rust-av/ssimulacra2) | quality metric | BSD-2-Clause |
| [fast_image_resize](https://github.com/Cykooz/fast_image_resize) | resizing | MIT or Apache-2.0 |
| [rayon](https://github.com/rayon-rs/rayon), [serde](https://serde.rs), [thiserror](https://github.com/dtolnay/thiserror), [clap](https://github.com/clap-rs/clap) | infrastructure | MIT or Apache-2.0 |

The full dependency tree with licences can be listed with
`cargo tree` / [`cargo about`](https://github.com/EmbarkStudios/cargo-about)
in `core/` and `app/rust/`.

## App only

| Component | Licence |
|---|---|
| [Flutter](https://flutter.dev) and Dart SDK | BSD-3-Clause |
| [cargokit](https://github.com/irondash/cargokit) (vendored in `app/rust_builder/cargokit`) | MIT or Apache-2.0 |
| file_picker, desktop_drop, share_plus, shared_preferences, path_provider, intl, ffi, path | MIT / BSD-3-Clause (see each package) |
