# Jixel — Design Specification

- **Status:** Approved (author was given full autonomy by the project owner on 2026-10-17)
- **Date:** 2026-10-17
- **Owner:** @lofiorin

## 1. Intent

Jixel is a free, open-source, cross-platform app (iOS, Android, macOS, Windows,
Linux) that makes **JPEG XL** (`.jxl`) practical for everyday use, and in
particular for **shipping lighter, higher-quality images on websites**.

The owner builds many websites on many stacks (WordPress, Next.js, plain HTML
and everything in between). Today they use WebP. They want smaller files *and*
less visible quality loss, plus the JPEG XL features that no other mainstream
format has: subject-aware encoding, lossless JPEG recompression, lossless
cropping/splitting, HDR, progressive decoding, layers and animation.

The code will be published on GitHub. Every decision is judged by one question:
**will another developer understand this quickly and be able to contribute?**

### Why now

As of October 2026 JPEG XL decodes by default in Safari (17+), Chrome (155+,
via the Rust `jxl-rs` decoder) and Firefox (157/158+). Older browsers and
Firefox ESR still lack it, so a fallback format is still required. WordPress
core still rejects `.jxl` uploads.

## 2. Goals

1. **Convert anything to JXL** and **JXL to anything** (PNG, JPEG, WebP, AVIF,
   GIF, TIFF, BMP, QOI, ICO, TGA, PNM), single files or batches.
2. **Web delivery first:** one action produces responsive `srcset` widths, JXL
   plus a fallback (AVIF / WebP / JPEG), and copy-paste snippets for HTML,
   Next.js (React) and WordPress, plus Nginx/Apache content-negotiation configs.
3. **Expose every useful JXL advantage** through a tool a non-expert can use:
   - lossless JPEG → JXL recompression (~20% smaller, bit-exact reversible);
   - true lossless mode for PNG-style content;
   - perceptual quality control via Butteraugli *distance*, with a plain
     "quality" slider mapped onto it;
   - **subject focus**: (a) the subject's 256×256 groups are stored first so it
     sharpens first while loading, (b) optional *subject boost* that encodes the
     subject as a higher-quality layer composited over a cheaper background;
   - progressive decoding, with a "slow network" preview that shows how the
     file appears at 5%, 15%, 30%… of its bytes;
   - **lossless crop** and **tile splitting** (with a Deep Zoom pyramid for
     zoomable images);
   - high bit depth / HDR and ICC colour profile preservation;
   - animation (animated GIF → animated JXL);
   - alpha channel, metadata keep/strip (EXIF, XMP, GPS privacy strip).
4. **Background removal** running fully on-device with a model small enough
   for any PC or phone (≈4.7 MB U²-Netp, Apache-2.0). The same mask drives
   automatic subject detection.
5. **Format battle:** encode the same image as JXL, AVIF, WebP and JPEG and
   show size and SSIMULACRA2 score side by side, so the user sees the gain on
   *their* photos.
6. A **CLI** (`jixel`) sharing the same engine, for build pipelines on any
   website stack.
7. Interface that is extremely intuitive and modern, on par with market-leading
   apps; English and Italian UI.
8. Best-in-class documentation in English with an Italian translation.

## 3. Non-goals (for v0.1)

- A full photo editor (filters, retouching, brushes).
- Cloud processing or accounts. Everything is local.
- HEIC and AVIF *decoding* (needs native libheif/dav1d on every platform;
  listed on the roadmap). AVIF *encoding* is supported.
- A WordPress plugin or Next.js loader package (snippets are provided instead;
  packages are on the roadmap).

## 4. Anti-goals (failure even if it "works")

- Any photo leaving the device.
- A UI that requires knowing what "distance", "effort" or "VarDCT" mean to get
  a good result. Expert controls exist but are tucked away.
- Pretending an operation is lossless when it is not. The app always says
  whether an operation is bit-exact, pixel-exact or re-encoded.
- Code only the author can follow: no clever macros, no undocumented unsafe.

## 5. Architecture

```
┌────────────────────────── Flutter app (Dart) ──────────────────────────┐
│ Screens: Home · Convert · Web Export · Studio · Inspect · Settings     │
│ State: Riverpod-free plain ChangeNotifier controllers                  │
│ l10n: ARB files (en, it)                                               │
└───────────────▲────────────────────────────────────────────────────────┘
                │ flutter_rust_bridge v2 (generated, async, off UI thread)
┌───────────────┴──────────── app/rust (jixel_bridge) ───────────────────┐
│ Thin, typed façade: DTOs + calls into jixel-core                       │
└───────────────▲────────────────────────────────────────────────────────┘
                │ plain Rust calls
┌───────────────┴──────────── core/crates/jixel-core ────────────────────┐
│ codec/   jxl (libjxl via jpegxl-sys), raster formats (image, webp, avif)│
│ ops/     resize, crop, tiles, subject, background                      │
│ analyze/ inspect, compare (SSIMULACRA2), progressive preview           │
│ web/     responsive export, snippets, server configs                   │
└───────────────▲────────────────────────────────────────────────────────┘
                │
┌───────────────┴──────────── core/crates/jixel-cli ─────────────────────┐
│ `jixel` command-line tool (clap)                                       │
└────────────────────────────────────────────────────────────────────────┘
```

