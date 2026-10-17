# JPEG XL, explained

This guide explains what JPEG XL is, what it does better than the formats you
already use, where it is *not* the best choice, and which Jixel tool uses each
feature. No prior knowledge of image compression is needed.

- [In one minute](#in-one-minute)
- [The features, one by one](#the-features-one-by-one)
- [Compared with WebP, AVIF and JPEG](#compared-with-webp-avif-and-jpeg)
- [Browser and software support](#browser-and-software-support)
- [Measuring quality: SSIMULACRA2](#measuring-quality-ssimulacra2)
- [Limits and honest caveats](#limits-and-honest-caveats)
- [Further reading](#further-reading)

## In one minute

JPEG XL (ISO/IEC 18181, file extension `.jxl`) is a royalty-free image format
designed by the JPEG committee to replace JPEG, PNG, GIF and WebP with one
format. Compared with what most websites use today:

- **Smaller files at the same visual quality.** Typically 20–40% smaller than
  WebP and 35–60% smaller than JPEG for photos at high quality.
- **Less visible damage at the same size.** JPEG XL degrades gracefully: it
  blurs slightly instead of producing blocks, banding or smeared colours.
- **Lossless when you need it,** 30–50% smaller than PNG.
- **Existing JPEGs shrink ~20% losslessly,** and can be turned back into the
  exact original file.
- **Features no other web format has:** subject-first loading, progressive
  decoding, layers, lossless cropping, up to 32-bit HDR, animation, large
  images (up to about a billion pixels per side).

## The features, one by one

### 1. Perceptual lossy compression (VarDCT + XYB)

Lossy JPEG XL stores colour in **XYB**, a colour space modelled on the human
eye, and uses variable-size transforms (from 2×2 up to 256×256 blocks) chosen
per area of the image: big smooth blocks for sky, small ones for edges and
texture. The encoder is tuned with the **Butteraugli** perceptual metric, so it
spends bits where people notice differences.

The encoder setting is a *distance* (0 = lossless, 1 = visually lossless, 3 =
"web quality"…). Jixel shows a familiar 0–100 quality slider and maps it to
distance the same way the reference `cjxl` tool does.

**In Jixel:** Convert, Optimize for web, Studio → Compare.

### 2. Lossless JPEG recompression

JPEG XL can take an existing JPEG and store the *same* DCT coefficients with a
much better entropy coder. Result: about **20% smaller**, **no change at all in
the pixels**, and the original JPEG file can be **reconstructed bit for bit**
(the `.jxl` keeps a small `jbrd` box with what is needed).

This is the one and only "free lunch" in image compression: you can convert a
whole photo archive with zero risk.

**In Jixel:** converting a JPEG to JPEG XL does this by default (results are
marked *Lossless · original restorable*). Inspect → *Restore original JPEG*
turns it back. CLI: `jixel convert`, `jixel reconstruct`.

### 3. Modular mode: true lossless

For screenshots, logos, line art, UI, medical and scientific images, JPEG XL's
*modular* mode is lossless and usually 30–50% smaller than optimised PNG, and
smaller than lossless WebP. It supports up to 16-bit integers and 32-bit float.

**In Jixel:** quality 100 (labelled *Lossless*). PNG → JPEG XL suggests it.

### 4. Progressive decoding

A JPEG XL file can be organised so that a usable preview appears after a small
part of it has downloaded, then sharpens in passes. Every lossy JPEG XL starts
with a 1:8 "DC" image; progressive files add more passes on top.

How early the first preview appears depends on the image: for a very detailed
4K photo the 1:8 image was ~19% of the file in our tests; for typical web
photos it is around 10–15%.

**In Jixel:** *Progressive loading* (Convert → Advanced; always on in Optimize
for web). Studio → Loading simulates a slow download.

### 5. Group ordering: the subject loads first ("main subject")

JPEG XL splits an image into 256×256 *groups* and lets the encoder choose their
order in the file. Instead of top-to-bottom, Jixel can write the groups
**starting from a point of interest and spiralling outwards**, so on a slow
connection the subject is sharp before the background. The file is a normal
JPEG XL; every decoder understands it.

**In Jixel:** Optimize for web → *Load the subject first* (on by default);
Studio → Subject; CLI `--auto-focus` or `--focus x,y`.

### 6. Layers: give the subject more detail ("subject boost")

A JPEG XL image can contain several **layers** (frames composited into one
still image). Jixel uses this to encode the background at a lower quality and
the subject as a second, higher-quality layer on top. Under the subject, the
background is flattened so it costs almost nothing.

Jixel always encodes both the standard file and the layered one, scores the
subject area with SSIMULACRA2, and **keeps whichever is better**. On our test
photo the layered file was 16% smaller *and* its subject scored 82.9 instead
of 76.5.

**In Jixel:** Studio → Subject → *Create optimised file*. CLI: `jixel subject`.

### 7. Lossless crop and tiles

Cropping or splitting an image normally means decoding and re-encoding it,
which loses quality each time. Jixel stores crops and tiles **pixel-exact**
(lossless JPEG XL is efficient enough to make this practical), so every pixel
equals the original. It can also build a **Deep Zoom (DZI) pyramid** for
viewers like OpenSeadragon — maps, floor plans, product close-ups, artwork.

**In Jixel:** Studio → Crop & tiles. CLI: `jixel crop`, `jixel tiles`,
`jixel dzi`.

### 8. HDR, wide gamut and high bit depth

JPEG XL handles 8 to 32 bits per channel, PQ and HLG HDR transfer functions,
and any ICC colour profile, all natively. Jixel preserves bit depth and
profiles through conversions and warns you (Inspect) when a file is HDR.

### 9. Alpha and animation

Full alpha channel (with lossy or lossless colour), and animation with full
colour instead of GIF's 256. Animated GIF → JPEG XL is usually much smaller.

### 10. Metadata and privacy

EXIF, XMP and JUMBF live in container boxes, optionally Brotli-compressed.
Jixel removes GPS location by default when exporting (choose *Keep all*,
*Remove location* or *Remove all*).

## Compared with WebP, AVIF and JPEG

| | JPEG XL | AVIF | WebP | JPEG |
|---|---|---|---|---|
| Photo quality per byte, high quality | **best** | very good | good | baseline |
| Photo quality per byte, very low quality | good | **best** | fair | poor |
| Lossless | **best** | weak | good | — |
| Lossless recompression of JPEG | **yes, ~20%** | no | no | — |
| Progressive decoding | **yes** | no | no | yes |
| Subject-first loading | **yes** | no | no | no |
| Max bit depth | **32 (float)** | 12 | 8 | 8 (12) |
| Max size | ~10⁹ px/side | 65 536 (tiled) | 16 383 | 65 535 |
| Encoding speed | fast | slow | fast | fastest |
| Browser support (2026) | Safari, Chrome, Firefox | all modern | all | all |

**WebP** caps you at 8 bits and halves the colour resolution (4:2:0), which is
where its blurry reds and banded skies come from. JPEG XL keeps full colour
and wins clearly at the quality levels people actually publish (score ≥ 70).

**AVIF** is excellent at *very* low bitrates and is the best *fallback* for
browsers without JPEG XL, which is why Jixel uses it as the first fallback. At
high quality JPEG XL is smaller, encodes much faster, decodes progressively and
supports lossless JPEG recompression.

## Browser and software support

As of October 2026:

- **Safari** 17+ (macOS Sonoma, iOS 17) — full support.
- **Chrome / Edge** 155+ — decoded with the memory-safe Rust `jxl-rs` decoder.
- **Firefox** 157+ — enabled by default.
- Older browsers and Firefox ESR need a fallback: Jixel always creates one with
  `<picture>`, so nobody sees a broken image.
- **Operating systems:** macOS 14+ and iOS 17+ natively; Windows 11 with the
  JPEG XL extension; most Linux image viewers via libjxl.
- **Software:** Photoshop, Lightroom, Affinity, GIMP, Krita, darktable,
  ImageMagick, libvips, ffmpeg.
- **WordPress** core still rejects `.jxl` uploads; Jixel's WordPress snippet
  fixes that (see [web delivery](web-delivery.md)).

## Measuring quality: SSIMULACRA2

Jixel scores results with **SSIMULACRA2**, the metric the JPEG XL authors
developed and validated against large human studies. Scale 0–100:

| Score | Meaning |
|---|---|
| 90+ | visually lossless: indistinguishable at normal viewing |
| 80–90 | very high: differences only under close inspection |
| 70–80 | high: good for the web |
| 50–70 | medium: visible artefacts |
| < 50 | low |

The format battle uses it to tune each format to the **same score** and then
compare sizes — the only fair way to compare formats.

## Limits and honest caveats

- **Not every browser yet.** Always ship a fallback (Jixel does).
- **Very low quality:** below a score of ~60, AVIF often wins. Don't push JPEG
  XL that low; it's not what it's designed for.
- **Already-tiny images** (icons, small thumbnails) gain less; the fallback
  overhead may not be worth it.
- **Re-encoding a JPEG lossily** is generation loss: prefer lossless
  recompression unless you also resize.
- **AVIF decoding:** Jixel can *write* AVIF but cannot *read* it yet, so the
  format battle reports AVIF sizes without a score.

## Further reading

- [jpeg.org/jpegxl](https://jpeg.org/jpegxl/) — official page
- [libjxl](https://github.com/libjxl/libjxl) — reference implementation
- Alakuijala et al., *JPEG XL next-generation image compression architecture and coding tools* (SPIE 2019)
- [jpegxl.info](https://jpegxl.info) — community resources and comparisons
- [SSIMULACRA2](https://github.com/cloudinary/ssimulacra2)
