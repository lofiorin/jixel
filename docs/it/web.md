# JPEG XL sui siti web

Questa guida ti porta da un'immagine a una pagina veloce con qualsiasi
tecnologia: HTML semplice, Next.js / React, WordPress, server nginx o Apache.

- [Come funziona](#come-funziona)
- [1. Esporta](#1-esporta)
- [2. HTML semplice / qualsiasi sito statico](#2-html-semplice--qualsiasi-sito-statico)
- [3. Next.js e React](#3-nextjs-e-react)
- [4. WordPress](#4-wordpress)
- [5. Configurazione del server](#5-configurazione-del-server)
- [6. CDN e cache](#6-cdn-e-cache)
- [7. Pipeline di build e tante immagini](#7-pipeline-di-build-e-tante-immagini)
- [Lista di controllo](#lista-di-controllo)

## Come funziona

I browser che supportano JPEG XL scaricano il file `.jxl`; gli altri scelgono
il primo formato di riserva che conoscono. Lo fa l'elemento HTML standard
`<picture>`, che tutti i browser supportano, quindi **nessuna immagine resta
mai rotta**:

```html
<picture>
  <source type="image/jxl"  srcset="/images/hero-480.jxl 480w, /images/hero-1080.jxl 1080w, …" sizes="100vw">
  <source type="image/avif" srcset="/images/hero-480.avif 480w, …" sizes="100vw">
  <img src="/images/hero-1080.jpg" srcset="/images/hero-480.jpg 480w, …" sizes="100vw"
       width="3840" height="2160" alt="…" loading="lazy" decoding="async">
</picture>
```

`srcset` + `sizes` permettono al browser di scegliere la larghezza giusta per
lo schermo, così i telefoni non scaricano immagini da desktop. `width` e
`height` evitano gli spostamenti del layout (CLS). `loading="lazy"` rimanda le
immagini sotto la piega; le immagini principali usano invece
`loading="eager" fetchpriority="high"`.

## 1. Esporta

Nell'app: **Ottimizza per il web**; oppure con la CLI:

```bash
jixel web foto.jpg -o export/ --name hero --alt "Tramonto sulla baia" --hero
```

La cartella esportata contiene:

```
hero-480.jxl  hero-768.jxl  …  hero-2560.jxl
hero-480.avif …                hero-2560.avif
hero-480.jpg  …                hero-2560.jpg
manifest.json
snippets/picture.html
snippets/JxlPicture.tsx
snippets/wordpress.php
snippets/nginx.conf
snippets/apache.htaccess
```

**Scegliere le opzioni**

- *Dimensioni*: quelle predefinite (480, 768, 1080, 1440, 1920, 2560) coprono
  dai telefoni ai grandi schermi desktop a 1× e 2×. Per un'immagine mostrata al
  massimo a 600 px bastano 480/768/1080/1440.
- *Sizes*: dice al browser quanto è larga l'immagine mostrata. A tutta
  larghezza: `100vw`. Metà su desktop: `(min-width: 1024px) 50vw, 100vw`.
- *Qualità*: 85 è un buon valore per le foto (SSIMULACRA2 ≈ 80–85).
- *Formati di riserva*: AVIF + JPEG (predefinito) dà i file più leggeri
  ovunque. WebP + JPEG se vuoi evitare i tempi di codifica AVIF. Solo JPEG è
  la scelta più semplice.

## 2. HTML semplice / qualsiasi sito statico

1. Carica la cartella (anche senza `snippets/`) nel percorso che hai indicato
   come *Percorso URL pubblico*, es. `/images/`.
2. Incolla `snippets/picture.html` dove va l'immagine.
3. Verifica che il server invii il tipo MIME corretto per `.jxl` (vedi
   [configurazione del server](#5-configurazione-del-server)). La maggior
   parte degli hosting lo fa già; se le immagini JPEG XL non compaiono in
   Safari, il motivo è questo.

Funziona con Astro, Hugo, Eleventy, Jekyll, SvelteKit, Nuxt, Laravel Blade,
template Django, viste Rails… qualsiasi cosa produca HTML.

## 3. Next.js e React

`next/image` ricodifica le immagini con il suo ottimizzatore, che non produce
JPEG XL; per questo Jixel genera un piccolo componente che crea direttamente
`<picture>`. Funziona con App Router e Pages Router (componente server, niente
`"use client"`), e con Remix, Gatsby, Vite, CRA.

1. Copia l'esportazione in `public/images/hero/` (con `manifest.json`).
2. Copia `snippets/JxlPicture.tsx` in `components/JxlPicture.tsx` (una volta
   per progetto).
3. Usalo:

```tsx
import JxlPicture from "@/components/JxlPicture";
import hero from "@/public/images/hero/manifest.json";

export default function Page() {
  return <JxlPicture manifest={hero} alt="Tramonto sulla baia" priority sizes="100vw" />;
}
```

Imposta il *Percorso URL pubblico* dell'esportazione sull'URL da cui la
cartella è servita (`/images/hero/` in questo esempio).

Props: `manifest`, `alt`, `sizes`, `priority` (caricamento immediato e
priorità alta), `className`, `style`.

Su Vercel o Netlify i file `.jxl` in `public/` sono serviti automaticamente con
il tipo MIME corretto.

## 4. WordPress

WordPress rifiuta il caricamento dei `.jxl`. Il file `wordpress.php` generato è
un *plugin must-use* che:

- consente il caricamento dei `.jxl` nella Libreria media,
- aggiunge lo shortcode `[jixel name="hero"]` e la funzione PHP
  `jixel_picture('hero')` che mostrano un'esportazione di Jixel come
  `<picture>`.

Passaggi:

1. Copia `snippets/wordpress.php` in `wp-content/mu-plugins/jixel.php` (crea la
   cartella se non c'è). I plugin must-use sono sempre attivi.
2. Carica la cartella esportata in `wp-content/uploads/jixel/hero/` (SFTP, il
   file manager dell'hosting o un plugin come WP File Manager).
3. In un articolo o pagina aggiungi un blocco *Shortcode* con
   `[jixel name="hero"]`. Attributi facoltativi: `alt`, `sizes`, `class`,
   `priority="1"`.
   In un tema: `<?php echo jixel_picture('hero', ['class' => 'cover']); ?>`

I page builder (Elementor, Divi, Gutenberg, WPBakery) supportano tutti gli
shortcode.

**Nota:** i plugin di cache/ottimizzazione che riscrivono `<img>` in WebP (es.
alcune configurazioni di "WebP Express") vanno impostati per ignorare le
immagini dentro `<picture>`.

## 5. Configurazione del server

Contano due cose: il **tipo MIME** (`image/jxl`) e **intestazioni di cache
lunghe** (i nomi dei file non cambiano mai, quindi possono restare in cache per
un anno).

### nginx

Aggiungi `snippets/nginx.conf` al blocco `server { }`. Imposta tipo MIME e
cache. Contiene anche una sezione *facoltativa* di **negoziazione del
contenuto**: servire `foto.jxl` quando la pagina chiede `foto.jpg` e il browser
dichiara di accettare JPEG XL. Così puoi usare JPEG XL su un sito esistente
senza toccarne l'HTML, purché metti `foto.jxl` accanto a `foto.jpg`.

### Apache / hosting condiviso

Metti `snippets/apache.htaccess` nella cartella delle immagini (o uniscilo al
`.htaccess` del sito). Stesse funzioni: tipo MIME, intestazioni di cache,
negoziazione facoltativa con `mod_rewrite` e l'intestazione `Vary: Accept`.

### Altri

- **Caddy**: `.jxl` ha il tipo corretto dalla 2.7.
- **IIS**: aggiungi una mappa MIME `.jxl → image/jxl` in `web.config`.
- **Node / Express**: `express.static` usa `mime`, che conosce `image/jxl`.
- **S3 / R2 / GCS**: imposta `Content-Type: image/jxl` al caricamento
  (`aws s3 cp … --content-type image/jxl`).

## 6. CDN e cache

- Con `<picture>` l'URL indica il formato, quindi qualsiasi CDN funziona senza
  configurazioni aggiuntive.
- Se usi invece la **negoziazione del contenuto**, la CDN deve differenziare la
  cache in base all'intestazione `Accept` (Cloudflare: "Vary for images" o una
  regola di cache; Fastly e CloudFront: includere `Accept` nella chiave di
  cache). Altrimenti un browser senza JPEG XL potrebbe ricevere un `.jxl` dalla
  cache.

## 7. Pipeline di build e tante immagini

La CLI `jixel` produce lo stesso risultato e può stampare JSON:

```bash
# Ogni immagine di una cartella, un'esportazione per immagine
for f in src/images/*.{jpg,png}; do
  jixel web "$f" -o public/images --name "$(basename "${f%.*}")" --json
done
```

- **Script npm**: `"images": "sh scripts/images.sh"` prima di `next build`.
- **GitHub Actions / CI**: installa la CLI una volta, metti in cache la
  cartella di output.
- **WordPress su larga scala**: esporta in locale, sincronizza
  `uploads/jixel/` con rsync.

## Lista di controllo

- [ ] `.jxl` servito come `image/jxl` (controlla la scheda *Rete* degli
      strumenti per sviluppatori)
- [ ] `width`/`height` presenti (nessuno spostamento del layout)
- [ ] Immagine principale esportata con *Immagine principale*; le altre lazy
- [ ] `sizes` corrisponde alla larghezza con cui l'immagine è mostrata
- [ ] Intestazioni di cache lunghe sui file immagine
- [ ] Testo alternativo scritto
- [ ] Provato in un browser **senza** JPEG XL (es. Firefox ESR): deve caricarsi
      il formato di riserva
