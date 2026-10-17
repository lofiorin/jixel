# FAQ

**Is Jixel free?**
Yes, and open source (Apache-2.0). No accounts, no ads, no tracking.

**Are my images uploaded anywhere?**
No. Every operation, including background removal, runs on your device.

**Will my website break in browsers without JPEG XL?**
No. Jixel's web export always includes fallbacks inside `<picture>`; browsers
pick the first format they support.

**Is converting JPEG to JPEG XL really lossless?**
Yes, by default. The JPEG's data is repacked, not re-encoded: the pixels are
identical and the original file can be restored byte for byte (Inspect →
*Restore original JPEG*, or `jixel reconstruct`).

**Then why is my JPEG → JPEG XL only 15% smaller, not 50%?**
Lossless recompression saves ~15–25%. Bigger savings need re-encoding (Convert
→ Advanced → *Re-encode JPEGs instead*), which is lossy — best combined with
resizing for the web, which Optimize for web does.

**JPEG XL or AVIF?**
JPEG XL for high-quality photos, lossless images, existing JPEGs, large and HDR
images, and progressive loading. AVIF is better at very low quality. For the
web, use JPEG XL with AVIF as the fallback — Jixel's default.

**Why is AVIF "not scored" in the format battle?**
Jixel can encode AVIF but can't decode it yet, so it can't measure its
quality. AVIF is encoded at quality 80 for reference.

**What quality should I use for my website?**
85 (the web export default). It gives SSIMULACRA2 around 80–85: very high,
differences only on close inspection.

**What does "Load the subject first" do?**
It changes the order of the image's 256×256 blocks in the file so the subject
arrives first. The file is a standard JPEG XL, same size.

**What does "subject boost" do?**
It stores the subject as a higher-quality layer over a cheaper background.
Jixel compares it with a normal encode and keeps the better one.

**How big is the AI model? Does it work on old phones?**
4.6 MB (U²-Netp), embedded in the app. It runs at 320×320 internally, so it
works on practically any device; edges are then refined at full resolution.

**Can I use Jixel in my build pipeline?**
Yes: the `jixel` CLI does everything the app does, with `--json` output. See
[cli.md](cli.md).

**WordPress says "Sorry, you are not allowed to upload this file type".**
Install the `wordpress.php` must-use plugin from the web export; it enables
`.jxl` uploads. See [web-delivery.md](web-delivery.md#4-wordpress).

**JPEG XL images don't show in Safari on my site.**
Your server probably sends the wrong `Content-Type`. Add the nginx or Apache
snippet from the web export (it sets `image/jxl`).

**Does it keep EXIF? Colour profiles? HDR?**
Colour profiles and bit depth are always kept. EXIF/XMP: your choice (default:
keep, but remove GPS location). HDR (PQ/HLG) is preserved in JPEG XL.
