#!/usr/bin/env python3
"""Single source for the app's UI strings in English and Italian.

Edit this table, then run `python3 tool/strings.py` (from app/) to rewrite
lib/l10n/app_en.arb and lib/l10n/app_it.arb, and `flutter gen-l10n`.
Keeping both languages side by side makes missing translations obvious.
"""
import json, os, re

S = {
 # key: (English, Italian)
 "appTitle": ("Jixel", "Jixel"),
 "navHome": ("Home", "Home"),
 "navConvert": ("Convert", "Converti"),
 "navWeb": ("Web", "Web"),
 "navStudio": ("Studio", "Studio"),
 "navInspect": ("Inspect", "Ispeziona"),
 "navSettings": ("Settings", "Impostazioni"),

 "homeHeadline": ("Smaller images. Same quality.", "Immagini più leggere. Stessa qualità."),
 "homeSubtitle": ("Convert, optimise and inspect images with JPEG XL. Everything happens on your device: nothing is uploaded.",
                  "Converti, ottimizza e analizza immagini con JPEG XL. Tutto avviene sul tuo dispositivo: nessun file viene caricato online."),
 "dropTitle": ("Drop images here", "Trascina qui le immagini"),
 "dropOr": ("or", "oppure"),
 "chooseFiles": ("Choose files", "Scegli file"),
 "chooseImage": ("Choose image", "Scegli immagine"),
 "changeImage": ("Change image", "Cambia immagine"),
 "supportedFormats": ("Reads JPEG XL, JPEG, PNG, WebP, GIF, TIFF, BMP and more.", "Legge JPEG XL, JPEG, PNG, WebP, GIF, TIFF, BMP e altri."),
 "whatToDo": ("What do you want to do?", "Cosa vuoi fare?"),
 "taskConvertTitle": ("Convert", "Converti"),
 "taskConvertBody": ("Any format to JPEG XL and back. JPEGs shrink by about 20% with zero loss.",
                     "Da qualsiasi formato a JPEG XL e ritorno. I JPEG si riducono di circa il 20% senza alcuna perdita."),
 "taskWebTitle": ("Optimize for web", "Ottimizza per il web"),
 "taskWebBody": ("Every screen size, fallbacks for old browsers, and code for HTML, Next.js and WordPress.",
                 "Tutte le dimensioni per ogni schermo, alternative per i browser datati e codice per HTML, Next.js e WordPress."),
 "taskBackgroundTitle": ("Remove background", "Rimuovi sfondo"),
 "taskBackgroundBody": ("On-device AI. Transparent or solid colour, ready for your shop.",
                        "IA sul dispositivo. Sfondo trasparente o a tinta unita, pronto per il tuo negozio online."),
 "taskStudioTitle": ("Studio", "Studio"),
 "taskStudioBody": ("Compare quality, sharpen the subject, crop, split into tiles, preview loading.",
                    "Confronta la qualità, valorizza il soggetto, ritaglia, dividi in riquadri, simula il caricamento."),

 "addFiles": ("Add files", "Aggiungi file"),
 "clearList": ("Clear", "Svuota"),
 "convertCount": ("{count, plural, =1{Convert 1 image} other{Convert {count} images}}", "{count, plural, =1{Converti 1 immagine} other{Converti {count} immagini}}"),
 "emptyQueue": ("Add images to start", "Aggiungi immagini per iniziare"),
 "formatLabel": ("Format", "Formato"),
 "qualityLabel": ("Quality", "Qualità"),
 "qualityLossless": ("Lossless", "Senza perdita"),
 "qualityVisuallyLossless": ("Visually lossless", "Visivamente identica"),
 "qualityHigh": ("High", "Alta"),
 "qualityBalanced": ("Balanced", "Bilanciata"),
 "qualitySmall": ("Smallest", "Più leggera"),
 "jpegLosslessHint": ("JPEG files are repacked losslessly: about 20% smaller, and the original can be restored exactly.",
                      "I file JPEG vengono ricompressi senza perdita: circa il 20% più leggeri, e l'originale si può ripristinare identico."),
 "advanced": ("Advanced", "Avanzate"),
 "speedLabel": ("Encoding speed", "Velocità di codifica"),
 "speedFast": ("Fast", "Veloce"),
 "speedBalanced": ("Balanced", "Bilanciata"),
 "speedMax": ("Smallest file", "File più leggero"),
 "progressiveLabel": ("Progressive loading", "Caricamento progressivo"),
 "progressiveHint": ("A preview appears while the image downloads.", "Un'anteprima appare mentre l'immagine si scarica."),
 "maxWidthLabel": ("Maximum width", "Larghezza massima"),
 "noLimit": ("Original size", "Dimensione originale"),
 "reencodeJpeg": ("Re-encode JPEGs instead (smaller, lossy)", "Ricodifica i JPEG (più leggeri, con perdita)"),
 "metadataLabel": ("Metadata", "Metadati"),
 "metadataKeep": ("Keep all", "Mantieni tutto"),
 "metadataStripGps": ("Remove location", "Rimuovi posizione"),
 "metadataStripAll": ("Remove all", "Rimuovi tutto"),
 "statusWaiting": ("Waiting", "In attesa"),
 "statusWorking": ("Converting…", "Conversione…"),
 "statusFailed": ("Failed", "Errore"),
 "fidelityReversible": ("Lossless · original restorable", "Senza perdita · originale ripristinabile"),
 "fidelityPixelExact": ("Lossless", "Senza perdita"),
 "fidelityLossy": ("Lossy", "Con perdita"),
 "savedTo": ("Saved to {folder}", "Salvato in {folder}"),
 "showInFolder": ("Show", "Mostra"),
 "share": ("Share", "Condividi"),
 "totalSaving": ("{saved} saved ({percent})", "{saved} risparmiati ({percent})"),
 "totalGrowth": ("{size} larger ({percent})", "{size} in più ({percent})"),
 "beforeAfter": ("Before / after", "Prima / dopo"),
 "original": ("Original", "Originale"),
 "result": ("Result", "Risultato"),
 "notes": ("Notes", "Note"),

 "webIntro": ("Creates JPEG XL in every useful size, fallbacks for older browsers, and the code to paste into your site.",
              "Crea JPEG XL in tutte le dimensioni utili, le alternative per i browser datati e il codice da incollare nel tuo sito."),
 "widthsLabel": ("Sizes (px)", "Dimensioni (px)"),
 "fallbacksLabel": ("Fallback for older browsers", "Alternativa per i browser datati"),
 "heroLabel": ("Hero image (top of the page)", "Immagine principale (in cima alla pagina)"),
 "heroHint": ("Loads first, with high priority, for the best Largest Contentful Paint.",
              "Viene caricata per prima, con priorità alta, per il miglior Largest Contentful Paint."),
 "altLabel": ("Alt text (accessibility and SEO)", "Testo alternativo (accessibilità e SEO)"),
 "baseUrlLabel": ("Public URL path", "Percorso URL pubblico"),
 "nameLabel": ("File name", "Nome file"),
 "autoFocusLabel": ("Load the subject first", "Carica prima il soggetto"),
 "autoFocusHint": ("The subject sharpens before the background on slow connections.",
                   "Con connessioni lente il soggetto diventa nitido prima dello sfondo."),
 "exportButton": ("Export", "Esporta"),
 "exporting": ("Exporting…", "Esportazione…"),
 "webResultTitle": ("{count} files ready", "{count} file pronti"),
 "smallerThan": ("{percent} smaller than {format}", "{percent} più leggero di {format}"),
 "largerThan": ("{percent} larger than {format}", "{percent} più pesante di {format}"),
 "copy": ("Copy", "Copia"),
 "copied": ("Copied to clipboard", "Copiato negli appunti"),
 "webHowTo": ("Upload the folder to your server at the URL path above, then paste the code into your page.",
              "Carica la cartella sul server al percorso URL indicato, poi incolla il codice nella pagina."),

 "studioEmpty": ("Open an image to explore what JPEG XL can do with it.", "Apri un'immagine per scoprire cosa può farne JPEG XL."),
 "toolCompare": ("Compare", "Confronta"),
 "toolSubject": ("Subject", "Soggetto"),
 "toolBackground": ("Background", "Sfondo"),
 "toolCrop": ("Crop & tiles", "Ritaglio e riquadri"),
 "toolProgressive": ("Loading", "Caricamento"),
 "toolBattle": ("Format battle", "Sfida tra formati"),
 "scoreLabel": ("Quality score", "Punteggio qualità"),
 "scoreVisuallyLossless": ("visually lossless", "visivamente identica"),
 "scoreVeryHigh": ("very high", "molto alta"),
 "scoreHigh": ("high", "alta"),
 "scoreMedium": ("medium", "media"),
 "scoreLow": ("low", "bassa"),
 "scoreExplain": ("SSIMULACRA2, the metric used to tune JPEG XL: 90+ means you cannot tell the difference.",
                  "SSIMULACRA2, la metrica usata per calibrare JPEG XL: sopra 90 la differenza non si vede."),
 "saveResult": ("Save", "Salva"),
 "working": ("Working…", "Elaborazione…"),
 "subjectIntro": ("JPEG XL can load the subject first and give it more detail than the background, in one normal image file.",
                  "JPEG XL può caricare prima il soggetto e dargli più dettaglio dello sfondo, in un normale file immagine."),
 "detectSubject": ("Find subject", "Trova soggetto"),
 "boostRun": ("Create optimised file", "Crea file ottimizzato"),
 "boostStandard": ("Standard", "Standard"),
 "boostLayered": ("Subject boost", "Soggetto potenziato"),
 "boostSubjectScore": ("subject score {score}", "punteggio soggetto {score}"),
 "boostRecommended": ("Subject boost wins: sharper subject, smaller or equal file. Saved.",
                      "Vince il soggetto potenziato: soggetto più nitido, file uguale o più leggero. Salvato."),
 "boostNotWorth": ("The standard file is better for this image. Saved, with the subject loading first.",
                   "Per questa immagine è meglio il file standard. Salvato, con il soggetto caricato per primo."),
 "boostWholeImage": ("The subject fills most of the image: subject-first loading is enough.",
                     "Il soggetto riempie quasi tutta l'immagine: basta caricarlo per primo."),
 "bgTransparent": ("Transparent", "Trasparente"),
 "bgWhite": ("White", "Bianco"),
 "bgBlack": ("Black", "Nero"),
 "hardEdges": ("Hard edges", "Bordi netti"),
 "removeBackgroundButton": ("Remove background", "Rimuovi sfondo"),
 "bgUnavailable": ("Background removal is not included in this build.", "La rimozione dello sfondo non è inclusa in questa versione."),
 "cropIntro": ("Crops and tiles are pixel-exact: every pixel is identical to the original, whatever its format.",
               "Ritagli e riquadri sono identici al pixel: ogni pixel resta uguale all'originale, qualunque sia il formato."),
 "cropButton": ("Save crop", "Salva ritaglio"),
 "cropHint": ("Drag on the image to select an area.", "Trascina sull'immagine per selezionare un'area."),
 "gridLabel": ("Grid {cols} × {rows}", "Griglia {cols} × {rows}"),
 "tilesButton": ("Split into tiles", "Dividi in riquadri"),
 "deepZoomButton": ("Deep Zoom pyramid", "Piramide Deep Zoom"),
 "deepZoomHint": ("For zoomable viewers such as OpenSeadragon: maps, plans, product close-ups.",
                  "Per visualizzatori con zoom come OpenSeadragon: mappe, planimetrie, dettagli di prodotto."),
 "tilesDone": ("{count} tiles saved", "{count} riquadri salvati"),
 "deepZoomDone": ("{levels} levels, {count} tiles saved", "{levels} livelli, {count} riquadri salvati"),
 "progressiveIntro": ("Drag the slider to see what a visitor sees while the image downloads.",
                      "Trascina il cursore per vedere cosa vede un visitatore mentre l'immagine si scarica."),
 "downloaded": ("{percent} downloaded · {size}", "{percent} scaricato · {size}"),
 "nothingYet": ("Nothing to show yet", "Ancora niente da mostrare"),
 "battleIntro": ("Each format is tuned to the same visual quality. Who makes the smallest file?",
                 "Ogni formato viene regolato sulla stessa qualità visiva. Chi produce il file più leggero?"),
 "battleTarget": ("Target score {score}", "Punteggio obiettivo {score}"),
 "battleRun": ("Start", "Avvia"),
 "notScored": ("not scored", "non valutato"),
 "vsJpeg": ("{percent} vs JPEG", "{percent} rispetto a JPEG"),
 "winner": ("Winner", "Vincitore"),

 "inspectEmpty": ("Open a file to see what is inside.", "Apri un file per vedere cosa contiene."),
 "dimensions": ("Dimensions", "Dimensioni"),
 "fileSize": ("File size", "Peso del file"),
 "bitDepth": ("Bit depth", "Profondità colore"),
 "bitsPerPixel": ("Bits per pixel", "Bit per pixel"),
 "transparency": ("Transparency", "Trasparenza"),
 "colorProfile": ("Colour profile", "Profilo colore"),
 "exif": ("EXIF", "EXIF"),
 "location": ("Location (GPS)", "Posizione (GPS)"),
 "frames": ("Frames", "Fotogrammi"),
 "yes": ("Yes", "Sì"),
 "no": ("No", "No"),
 "unused": ("present but unused", "presente ma inutilizzata"),
 "embedded": ("Embedded", "Incorporato"),
 "srgbDefault": ("sRGB (default)", "sRGB (predefinito)"),
 "jxlDetails": ("JPEG XL details", "Dettagli JPEG XL"),
 "jxlMode": ("Mode", "Modalità"),
 "jxlLossless": ("Lossless / original colour space", "Senza perdita / spazio colore originale"),
 "jxlLossy": ("Lossy (XYB)", "Con perdita (XYB)"),
 "jxlContainsJpeg": ("Contains the original JPEG", "Contiene il JPEG originale"),
 "hdr": ("HDR", "HDR"),
 "restoreJpeg": ("Restore original JPEG", "Ripristina il JPEG originale"),
 "suggestions": ("Suggestions", "Suggerimenti"),
 "tipJpeg": ("Convert to JPEG XL with default settings: about 20% smaller with zero loss, and the original JPEG can be restored bit for bit.",
             "Converti in JPEG XL con le impostazioni predefinite: circa il 20% più leggero senza perdita, e il JPEG originale si può ripristinare identico."),
 "tipLosslessPng": ("Lossless JPEG XL keeps every pixel and is usually 30–50% smaller than PNG.",
                    "JPEG XL senza perdita mantiene ogni pixel ed è di solito il 30–50% più leggero di PNG."),
 "tipAnimatedGif": ("Animated JPEG XL keeps full colour (GIF is limited to 256) and is usually much smaller.",
                    "JPEG XL animato mantiene tutti i colori (GIF ne ha solo 256) ed è di solito molto più leggero."),
 "tipWebp": ("WebP is limited to 8 bits and reduced colour detail; JPEG XL at the same size keeps more detail.",
             "WebP è limitato a 8 bit e riduce il dettaglio del colore; JPEG XL a parità di peso conserva più dettagli."),
 "tipGps": ("This photo contains the GPS location where it was taken. Jixel removes it by default when exporting.",
            "Questa foto contiene la posizione GPS in cui è stata scattata. Jixel la rimuove per impostazione predefinita."),
 "tipOpaqueAlpha": ("The transparency channel is fully opaque: dropping it saves space (Jixel does this automatically).",
                    "Il canale di trasparenza è completamente opaco: rimuoverlo fa risparmiare spazio (Jixel lo fa in automatico)."),
 "tipReconstruct": ("This JPEG XL contains a recompressed JPEG: it can be turned back into the identical original.",
                    "Questo JPEG XL contiene un JPEG ricompresso: si può riconvertire nell'originale identico."),
 "tipHdr": ("This is an HDR image. Keep it as JPEG XL (or 16-bit PNG/TIFF) to preserve the highlights.",
            "È un'immagine HDR. Conservala in JPEG XL (o PNG/TIFF a 16 bit) per preservare le alte luci."),
 "tipLarge": ("At {width}×{height} this image is larger than most screens: Optimize for web creates right-sized versions.",
              "Con {width}×{height} pixel questa immagine è più grande della maggior parte degli schermi: Ottimizza per il web crea le versioni giuste."),

 "appearance": ("Appearance", "Aspetto"),
 "themeSystem": ("System", "Sistema"),
 "themeLight": ("Light", "Chiaro"),
 "themeDark": ("Dark", "Scuro"),
 "language": ("Language", "Lingua"),
 "languageSystem": ("System", "Sistema"),
 "outputFolder": ("Output folder", "Cartella di destinazione"),
 "change": ("Change", "Cambia"),
 "defaults": ("Defaults", "Predefiniti"),
 "about": ("About", "Informazioni"),
 "aboutBody": ("Jixel is open source (Apache-2.0). Your images never leave your device.",
               "Jixel è open source (Apache-2.0). Le tue immagini non lasciano mai il dispositivo."),
 "engineVersion": ("Jixel {version} · libjxl {libjxl}", "Jixel {version} · libjxl {libjxl}"),
 "licenses": ("Open-source licences", "Licenze open source"),
 "errorGeneric": ("Something went wrong: {message}", "Qualcosa è andato storto: {message}"),
 "unsupportedFiles": ("{count, plural, =1{1 file skipped: not a supported image} other{{count} files skipped: not supported images}}",
                      "{count, plural, =1{1 file ignorato: formato non supportato} other{{count} file ignorati: formato non supportato}}"),
}

