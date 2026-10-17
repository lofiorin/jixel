# Command line: `jixel`

The `jixel` command uses the same engine as the app. It is meant for scripts,
build pipelines and servers.

```bash
cargo install --path core/crates/jixel-cli   # after scripts/bootstrap.sh
jixel --help
jixel <command> --help
```

**Conventions**

- Without `-o`, outputs go next to the input with a new extension or suffix.
- Existing files are never replaced unless you pass `--overwrite`.
- `--json` (any command) prints machine-readable JSON instead of text. Errors
  are then `{"error": "…"}` with a non-zero exit code.
- Quality is 0–100 (100 = lossless). Effort is 1–10 (higher = smaller, slower;
  default 7).
- Metadata: `--metadata keep | strip-gps | strip-all` (default `strip-gps`).

## convert

Convert one or many images. The format comes from `-f`, else from the `-o`
file extension, else JPEG XL.

```bash
jixel convert photo.jpg                     # → photo.jxl, lossless recompression
jixel convert *.png -o out/                 # batch into a folder (parallel)
jixel convert photo.jxl -o photo.png        # JPEG XL → PNG
jixel convert photo.jpg -q 85 --reencode-jpeg --max-width 1920 --progressive
jixel convert portrait.jpg --reencode-jpeg --auto-focus   # subject loads first
jixel convert map.png --focus 1200,800      # this point loads first
```

| Option | Meaning |
|---|---|
| `-f, --format` | `jxl`, `avif`, `webp`, `jpg`, `png`, `gif`, `tiff`, `bmp`, `qoi`, `ico`, `tga`, `ppm` |
| `-o, --output` | file (one input) or folder (several) |
| `-q, --quality` | default 90 |
| `-e, --effort` | default 7 |
| `--reencode-jpeg` | lossy re-encode instead of lossless JPEG recompression |
| `--max-width`, `--max-height` | shrink, keeping aspect ratio |
| `--progressive` | progressive JPEG XL |
| `--focus x,y` / `--auto-focus` | group order starts from this point / the detected subject |

## web

Responsive web export with fallbacks and code snippets (see
[web-delivery.md](web-delivery.md)).

```bash
jixel web hero.jpg -o public/images/hero --name hero --alt "…" --hero --auto-focus
jixel web card.png -o out --widths 320,640,960 --fallbacks webp,png --sizes "(min-width: 800px) 33vw, 100vw"
```

| Option | Default |
|---|---|
| `--name` | input file name |
| `--widths` | `480,768,1080,1440,1920,2560` |
| `-q, --quality` | 85 |
| `--fallbacks` | `avif,jpg` (JPEG becomes PNG for transparent images) |
| `--base-url` | `/images/` |
| `--sizes` | `100vw` |
| `--alt`, `--hero`, `--auto-focus` | |

## info

```bash
jixel info photo.jpg image.jxl
jixel info --json photo.jpg | jq .tips
```

Format, size, bit depth, alpha, ICC/EXIF/XMP/GPS, JPEG XL details and
suggestions.

## compare

```bash
jixel compare original.png encoded.jxl     # SSIMULACRA2 score, 0–100
```

## battle

```bash
jixel battle photo.jpg                      # same visual quality (score 80)
jixel battle photo.jpg --score 90 -o battle/   # also save each file
jixel battle photo.jpg -q 80                # same quality *setting* instead
```

Images above 2 megapixels are downscaled for the battle to keep it fast.
AVIF is encoded at quality 80 and not scored (Jixel cannot decode AVIF yet).

## crop

```bash
jixel crop scan.png --rect 100,200,800,600            # pixel-exact JPEG XL
jixel crop photo.jpg --rect 0,0,1920,1080 -q 90 -o crop.jxl
```

## tiles

```bash
jixel tiles poster.png --grid 3x3 -o tiles/
jixel tiles map.png --size 512x512 -f png -o tiles/
```

Tiles are lossless by default (`-q 100`).

## dzi

```bash
jixel dzi plan.png -o viewer/              # plan.dzi + plan_files/<level>/<col>_<row>.jxl
jixel dzi photo.jpg -o viewer/ -f jpg -q 85 --tile-size 510
```

Use with [OpenSeadragon](https://openseadragon.github.io/):
`OpenSeadragon({ id: "viewer", tileSources: "/viewer/plan.dzi" })`.
(Browsers without JPEG XL need `-f jpg` or `-f webp` tiles.)

## rmbg

```bash
jixel rmbg product.jpg                     # → product-cutout.png, transparent
jixel rmbg product.jpg --fill "#ffffff" -f jpg
jixel rmbg logo.jpg --hard -f jxl
```

## subject

```bash
jixel subject portrait.jpg                 # detect subject, boost if it helps
jixel subject portrait.jpg --rect 800,300,900,1200 -q 80
```

Prints both the standard and the boosted result (size and subject score) and
saves the better one.

## reconstruct

```bash
jixel reconstruct photo.jxl                # → photo.jpg, identical to the original
```

## preview

```bash
jixel preview photo.jxl -o frames/ --steps 0.05,0.15,0.3,1
```

Saves PNGs of what the image looks like after each fraction of the file has
downloaded.

## Using JSON output

```bash
jixel --json convert *.jpg -o out | jq -s 'map(.output_bytes) | add'
jixel --json battle photo.jpg | jq '.entries[] | {format, bytes, score}'
```
