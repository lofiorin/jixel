# Architettura

Una mappa del codice in 10 minuti. Se stai per modificare qualcosa, qui trovi
dove si trova e perché è fatto così.

## Il quadro generale

```
┌──────────────────────────── App Flutter (app/lib) ───────────────────────────┐
│ ui/screens  ──►  state/  ──►  engine/engine.dart  (API asincrona tipizzata)  │
│                                     │  JSON + buffer di byte, isolate separato│
│                                     ▼                                         │
│                         engine/native.dart  (dart:ffi)                        │
└─────────────────────────────────────┬─────────────────────────────────────────┘
                                      │  C ABI: jixel_call / jixel_free
┌─────────────────────────────────────▼─────────────────────────────────────────┐
│ app/rust (jixel_bridge) — decodifica il JSON, chiama jixel-core, restituisce  │
└─────────────────────────────────────┬─────────────────────────────────────────┘
                                      │  normali chiamate Rust
┌────────────── core/ (workspace Rust) ▼───────────────────────────────────────┐
│ jixel-core: raster · format · codec · convert · ops · analyze · web          │
│        │                                                                      │
│        ▼                                                                      │
│ jixel-libjxl-sys: libjxl 0.12 compilata dai sorgenti + FFI scritta a mano     │
│ jixel-cli: il comando `jixel`, stesso motore                                  │
└───────────────────────────────────────────────────────────────────────────────┘
```

**Regola d'oro:** la logica sulle immagini vive in `jixel-core`, *solo* lì. La
CLI e il ponte traducono gli argomenti e stampano o restituiscono i risultati;
l'app Flutter si limita a mostrarli. Così il motore è testabile senza
interfaccia e identico ovunque.

## core/crates/jixel-libjxl-sys

- `build.rs` compila con CMake libjxl e le sue tre dipendenze incluse (brotli,
  highway, skcms) come librerie statiche, per la piattaforma *di destinazione*
  (così la cross-compilazione per iOS/Android funziona). libjxl è sempre
  compilata in Release: in debug è ~20 volte più lenta.
- `src/lib.rs` dichiara il sottoinsieme dell'API C di libjxl che usiamo.
  Scritta a mano invece di usare `jpegxl-sys` perché quel crate è GPL e Jixel è
  Apache-2.0 ([ADR 0002](../adr/0002-own-libjxl-bindings.md)).

## core/crates/jixel-core

| Modulo | Responsabilità |
|---|---|
| `raster` | `Raster`: larghezza, altezza, canali, campioni 8/16/32f, ICC, fotogrammi. Ogni modulo si passa questo. |
| `format` | Riconoscimento del `Format` dai byte (magic number) e capacità (alfa, lossless, animazione). |
| `codec::jxl` | Encoder/decoder sicuri su libjxl: distanza dalla qualità, effort, progressivo, punto focale (ordine dei gruppi), livelli, ricompressione e ricostruzione JPEG, decodifica parziale (anteprima progressiva), box dei metadati. |
| `codec::raster` | Tutti gli altri formati con il crate `image` (+ `ravif` per scrivere AVIF). |
| `metadata` | Estrazione EXIF/XMP, rimozione GPS, orientamento. |
| `convert` | `convert(bytes, options) -> ConvertOutput` — una sola chiamata da qualsiasi formato a qualsiasi formato, con una `Fidelity` (`Reversible`, `PixelExact`, `Lossy`). |
| `ops::resize` | Riduzione di alta qualità (Lanczos3 con `fast_image_resize`). |
| `ops::crop` | Ritagli identici al pixel. |
| `ops::tiles` | Riquadri a griglia/dimensione e piramidi Deep Zoom. |
| `ops::subject` | Rilevamento del soggetto (dalla maschera dello sfondo), punto focale, *subject boost* (codifica a livelli, tiene il migliore). |
| `ops::background` | Inferenza U²-Netp con `tract-onnx`, rifinitura dei bordi con guided filter. Dietro la feature `background-removal` (attiva di default). |
| `analyze::inspect` | `Inspection` + codici `Tip` (traducibili). |
| `analyze::compare` | Punteggio SSIMULACRA2 e sfida tra formati (ricerca della qualità per formato). |
| `analyze::progressive` | Come appare un JPEG XL dopo l'N% dei suoi byte. |
| `web` | Esportazione responsive, `Manifest` e `snippets` (HTML, React, WordPress, nginx, Apache). |

Gli errori usano un solo enum `Error` (`thiserror`), con messaggi scritti per
gli utenti.

### Scelte chiave

- **Risultati onesti.** Ogni conversione riporta la sua `Fidelity`. L'interfaccia
  la mostra come bollino; niente è chiamato "senza perdita" se non lo è.
