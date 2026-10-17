# Domande frequenti

**Jixel è gratuito?**
Sì, e open source (Apache-2.0). Niente account, pubblicità o tracciamento.

**Le mie immagini vengono caricate da qualche parte?**
No. Ogni operazione, compresa la rimozione dello sfondo, avviene sul tuo
dispositivo.

**Il mio sito si rompe nei browser senza JPEG XL?**
No. L'esportazione web di Jixel include sempre formati di riserva dentro
`<picture>`; ogni browser sceglie il primo formato che supporta.

**Convertire JPEG in JPEG XL è davvero senza perdita?**
Sì, per impostazione predefinita. I dati del JPEG vengono reimpacchettati, non
ricodificati: i pixel sono identici e il file originale si può ripristinare
byte per byte (Ispeziona → *Ripristina il JPEG originale*, oppure
`jixel reconstruct`).

**Allora perché il mio JPEG → JPEG XL è solo il 15% più leggero e non il 50%?**
La ricompressione senza perdita fa risparmiare circa il 15–25%. Risparmi
maggiori richiedono la ricodifica (Converti → Avanzate → *Ricodifica i JPEG*),
che è con perdita — da combinare con il ridimensionamento per il web, come fa
Ottimizza per il web.

**JPEG XL o AVIF?**
JPEG XL per foto ad alta qualità, immagini senza perdita, JPEG esistenti,
immagini grandi e HDR, caricamento progressivo. AVIF va meglio a qualità
molto bassa. Per il web: JPEG XL con AVIF come riserva — l'impostazione
predefinita di Jixel.

**Perché AVIF è "senza punteggio" nella sfida tra formati?**
Jixel sa codificare AVIF ma non ancora decodificarlo, quindi non può misurarne
la qualità. AVIF viene codificato a qualità 80 come riferimento.

**Che qualità uso per il mio sito?**
85 (il valore predefinito dell'esportazione web). Dà un SSIMULACRA2 intorno a
80–85: qualità molto alta, differenze solo guardando da vicino.

**Cosa fa "Carica prima il soggetto"?**
Cambia l'ordine dei blocchi 256×256 dell'immagine nel file così che il
soggetto arrivi per primo. Il file è un JPEG XL standard, stesso peso.

**Cosa fa il "subject boost"?**
Memorizza il soggetto come livello ad alta qualità sopra uno sfondo più
economico. Jixel lo confronta con una codifica normale e tiene la migliore.

**Quanto pesa il modello di IA? Funziona sui telefoni vecchi?**
4,6 MB (U²-Netp), incorporato nell'app. Lavora internamente a 320×320, quindi
funziona praticamente su qualsiasi dispositivo; i bordi vengono poi rifiniti a
piena risoluzione.

**Posso usare Jixel nella mia pipeline di build?**
Sì: la CLI `jixel` fa tutto quello che fa l'app, con output `--json`. Vedi
[cli.md](cli.md).

**WordPress dice "Non hai i permessi per caricare questo tipo di file".**
Installa il plugin must-use `wordpress.php` dell'esportazione web: abilita il
caricamento dei `.jxl`. Vedi [web.md](web.md#4-wordpress).

**Le immagini JPEG XL non si vedono in Safari sul mio sito.**
Probabilmente il server invia il `Content-Type` sbagliato. Aggiungi lo snippet
nginx o Apache dell'esportazione web (imposta `image/jxl`).

**Mantiene EXIF? Profili colore? HDR?**
Profili colore e profondità di bit vengono sempre mantenuti. EXIF/XMP: a tua
scelta (predefinito: mantenuti, ma senza la posizione GPS). L'HDR (PQ/HLG) è
conservato in JPEG XL.
