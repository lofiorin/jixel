# Guida utente

Jixel ha sei sezioni. Su computer sono nella barra laterale; su telefono le
prime cinque sono in basso e **Impostazioni** è l'ingranaggio in alto a destra.

Tutto avviene sul tuo dispositivo. Non viene caricato nulla.

- [Home](#home)
- [Converti](#converti)
- [Ottimizza per il web](#ottimizza-per-il-web)
- [Studio](#studio)
- [Ispeziona](#ispeziona)
- [Impostazioni](#impostazioni)
- [Capire i risultati](#capire-i-risultati)

## Home

Trascina le immagini sull'area grande (computer) o tocca **Scegli file**, poi
scegli cosa fare: **Converti**, **Ottimizza per il web**, **Rimuovi sfondo** o
**Studio**. Puoi anche andare direttamente in una sezione e aprire i file lì.

Formati letti: JPEG XL, JPEG, PNG, WebP, GIF (anche animate), TIFF, BMP, QOI,
ICO, TGA, PNM/PPM/PGM/PAM.

## Converti

1. Aggiungi una o più immagini (puoi trascinarne altre in qualsiasi momento).
2. Scegli il **formato**. JPEG XL è il primo; qualsiasi formato può essere la
   destinazione, quindi da qui converti anche JPEG XL *verso* PNG, JPEG, WebP…
3. Imposta la **qualità**. L'etichetta dice cosa aspettarti:

   | Qualità | Etichetta | Uso tipico |
   |---|---|---|
   | 100 | Senza perdita | screenshot, loghi, archiviazione |
   | 90–99 | Visivamente identica | fotografia, stampa, originali |
   | 80–89 | Alta | siti web (consigliata 85) |
   | 65–79 | Bilanciata | miniature, gallerie |
   | < 65 | Più leggera | anteprime |

4. Premi **Converti**. Ogni file mostra il nuovo peso, il risparmio e un
   bollino di onestà (vedi [Capire i risultati](#capire-i-risultati)).

I **file JPEG** vengono *reimpacchettati* senza perdita per impostazione
predefinita: ~20% più leggeri, pixel identici, e l'originale si può
ripristinare. Il cursore qualità non si applica a loro, a meno che tu non
attivi *Ricodifica i JPEG* in **Avanzate**.

**Prima / dopo:** l'icona di confronto accanto a un file in attesa lo codifica
in memoria con le impostazioni attuali e lo mostra accanto all'originale con
un divisore trascinabile (pizzica o scorri per lo zoom), più un punteggio di
qualità.

**Avanzate**

- *Velocità di codifica*: Veloce, Bilanciata (predefinita), File più leggero.
  Più lenta = più leggera.
- *Caricamento progressivo*: un'anteprima compare mentre l'immagine si scarica.
- *Ricodifica i JPEG*: più leggeri, ma con perdita.
- *Larghezza massima*: riduce le immagini grandi (proporzioni mantenute).
- *Metadati*: Mantieni tutto, Rimuovi posizione (predefinito), Rimuovi tutto.

I risultati vanno nella cartella di destinazione (Impostazioni). Su computer
**Mostra** la apre; su telefono **Condividi** apre il menu di condivisione.

## Ottimizza per il web

Produce tutto il necessario per pubblicare un'immagine su un sito:

- JPEG XL in più larghezze (480–2560 px per impostazione predefinita; le
  larghezze più grandi dell'originale vengono saltate),
- le stesse larghezze nei **formati di riserva** (AVIF e JPEG; PNG sostituisce
  JPEG se l'immagine ha trasparenza),
- `manifest.json` che descrive i file,
- codice pronto da incollare: **HTML**, **Next.js / React**, **WordPress**,
  **nginx**, **Apache**.

Opzioni:

- *Nome file* e *Percorso URL pubblico*: dove staranno i file sul sito (es.
  `/images/`). Il codice li usa.
- *Testo alternativo*: descrivi l'immagine (accessibilità e SEO).
- *Immagine principale*: per la grande immagine in cima alla pagina. Viene
  caricata subito e con priorità alta, migliorando il Largest Contentful Paint.
- *Carica prima il soggetto* (attivo per impostazione predefinita): il soggetto
  diventa nitido prima dello sfondo sulle connessioni lente.

Dopo l'esportazione Jixel mostra il peso totale per formato (e quanto è più
leggero JPEG XL) e il codice in schede con il pulsante **Copia**. Le istruzioni
passo passo per ogni tecnologia sono in [web.md](web.md).

## Studio

Apri un'immagine ed esplora cosa può farne JPEG XL. Sei strumenti:

### Confronta
Scegli JPEG XL, AVIF, WebP o JPEG e una qualità; il risultato compare accanto
all'originale con un divisore trascinabile e un punteggio SSIMULACRA2. **Salva**
lo scrive su disco.

### Soggetto
Jixel trova il soggetto principale (oppure trascina un rettangolo per
sceglierlo tu). **Crea file ottimizzato** codifica l'immagine due volte —
normale e con *subject boost* (soggetto come livello ad alta qualità sopra uno
sfondo più economico) — confronta qualità del soggetto e peso, e salva il
migliore. In entrambi i casi il soggetto viene caricato per primo.

### Sfondo
Rimuove lo sfondo sul dispositivo. Scegli *Trasparente*, *Bianco* o *Nero*, il
formato e *Bordi netti* se ti serve un ritaglio deciso (i bordi morbidi rendono
meglio capelli e pelo). Il risultato è mostrato su una scacchiera.

### Ritaglio e riquadri
Trascina sull'immagine per selezionare un'area e **Salva ritaglio**. **Dividi
in riquadri** la taglia in una griglia (fino a 8×8). **Piramide Deep Zoom** crea
i riquadri per visualizzatori con zoom come OpenSeadragon. Tutto identico al
pixel.

### Caricamento
Mostra cosa vede un visitatore mentre si scarica un JPEG XL progressivo:
trascina il cursore dal 5% al 100% del file.

### Sfida tra formati
Ogni formato viene regolato per raggiungere la stessa qualità visiva (scegli il
punteggio obiettivo), poi si confrontano i pesi. Il vincitore è evidenziato.

## Ispeziona

Apri qualsiasi immagine per vedere: formato, dimensioni, peso, bit per pixel,
profondità di bit, trasparenza (e se è davvero usata), profilo colore, EXIF,
posizione GPS e fotogrammi. Per i file JPEG XL mostra anche la modalità (con o
senza perdita), la descrizione del colore, HDR e se il file contiene un JPEG
ricompresso — con il pulsante **Ripristina il JPEG originale**.

I **Suggerimenti** dicono cosa fare del file, per esempio "converti in JPEG XL:
circa il 20% in meno senza alcuna perdita" oppure "questa foto contiene la
posizione GPS".

## Impostazioni

- *Aspetto*: Sistema, Chiaro, Scuro.
- *Lingua*: Sistema, English, Italiano.
- *Cartella di destinazione* (computer): predefinita `Download/Jixel`. Su
  telefono i risultati restano nella cartella dell'app e si condividono con
  **Condividi**.
- *Predefiniti*: qualità, velocità di codifica e metadati usati da Converti.
- *Informazioni*: versione di Jixel e di libjxl, licenze open source.

## Capire i risultati

Ogni risultato ha uno di tre bollini:

| Bollino | Significato |
|---|---|
| **Senza perdita · originale ripristinabile** | Il file originale si può ricreare bit per bit (JPEG → JPEG XL). |
| **Senza perdita** | Ogni pixel è identico alla sorgente. |
| **Con perdita** | Ricodificato con qualche perdita. Il punteggio di qualità dice quanta. |

**Punteggio di qualità** (SSIMULACRA2, 0–100): 90+ visivamente identica, 80+
molto alta, 70+ alta, 50+ media, sotto 50 bassa.