### Decisions (author's calls)

| Area | Choice | Why |
|---|---|---|
| UI | Flutter 3.44, Material 3 | One codebase for all five platforms; already installed. |
| Engine | Rust 2021 workspace | Memory safety around C codecs; reusable by CLI and app. |
| JXL codec | libjxl 0.12 through `jpegxl-sys` (vendored build) | Reference implementation; the only one exposing JPEG reconstruction, group order, layers, progressive flags. A small safe wrapper is written in `codec/jxl`. |
| Other formats | `image` 0.25, `webp` (libwebp), `ravif` (AVIF encode) | Well maintained, portable. |
| Background model | U²-Netp ONNX (~4.7 MB) run with `tract-onnx` | Pure Rust inference, no native runtime to ship, works on mobile. |
| Quality metric | `ssimulacra2` crate | The metric libjxl itself is tuned against. |
| Bridge | flutter_rust_bridge 2.x + cargokit | Builds Rust automatically per platform from `flutter build`. |
| License | Apache-2.0 | Patent grant, compatible with libjxl (BSD-3) and U²-Net (Apache-2.0). Owner may change before publishing. |

## 6. Feature behaviour

### 6.1 Quality model
User sees **Quality 0–100** (default 90 for photos) and a **Lossless** toggle.
Mapped with libjxl's own `JxlEncoderDistanceFromQuality`. *Effort* is exposed
as "Speed: Fast / Balanced / Max compression" (3 / 7 / 9).
JPEG inputs default to **lossless recompression** (bit-exact) unless the user
asks for resizing or a lower quality.

### 6.2 Subject focus
- Subject source: automatic (background-removal mask → bounding box and
  centroid) or manual rectangle.
- *Load-first*: `JXL_ENC_FRAME_SETTING_GROUP_ORDER = 1` with
  `GROUP_ORDER_CENTER_X/Y` set to the subject centroid.
- *Subject boost*: background frame encoded at the chosen quality minus a
  delta, then a second, cropped frame (layer) at higher quality positioned
  over the subject, blended with `JXL_BLEND_REPLACE`. Both frames have
  duration 0 so decoders composite them into one still image. The encoder
  measures the result and falls back to single-layer if boost is not smaller
  at equal or better subject SSIMULACRA2 (honesty rule).

### 6.3 Lossless crop & tiles
- **Pixel-exact crop:** decode → crop → lossless JXL. The crop's pixels equal
  the decoded source pixels exactly, for any source. Larger than a re-encode.
- **Re-encode crop:** decode → crop → encode at chosen quality.
- **Bit-exact JPEG** sources (raw `.jpg` or JPEG-reconstructible `.jxl`): crop
  is pixel-exact as above; the UI labels it accordingly.
- **Tiles:** split into an N×M grid or fixed tile size; optional Deep Zoom
  (`.dzi`) pyramid for OpenSeadragon-style viewers.

### 6.4 Progressive preview
Truncate the encoded bytes at several fractions, decode with progressive detail
and `JxlDecoderFlushImage`, return the partial frames. Shows load-first in action.

### 6.5 Web export
Inputs: images, widths (default 480, 768, 1080, 1440, 1920, 2560, capped at the
source width), fallback format (AVIF default, WebP, JPEG), quality, subject
focus, metadata strip (default on). Output folder contains files named
`<name>-<width>.<ext>`, `manifest.json`, and `snippets/` with `picture.html`,
`JxlPicture.tsx` (Next.js), `wordpress.php` (helper + MIME allowance),
`nginx.conf`, `apache.htaccess`.

### 6.6 Background removal
Resize to 320×320, normalise with ImageNet mean/std, run U²-Netp, take output
d0, min-max normalise, upscale to source size bilinearly, optional threshold /
feather. Output RGBA PNG/JXL/WebP or replace background with a solid colour.

### 6.7 Inspect
Format, dimensions, bit depth, colour channels, alpha, animation frames, ICC
presence, JPEG-reconstruction data present, container boxes (Exif, xml,
jumb, brob), file size, bits per pixel.

## 7. UX
- Home: a big drop zone / "Choose photos" button and four task cards
  (Convert, Optimize for web, Remove background, Studio).
- Every task is a single page with a live result card: before/after slider,
  size saving in large type ("−63%"), and one primary button.
- Advanced settings in a collapsible "Advanced" panel.
- Adaptive layout: navigation rail on desktop/tablet, bottom bar on phones.
- Light/dark, system accent; locale from system, switchable in Settings.

## 8. Testing
- Rust unit tests per module, round-trip tests (encode → decode equals pixels
  for lossless; JPEG reconstruction bit-exact), web-export integration test.
- Flutter widget tests for the home and convert screens.
- CI: `cargo fmt/clippy/test`, `flutter analyze/test`, desktop builds on
  macOS, Windows, Linux; Android APK build.

## 9. Left to the builder
Exact visual styling, file naming of internal modules, icon artwork.
