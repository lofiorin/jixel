<p align="center">
  <b>Jixel</b> — JPEG XL per tutti<br>
  <i>Immagini più leggere. Stessa qualità. Ogni piattaforma.</i>
</p>

<p align="center">
  <a href="README.md">🇬🇧 Read in English</a> ·
  <a href="docs/it/guida-utente.md">Guida utente</a> ·
  <a href="docs/it/web.md">Immagini per il web</a> ·
  <a href="docs/it/cli.md">CLI</a> ·
  <a href="docs/it/architettura.md">Architettura</a>
</p>

---

Jixel è un'app gratuita e open source per **iOS, Android, macOS, Windows e
Linux** che rende semplice usare [JPEG XL](docs/it/jpeg-xl.md) — soprattutto per
**pubblicare sui siti web immagini più leggere e più nitide**.

- **Converti qualsiasi formato ↔ JPEG XL.** In ingresso JPEG, PNG, WebP, AVIF,
  GIF, TIFF, BMP, QOI, ICO, TGA, PNM; in uscita uno qualsiasi di questi. Anche
  in blocco.
- **I JPEG si riducono del ~20% senza alcuna perdita**, e il JPEG originale si
  può ripristinare identico bit per bit.
- **Ottimizza per il web** con un clic: tutte le dimensioni per ogni schermo,
  formati di riserva per i browser più vecchi e il codice per **HTML,
  Next.js/React, WordPress, nginx, Apache**.
- **Prima il soggetto.** Jixel trova il soggetto principale, lo fa caricare per
  primo e può dargli più dettaglio dello sfondo — in un normale file `.jxl`.
- **Ritaglio e suddivisione in riquadri senza perdita**, piramidi **Deep Zoom**
  per i visualizzatori con zoom.
- **Rimozione dello sfondo** interamente sul dispositivo, con un modello da
  4,6 MB che gira praticamente su qualsiasi telefono o PC.
- **Sfida tra formati**: JPEG XL contro AVIF, WebP e JPEG alla *stessa qualità
  visiva* (SSIMULACRA2), sulle *tue* foto.
- **Ispeziona** qualsiasi file: cosa contiene, privacy (GPS) e cosa farne.
- Onesta per scelta: ogni risultato dice se è **reversibile**, **identico al
  pixel** o **con perdita**.
- Le tue immagini non lasciano mai il dispositivo.

## Perché JPEG XL?

Su una foto 3840×2160, con ogni formato regolato per raggiungere la *stessa*
qualità visiva (SSIMULACRA2 80, "molto alta"):

| Formato | Dimensione | rispetto a JPEG |
|---|---:|---:|
| **JPEG XL** | **194 KB** | **−41%** |
| WebP | 280 KB | −15% |
| JPEG | 329 KB | — |

Per un JPEG esistente, la ricompressione senza perdita ha risparmiato il 21%
con un ripristino identico. Prova `jixel battle tua-foto.jpg` per vedere i tuoi
numeri. La [guida a JPEG XL](docs/it/jpeg-xl.md) spiega da dove vengono i
vantaggi, e quando JPEG XL *non* è la scelta migliore.

Supporto dei browser (ottobre 2026): Safari 17+, Chrome 155+ e Firefox 157+
decodificano JPEG XL. Jixel genera sempre un formato di riserva per tutti gli
altri — vedi [Immagini per il web](docs/it/web.md).

## Per iniziare

### App

Scarica una build dalle Release (con la prima versione pubblica) oppure
[compilala](docs/it/compilazione.md):

```bash
scripts/bootstrap.sh
cd app && flutter run
```

### CLI

```bash
scripts/bootstrap.sh
cargo install --path core/crates/jixel-cli

jixel convert foto/*.jpg -o out/            # JPEG → JPEG XL senza perdita
jixel web hero.png -o public/images --hero  # set responsive + snippet
jixel battle foto.jpg                       # confronta i formati
```

Riferimento completo: [docs/it/cli.md](docs/it/cli.md).

## Struttura del progetto

```
core/                   workspace Rust — il motore
  crates/jixel-libjxl-sys   libjxl compilata dai sorgenti + FFI scritta a mano
  crates/jixel-core         tutte le operazioni sulle immagini, come funzioni
  crates/jixel-cli          il comando `jixel`
app/                    app Flutter (tutte e cinque le piattaforme)
  rust/                     ponte C ABI tra Dart e jixel-core
  rust_builder/             plugin Flutter che compila il ponte (cargokit)
  lib/                      interfaccia: engine/, state/, ui/screens, ui/widgets
models/                 modello U²-Netp per la rimozione dello sfondo (4,6 MB)
docs/                   documentazione (en, it), ADR, specifica di progetto
scripts/                script di preparazione e di supporto
```

Comincia da [architettura.md](docs/it/architettura.md): 10 minuti di lettura
che associano ogni cartella alla sua responsabilità.

## Documentazione

| | Italiano | English |
|---|---|---|
| Cos'è JPEG XL e perché conta | [jpeg-xl.md](docs/it/jpeg-xl.md) | [jpeg-xl.md](docs/en/jpeg-xl.md) |
| Usare l'app | [guida-utente.md](docs/it/guida-utente.md) | [user-guide.md](docs/en/user-guide.md) |
| JPEG XL sui siti web | [web.md](docs/it/web.md) | [web-delivery.md](docs/en/web-delivery.md) |
| Riga di comando | [cli.md](docs/it/cli.md) | [cli.md](docs/en/cli.md) |
| Organizzazione del codice | [architettura.md](docs/it/architettura.md) | [architecture.md](docs/en/architecture.md) |
| Compilare dai sorgenti | [compilazione.md](docs/it/compilazione.md) | [building.md](docs/en/building.md) |
| Domande frequenti | [faq.md](docs/it/faq.md) | [faq.md](docs/en/faq.md) |
| Decisioni di progetto | | [docs/adr](docs/adr) |

## Contribuire

I contributi sono benvenuti — codice, traduzioni, documentazione, segnalazioni
di bug con immagini di esempio. Leggi prima [CONTRIBUTING.md](CONTRIBUTING.md)
(issue e PR anche in italiano). Rispetta il [Codice di condotta](CODE_OF_CONDUCT.md)
e segnala le vulnerabilità come descritto in [SECURITY.md](SECURITY.md).

## Licenza

Jixel è distribuita con [licenza Apache 2.0](LICENSE). Include libjxl
(BSD-3-Clause), il modello U²-Netp (Apache-2.0) e altri componenti open source
elencati in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
