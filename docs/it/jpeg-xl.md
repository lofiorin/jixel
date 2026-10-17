# JPEG XL, spiegato

Questa guida spiega cos'è JPEG XL, cosa fa meglio dei formati che usi già,
quando *non* è la scelta migliore e quale strumento di Jixel usa ciascuna
funzione. Non serve alcuna conoscenza di compressione delle immagini.

- [In un minuto](#in-un-minuto)
- [Le funzioni, una per una](#le-funzioni-una-per-una)
- [Confronto con WebP, AVIF e JPEG](#confronto-con-webp-avif-e-jpeg)
- [Supporto di browser e software](#supporto-di-browser-e-software)
- [Misurare la qualità: SSIMULACRA2](#misurare-la-qualità-ssimulacra2)
- [Limiti e avvertenze oneste](#limiti-e-avvertenze-oneste)
- [Approfondimenti](#approfondimenti)

## In un minuto

JPEG XL (ISO/IEC 18181, estensione `.jxl`) è un formato di immagine libero da
royalty, progettato dal comitato JPEG per sostituire JPEG, PNG, GIF e WebP con
un unico formato. Rispetto a ciò che usano oggi la maggior parte dei siti:

- **File più piccoli a parità di qualità visiva.** Di solito il 20–40% in meno
  di WebP e il 35–60% in meno di JPEG per foto ad alta qualità.
- **Difetti meno visibili a parità di peso.** JPEG XL degrada con eleganza:
  sfoca leggermente invece di creare blocchi, bande o colori impastati.
- **Senza perdita quando serve,** il 30–50% più leggero di PNG.
- **I JPEG esistenti si riducono del ~20% senza perdita,** e si possono
  riportare al file originale identico.
- **Funzioni che nessun altro formato web ha:** caricamento del soggetto per
  primo, decodifica progressiva, livelli, ritaglio senza perdita, HDR fino a
  32 bit, animazione, immagini enormi (circa un miliardo di pixel per lato).

## Le funzioni, una per una

### 1. Compressione percettiva con perdita (VarDCT + XYB)

JPEG XL con perdita memorizza il colore in **XYB**, uno spazio colore modellato
sull'occhio umano, e usa trasformate di dimensione variabile (da 2×2 a 256×256)
scelte zona per zona: blocchi grandi per il cielo, piccoli per bordi e
texture. L'encoder è regolato con la metrica percettiva **Butteraugli**, così
spende i bit dove le persone notano le differenze.

L'impostazione dell'encoder è una *distanza* (0 = senza perdita, 1 =
visivamente senza perdita, 3 = "qualità web"…). Jixel mostra un normale
cursore qualità 0–100 e lo converte in distanza come fa lo strumento ufficiale
`cjxl`.

**In Jixel:** Converti, Ottimizza per il web, Studio → Confronta.

### 2. Ricompressione JPEG senza perdita

JPEG XL può prendere un JPEG esistente e memorizzare gli *stessi* coefficienti
DCT con una codifica entropica molto migliore. Risultato: circa **il 20% in
meno**, **nessun cambiamento nei pixel**, e il file JPEG originale si può
**ricostruire bit per bit** (il `.jxl` conserva un piccolo box `jbrd` con il
necessario).

È l'unico vero "pasto gratis" della compressione: puoi convertire un intero
archivio fotografico senza alcun rischio.

**In Jixel:** convertire un JPEG in JPEG XL fa questo per impostazione
predefinita (risultato *Senza perdita · originale ripristinabile*). Ispeziona →
*Ripristina il JPEG originale* torna indietro. CLI: `jixel convert`,
`jixel reconstruct`.

### 3. Modalità modular: davvero senza perdita

Per screenshot, loghi, disegni, interfacce, immagini mediche e scientifiche,
la modalità *modular* di JPEG XL è senza perdita e di solito il 30–50% più
leggera di un PNG ottimizzato, e più leggera di WebP lossless. Supporta interi
fino a 16 bit e float a 32 bit.

**In Jixel:** qualità 100 (etichetta *Senza perdita*). Ispezionando un PNG Jixel
la suggerisce.

### 4. Decodifica progressiva

Un file JPEG XL può essere organizzato in modo che un'anteprima utilizzabile
compaia dopo che ne è arrivata una piccola parte, per poi diventare nitida a
passaggi. Ogni JPEG XL con perdita inizia con un'immagine "DC" a 1:8; i file
progressivi aggiungono altri passaggi.

Quanto presto compare la prima anteprima dipende dall'immagine: per una foto 4K
molto dettagliata l'immagine 1:8 era circa il 19% del file nelle nostre prove;
per le tipiche foto web è intorno al 10–15%.

**In Jixel:** *Caricamento progressivo* (Converti → Avanzate; sempre attivo in
Ottimizza per il web). Studio → Caricamento simula un download lento.

### 5. Ordine dei gruppi: il soggetto arriva per primo ("soggetto principale")

JPEG XL divide l'immagine in *gruppi* da 256×256 e permette all'encoder di
sceglierne l'ordine nel file. Invece che dall'alto in basso, Jixel può scrivere
i gruppi **partendo da un punto di interesse e allargandosi a spirale**, così
su una connessione lenta il soggetto è nitido prima dello sfondo. Il file è un
normale JPEG XL: ogni decoder lo capisce.

**In Jixel:** Ottimizza per il web → *Carica prima il soggetto* (attivo per
impostazione predefinita); Studio → Soggetto; CLI `--auto-focus` o
`--focus x,y`.

### 6. Livelli: più dettaglio al soggetto ("subject boost")

Un'immagine JPEG XL può contenere più **livelli** (frame composti in una sola
immagine). Jixel lo usa per codificare lo sfondo a qualità più bassa e il
soggetto come secondo livello, a qualità più alta, sopra di esso. Sotto il
soggetto lo sfondo viene appiattito, così non costa quasi nulla.

Jixel codifica sempre sia il file normale sia quello a livelli, valuta la zona
del soggetto con SSIMULACRA2 e **tiene il migliore**. Sulla nostra foto di
prova il file a livelli era il 16% più leggero *e* il soggetto aveva un
punteggio di 82,9 invece di 76,5.

**In Jixel:** Studio → Soggetto → *Crea file ottimizzato*. CLI:
`jixel subject`.

### 7. Ritaglio e riquadri senza perdita

Ritagliare o dividere un'immagine di solito significa decodificarla e
ricodificarla, perdendo qualità ogni volta. Jixel salva ritagli e riquadri
**identici al pixel** (JPEG XL senza perdita è abbastanza efficiente da
renderlo pratico). Può anche creare una **piramide Deep Zoom (DZI)** per
visualizzatori come OpenSeadragon — mappe, planimetrie, dettagli di prodotto,
opere d'arte.

**In Jixel:** Studio → Ritaglio e riquadri. CLI: `jixel crop`, `jixel tiles`,
`jixel dzi`.

### 8. HDR, gamut esteso e alta profondità di bit

JPEG XL gestisce da 8 a 32 bit per canale, le curve HDR PQ e HLG e qualsiasi
profilo colore ICC, in modo nativo. Jixel conserva profondità di bit e profili
nelle conversioni e ti avvisa (Ispeziona) quando un file è HDR.

### 9. Trasparenza e animazione

Canale alfa completo (con colore con o senza perdita) e animazioni a colori
pieni invece dei 256 della GIF. Una GIF animata convertita in JPEG XL di solito
pesa molto meno.

### 10. Metadati e privacy

EXIF, XMP e JUMBF stanno in box del contenitore, eventualmente compressi con
Brotli. Jixel rimuove la posizione GPS per impostazione predefinita quando
esporta (scegli *Mantieni tutto*, *Rimuovi posizione* o *Rimuovi tutto*).

## Confronto con WebP, AVIF e JPEG

| | JPEG XL | AVIF | WebP | JPEG |
|---|---|---|---|---|
| Qualità per byte, foto ad alta qualità | **migliore** | molto buona | buona | riferimento |
| Qualità per byte, qualità molto bassa | buona | **migliore** | discreta | scarsa |
| Senza perdita | **migliore** | debole | buona | — |
| Ricompressione JPEG senza perdita | **sì, ~20%** | no | no | — |
| Decodifica progressiva | **sì** | no | no | sì |
| Soggetto caricato per primo | **sì** | no | no | no |
| Profondità massima | **32 (float)** | 12 | 8 | 8 (12) |
| Dimensione massima | ~10⁹ px/lato | 65 536 (a riquadri) | 16 383 | 65 535 |
| Velocità di codifica | veloce | lenta | veloce | velocissima |
| Supporto browser (2026) | Safari, Chrome, Firefox | tutti i moderni | tutti | tutti |

**WebP** è limitato a 8 bit e dimezza la risoluzione del colore (4:2:0): da qui
i rossi sfocati e i cieli a bande. JPEG XL mantiene il colore pieno e vince
nettamente ai livelli di qualità che si pubblicano davvero (punteggio ≥ 70).

**AVIF** è eccellente a bitrate *molto* bassi ed è il miglior *formato di
riserva* per i browser senza JPEG XL: per questo Jixel lo usa come prima
riserva. Ad alta qualità JPEG XL è più leggero, codifica molto più in fretta,
si decodifica in modo progressivo e supporta la ricompressione dei JPEG senza
perdita.

## Supporto di browser e software

A ottobre 2026:

- **Safari** 17+ (macOS Sonoma, iOS 17) — supporto completo.
- **Chrome / Edge** 155+ — decodificato con `jxl-rs`, il decoder in Rust a
  memoria sicura.
- **Firefox** 157+ — attivo per impostazione predefinita.
- Browser più vecchi e Firefox ESR richiedono un formato di riserva: Jixel lo
  crea sempre con `<picture>`, così nessuno vede un'immagine rotta.
- **Sistemi operativi:** macOS 14+ e iOS 17+ nativamente; Windows 11 con
  l'estensione JPEG XL; la maggior parte dei visualizzatori Linux tramite libjxl.
- **Software:** Photoshop, Lightroom, Affinity, GIMP, Krita, darktable,
  ImageMagick, libvips, ffmpeg.
- **WordPress** rifiuta ancora il caricamento di `.jxl`; lo snippet WordPress
  di Jixel lo risolve (vedi [Immagini per il web](web.md)).

## Misurare la qualità: SSIMULACRA2

Jixel valuta i risultati con **SSIMULACRA2**, la metrica sviluppata dagli
autori di JPEG XL e validata con ampi studi su persone reali. Scala 0–100:

| Punteggio | Significato |
|---|---|
| 90+ | visivamente senza perdita: indistinguibile a visione normale |
| 80–90 | molto alta: differenze solo guardando da vicino |
| 70–80 | alta: buona per il web |
| 50–70 | media: difetti visibili |
| < 50 | bassa |

La sfida tra formati la usa per portare ogni formato allo **stesso punteggio**
e poi confrontare i pesi — l'unico modo equo per confrontare i formati.

## Limiti e avvertenze oneste

- **Non tutti i browser, ancora.** Prevedi sempre un formato di riserva (Jixel
  lo fa).
- **Qualità molto bassa:** sotto un punteggio di ~60 spesso vince AVIF. Non
  spingere JPEG XL così in basso: non è progettato per questo.
- **Immagini già piccolissime** (icone, miniature) guadagnano meno; il costo
  dei formati di riserva potrebbe non valerne la pena.
- **Ricodificare un JPEG con perdita** accumula degrado: preferisci la
  ricompressione senza perdita, a meno che tu non stia anche ridimensionando.
- **Lettura AVIF:** Jixel sa *scrivere* AVIF ma non ancora *leggerlo*, quindi
  nella sfida tra formati AVIF compare senza punteggio.

## Approfondimenti

- [jpeg.org/jpegxl](https://jpeg.org/jpegxl/) — pagina ufficiale
- [libjxl](https://github.com/libjxl/libjxl) — implementazione di riferimento
- Alakuijala et al., *JPEG XL next-generation image compression architecture and coding tools* (SPIE 2019)
- [jpegxl.info](https://jpegxl.info) — risorse e confronti della comunità
- [SSIMULACRA2](https://github.com/cloudinary/ssimulacra2)