DESCRIPTIONS = {
 "homeHeadline": "Big headline on the home screen.",
 "convertCount": "Primary button of the Convert screen.",
 "tipJpeg": "Inspect suggestion for JPEG files.",
}

def placeholders(text):
    names = []
    for m in re.finditer(r"\{(\w+)(,|\})", text):
        if m.group(1) not in names:
            names.append(m.group(1))
    return names

def arb(lang_index, locale):
    out = {"@@locale": locale}
    for key, texts in S.items():
        text = texts[lang_index]
        out[key] = text
        if lang_index == 0:
            meta = {}
            if key in DESCRIPTIONS:
                meta["description"] = DESCRIPTIONS[key]
            ph = placeholders(text)
            if ph:
                # Plural counts must be ints; everything else is preformatted text.
                meta["placeholders"] = {
                    p: {"type": "int" if p == "count" and "plural" in text else "Object"} for p in ph
                }
            if meta:
                out["@" + key] = meta
    return out

here = os.path.dirname(os.path.abspath(__file__))
for i, loc in enumerate(["en", "it"]):
    path = os.path.join(here, "..", "lib", "l10n", f"app_{loc}.arb")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(arb(i, loc), f, ensure_ascii=False, indent=2)
        f.write("\n")
print(f"{len(S)} strings written")