- **JPEG → JPEG XL è ricompressione senza perdita di default.** La ricodifica è
  facoltativa.
- **Confronti equi.** La sfida porta ogni formato allo *stesso punteggio
  SSIMULACRA2* con una ricerca binaria, invece di confrontare "qualità 80" che
  significa cose diverse per ogni formato.
- **Il subject boost tiene il vincitore.** La codifica a livelli non è sempre
  migliore: si codificano e valutano entrambe e si scrive la migliore.
- **Modello incorporato** (4,6 MB). Nessun download in esecuzione, funziona
  offline.

## core/crates/jixel-cli

`clap` derive. Un sottocomando per funzione. `--json` ovunque. I lotti girano
in parallelo con `rayon`. Nessuna logica sulle immagini oltre al parsing degli
argomenti.

## app/rust (il ponte)

Una sola funzione C:

```c
JixelResponse* jixel_call(const char* method, const char* args_json,
                          const JixelBuffer* buffers, size_t n_buffers);
void jixel_free(JixelResponse*);
```

`method` è ad esempio `"convert"`, gli argomenti sono JSON, i dati grandi
(pixel, file codificati) viaggiano come buffer grezzi accanto al JSON. La
risposta è `{"ok": …}` o `{"error": "…"}`; i panic vengono intercettati. Ogni
metodo è documentato in cima al suo ramo `match` in `src/lib.rs`.

Perché non flutter_rust_bridge? Vedi [ADR 0003](../adr/0003-json-ffi-bridge.md):
una sola funzione stabile è più facile da leggere, da debuggare e da compilare
su cinque piattaforme rispetto a binding generati, e il costo del JSON è
trascurabile rispetto al lavoro sulle immagini.

Il ponte è compilato da **cargokit** (incluso in `app/rust_builder/cargokit`)
tramite il plugin Flutter `jixel_native`: script phase di CocoaPods su
iOS/macOS, Gradle su Android, CMake su Windows/Linux.

## app/lib (Flutter)

```
main.dart            avvio: carica le impostazioni, avvia l'app, legge le info del motore
app.dart             MaterialApp: temi, lingue, AppScope
engine/
  native.dart        binding dart:ffi, callNative() su un isolate separato
  models.dart        viste tipizzate sul JSON del motore (rispecchiano gli struct Rust)
  engine.dart        Engine.convert(), Engine.webExport(), … — usa questo
state/
  settings.dart      preferenze salvate (shared_preferences)
  app_state.dart     sezione corrente, file passati tra le sezioni
  convert_queue.dart l'elenco di Converti e il suo avanzamento
ui/
  shell.dart         barra laterale (≥ 840 px) o barra in basso
  theme.dart         tema Material 3 da un solo colore, spaziature
  format.dart        formattazione di byte e percentuali
  widgets/           area di rilascio, anteprime, confronto, selezione rettangolo, controlli
  screens/           home, convert, web_export, studio (+ studio/*), inspect, settings
l10n/                app_en.arb, app_it.arb (generati da tool/strings.py)
```

- **Nessun pacchetto di state management.** `ChangeNotifier` + un solo
  `InheritedNotifier` (`AppScope`) bastano e non richiedono spiegazioni.
- **Tutte le chiamate al motore sono asincrone e fuori dal thread
  dell'interfaccia** (`Isolate.run`).
- **File, non byte.** Le operazioni ricevono percorsi e scrivono file; solo le
  anteprime (RGBA, al massimo 1600–2048 px) arrivano a Dart.

### Traduzioni

I testi stanno in un'unica tabella, `app/tool/strings.py` (chiave → inglese,
italiano). Esegui `python3 tool/strings.py`, poi `flutter gen-l10n` e infine `dart format --line-length 120 lib/l10n`. Per
aggiungere una lingua vedi [CONTRIBUTING](../../CONTRIBUTING.md).

## Test

| Dove | Cosa | Comando |
|---|---|---|
| `core/**` | test unitari e di andata e ritorno (codifica/decodifica, garanzie senza perdita, ricostruzione JPEG identica, riassemblaggio dei riquadri, esportazione web, snippet) | `cargo test --release` in `core/` |
| `app/rust` | andata e ritorno attraverso il ponte | `cargo test --release` in `app/rust/` |
| `app/test` | formattazione, JSON delle impostazioni, test dei widget (layout, navigazione, lingue) | `flutter test` in `app/` |
| `app/integration_test` | motore reale su dispositivo/desktop, rendering delle schermate | `flutter test integration_test -d macos` |

## Approfondimenti

- [Specifica di progetto](../superpowers/specs/2026-10-17-jixel-design.md) (in inglese)
- [Piano di implementazione](../superpowers/plans/2026-10-17-jixel-v0.1.md) (in inglese)
- [ADR](../adr) (in inglese)
