# Riga di comando: `jixel`

Il comando `jixel` usa lo stesso motore dell'app. È pensato per script,
pipeline di build e server.

```bash
cargo install --path core/crates/jixel-cli   # dopo scripts/bootstrap.sh
jixel --help
jixel <comando> --help
```

**Convenzioni**

- Senza `-o`, i risultati vengono scritti accanto al file di ingresso con una
  nuova estensione o un suffisso.
- I file esistenti non vengono mai sovrascritti, a meno di `--overwrite`.
- `--json` (qualsiasi comando) stampa JSON leggibile da programmi. Gli errori
  diventano `{"error": "…"}` con codice di uscita diverso da zero.
- Qualità 0–100 (100 = senza perdita). Effort 1–10 (più alto = più leggero, più
  lento; predefinito 7).
- Metadati: `--metadata keep | strip-gps | strip-all` (predefinito `strip-gps`).

## convert

Converte una o più immagini. Il formato si prende da `-f`, altrimenti
dall'estensione di `-o`, altrimenti JPEG XL.

```bash
jixel convert foto.jpg                      # → foto.jxl, ricompressione senza perdita
jixel convert *.png -o out/                 # in blocco in una cartella (in parallelo)
jixel convert foto.jxl -o foto.png          # JPEG XL → PNG
jixel convert foto.jpg -q 85 --reencode-jpeg --max-width 1920 --progressive
jixel convert ritratto.jpg --reencode-jpeg --auto-focus   # il soggetto arriva per primo
jixel convert mappa.png --focus 1200,800    # questo punto arriva per primo
```

| Opzione | Significato |
|---|---|
| `-f, --format` | `jxl`, `avif`, `webp`, `jpg`, `png`, `gif`, `tiff`, `bmp`, `qoi`, `ico`, `tga`, `ppm` |
| `-o, --output` | file (un ingresso) o cartella (più ingressi) |
| `-q, --quality` | predefinito 90 |
| `-e, --effort` | predefinito 7 |
| `--reencode-jpeg` | ricodifica con perdita invece della ricompressione JPEG senza perdita |
| `--max-width`, `--max-height` | riduce mantenendo le proporzioni |
| `--progressive` | JPEG XL progressivo |
| `--focus x,y` / `--auto-focus` | l'ordine dei gruppi parte da questo punto / dal soggetto rilevato |

## web

Esportazione web responsive con formati di riserva e snippet (vedi
[web.md](web.md)).

```bash
jixel web hero.jpg -o public/images/hero --name hero --alt "…" --hero --auto-focus
jixel web card.png -o out --widths 320,640,960 --fallbacks webp,png --sizes "(min-width: 800px) 33vw, 100vw"
```

| Opzione | Predefinito |
|---|---|
| `--name` | nome del file di ingresso |
| `--widths` | `480,768,1080,1440,1920,2560` |
| `-q, --quality` | 85 |
| `--fallbacks` | `avif,jpg` (JPEG diventa PNG per le immagini trasparenti) |
| `--base-url` | `/images/` |
| `--sizes` | `100vw` |
| `--alt`, `--hero`, `--auto-focus` | |

## info

```bash
jixel info foto.jpg immagine.jxl
jixel info --json foto.jpg | jq .tips
```

Formato, peso, profondità di bit, alfa, ICC/EXIF/XMP/GPS, dettagli JPEG XL e
suggerimenti.

## compare

```bash
jixel compare originale.png codificata.jxl   # punteggio SSIMULACRA2, 0–100
```

## battle

```bash
jixel battle foto.jpg                       # stessa qualità visiva (punteggio 80)
jixel battle foto.jpg --score 90 -o sfida/  # salva anche ogni file
jixel battle foto.jpg -q 80                 # stessa *impostazione* di qualità
```

Le immagini oltre i 2 megapixel vengono ridotte per la sfida, per tenerla
veloce. AVIF viene codificato a qualità 80 senza punteggio (Jixel non sa ancora
decodificare AVIF).

## crop

```bash
jixel crop scansione.png --rect 100,200,800,600     # JPEG XL identico al pixel
jixel crop foto.jpg --rect 0,0,1920,1080 -q 90 -o ritaglio.jxl
```

## tiles

```bash
jixel tiles poster.png --grid 3x3 -o riquadri/
jixel tiles mappa.png --size 512x512 -f png -o riquadri/
```

I riquadri sono senza perdita per impostazione predefinita (`-q 100`).

## dzi

```bash
jixel dzi pianta.png -o viewer/            # pianta.dzi + pianta_files/<livello>/<col>_<riga>.jxl
jixel dzi foto.jpg -o viewer/ -f jpg -q 85 --tile-size 510
```

Da usare con [OpenSeadragon](https://openseadragon.github.io/):
`OpenSeadragon({ id: "viewer", tileSources: "/viewer/pianta.dzi" })`.
(Per i browser senza JPEG XL servono riquadri `-f jpg` o `-f webp`.)

## rmbg

```bash
jixel rmbg prodotto.jpg                    # → prodotto-cutout.png, trasparente
jixel rmbg prodotto.jpg --fill "#ffffff" -f jpg
jixel rmbg logo.jpg --hard -f jxl
```

## subject

```bash
jixel subject ritratto.jpg                 # rileva il soggetto, boost se conviene
jixel subject ritratto.jpg --rect 800,300,900,1200 -q 80
```

Stampa sia il risultato normale sia quello con boost (peso e punteggio del
soggetto) e salva il migliore.

## reconstruct

```bash
jixel reconstruct foto.jxl                 # → foto.jpg, identico all'originale
```

## preview

```bash
jixel preview foto.jxl -o fotogrammi/ --steps 0.05,0.15,0.3,1
```

Salva dei PNG di come appare l'immagine dopo ogni frazione di file scaricata.

## Usare l'output JSON

```bash
jixel --json convert *.jpg -o out | jq -s 'map(.output_bytes) | add'
jixel --json battle foto.jpg | jq '.entries[] | {format, bytes, score}'
```
