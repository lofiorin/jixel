# User guide

Jixel has six sections. On a computer they are in the sidebar; on a phone the
first five are at the bottom and **Settings** is the gear at the top right.

Everything runs on your device. Nothing is uploaded.

- [Home](#home)
- [Convert](#convert)
- [Optimize for web](#optimize-for-web)
- [Studio](#studio)
- [Inspect](#inspect)
- [Settings](#settings)
- [Understanding the results](#understanding-the-results)

## Home

Drop images on the big area (computer) or tap **Choose files**, then pick what
to do: **Convert**, **Optimize for web**, **Remove background** or **Studio**.
You can also go straight to a section and open files there.

Supported input: JPEG XL, JPEG, PNG, WebP, GIF (including animated), TIFF,
BMP, QOI, ICO, TGA, PNM/PPM/PGM/PAM.

## Convert

1. Add one or more images (drop more at any time).
2. Choose the **format**. JPEG XL is first; any format can be the target, so
   this also converts JPEG XL *back* to PNG, JPEG, WebP…
3. Set the **quality**. The label tells you what to expect:

   | Quality | Label | Typical use |
   |---|---|---|
   | 100 | Lossless | screenshots, logos, archiving |
   | 90–99 | Visually lossless | photography, print, master copies |
   | 80–89 | High | websites (recommended 85) |
   | 65–79 | Balanced | thumbnails, galleries |
   | < 65 | Smallest | previews |

4. Press **Convert**. Each file shows its new size, the saving, and an honesty
   badge (see [Understanding the results](#understanding-the-results)).

**JPEG files** are *repacked* losslessly by default: ~20% smaller, identical
pixels, and the original can be restored. The quality slider does not apply to
them unless you enable *Re-encode JPEGs instead* in **Advanced**.

**Before / after:** the compare icon next to a waiting file encodes it in
memory with the current settings and shows it next to the original with a
draggable divider (pinch or scroll to zoom), plus a quality score.

**Advanced**

- *Encoding speed*: Fast, Balanced (default), Smallest file. Slower = smaller.
- *Progressive loading*: a preview appears while the image downloads.
- *Re-encode JPEGs instead*: smaller, but lossy.
- *Maximum width*: shrink large images (aspect ratio kept).
- *Metadata*: Keep all, Remove location (default), Remove all.

Results go to the output folder (Settings). On a computer, **Show** opens it;
on a phone, **Share** opens the system share sheet.

## Optimize for web

Produces everything you need to publish one image on a website:

- JPEG XL in several widths (480–2560 px by default; widths larger than the
  original are skipped),
- the same widths in the **fallback** formats (AVIF and JPEG by default; PNG
  replaces JPEG when the image has transparency),
- `manifest.json` describing the files,
- ready-to-paste code: **HTML**, **Next.js / React**, **WordPress**, **nginx**,
  **Apache**.

Options:

- *File name* and *Public URL path*: where the files will live on your site
  (e.g. `/images/`). The code uses them.
- *Alt text*: describe the image (accessibility and SEO).
- *Hero image*: for the big image at the top of a page. It is loaded eagerly
  with high priority, which improves Largest Contentful Paint.
- *Load the subject first* (on by default): the subject sharpens before the
  background on slow connections.

After exporting, Jixel shows the total size per format (and how much smaller
JPEG XL is) and the code in tabs with a **Copy** button. Step-by-step
instructions for each website stack are in [web-delivery.md](web-delivery.md).

## Studio

Open one image and explore what JPEG XL can do with it. Six tools:

### Compare
Pick JPEG XL, AVIF, WebP or JPEG and a quality; the result appears next to the
original with a draggable divider and a SSIMULACRA2 score. **Save** writes it.

### Subject
Jixel finds the main subject (or drag a rectangle to choose it yourself).
**Create optimised file** encodes the image twice — standard, and with
*subject boost* (subject as a high-quality layer over a cheaper background) —
compares the subject's quality and size, and saves the better one. Either way
the subject loads first.

### Background
Removes the background on-device. Choose *Transparent*, *White* or *Black*,
the format, and *Hard edges* if you need a crisp cut-out (soft edges look
better on hair and fur). The result is shown on a checkerboard.

### Crop & tiles
Drag on the image to select an area and **Save crop**. **Split into tiles**
cuts it into a grid (up to 8×8). **Deep Zoom pyramid** creates the tiles for
zoomable viewers such as OpenSeadragon. All of these are pixel-exact.

### Loading
Shows what a visitor sees while a progressive JPEG XL downloads: drag the
slider from 5% to 100% of the file.

### Format battle
Each format is tuned to reach the same visual quality (choose the target
score), then the file sizes are compared. The winner is highlighted.

## Inspect

Open any image to see: format, dimensions, file size, bits per pixel, bit
depth, transparency (and whether it is actually used), colour profile, EXIF,
GPS location and frames. For JPEG XL files it also shows the mode (lossless or
lossy), the colour description, HDR, and whether the file contains a
recompressed JPEG — with a **Restore original JPEG** button.

**Suggestions** tell you what to do with the file, e.g. "convert to JPEG XL:
about 20% smaller with zero loss" or "this photo contains GPS location".

## Settings

- *Appearance*: System, Light, Dark.
- *Language*: System, English, Italiano.
- *Output folder* (computers): default `Downloads/Jixel`. On phones, results
  stay in the app's folder and are shared with **Share**.
- *Defaults*: quality, encoding speed and metadata used by Convert.
- *About*: version of Jixel and libjxl, open-source licences.

## Understanding the results

Every result carries one of three badges:

| Badge | Meaning |
|---|---|
| **Lossless · original restorable** | The original file can be recreated bit for bit (JPEG → JPEG XL). |
| **Lossless** | Every pixel is identical to the source. |
| **Lossy** | Re-encoded with some loss. The quality score tells you how much. |

**Quality score** (SSIMULACRA2, 0–100): 90+ visually lossless, 80+ very high,
70+ high, 50+ medium, below 50 low.
