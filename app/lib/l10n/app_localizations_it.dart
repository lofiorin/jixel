// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Italian (`it`).
class AppLocalizationsIt extends AppLocalizations {
  AppLocalizationsIt([String locale = 'it']) : super(locale);

  @override
  String get appTitle => 'Jixel';

  @override
  String get navHome => 'Home';

  @override
  String get navConvert => 'Converti';

  @override
  String get navWeb => 'Web';

  @override
  String get navStudio => 'Studio';

  @override
  String get navInspect => 'Ispeziona';

  @override
  String get navSettings => 'Impostazioni';

  @override
  String get homeHeadline => 'Immagini più leggere. Stessa qualità.';

  @override
  String get homeSubtitle =>
      'Converti, ottimizza e analizza immagini con JPEG XL. Tutto avviene sul tuo dispositivo: nessun file viene caricato online.';

  @override
  String get dropTitle => 'Trascina qui le immagini';

  @override
  String get dropOr => 'oppure';

  @override
  String get chooseFiles => 'Scegli file';

  @override
  String get chooseImage => 'Scegli immagine';

  @override
  String get changeImage => 'Cambia immagine';

  @override
  String get supportedFormats => 'Legge JPEG XL, JPEG, PNG, WebP, GIF, TIFF, BMP e altri.';

  @override
  String get whatToDo => 'Cosa vuoi fare?';

  @override
  String get taskConvertTitle => 'Converti';

  @override
  String get taskConvertBody =>
      'Da qualsiasi formato a JPEG XL e ritorno. I JPEG si riducono di circa il 20% senza alcuna perdita.';

  @override
  String get taskWebTitle => 'Ottimizza per il web';

  @override
  String get taskWebBody =>
      'Tutte le dimensioni per ogni schermo, alternative per i browser datati e codice per HTML, Next.js e WordPress.';

  @override
  String get taskBackgroundTitle => 'Rimuovi sfondo';

  @override
  String get taskBackgroundBody =>
      'IA sul dispositivo. Sfondo trasparente o a tinta unita, pronto per il tuo negozio online.';

  @override
  String get taskStudioTitle => 'Studio';

  @override
  String get taskStudioBody =>
      'Confronta la qualità, valorizza il soggetto, ritaglia, dividi in riquadri, simula il caricamento.';

  @override
  String get addFiles => 'Aggiungi file';

  @override
  String get clearList => 'Svuota';

  @override
  String convertCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Converti $count immagini',
      one: 'Converti 1 immagine',
    );
    return '$_temp0';
  }

  @override
  String get emptyQueue => 'Aggiungi immagini per iniziare';

  @override
  String get formatLabel => 'Formato';

  @override
  String get qualityLabel => 'Qualità';

  @override
  String get qualityLossless => 'Senza perdita';

  @override
  String get qualityVisuallyLossless => 'Visivamente identica';

  @override
  String get qualityHigh => 'Alta';

  @override
  String get qualityBalanced => 'Bilanciata';

  @override
  String get qualitySmall => 'Più leggera';

  @override
  String get jpegLosslessHint =>
      'I file JPEG vengono ricompressi senza perdita: circa il 20% più leggeri, e l\'originale si può ripristinare identico.';

  @override
  String get advanced => 'Avanzate';

  @override
  String get speedLabel => 'Velocità di codifica';

  @override
  String get speedFast => 'Veloce';

  @override
  String get speedBalanced => 'Bilanciata';

  @override
  String get speedMax => 'File più leggero';

  @override
  String get progressiveLabel => 'Caricamento progressivo';

  @override
  String get progressiveHint => 'Un\'anteprima appare mentre l\'immagine si scarica.';

  @override
  String get maxWidthLabel => 'Larghezza massima';

  @override
  String get noLimit => 'Dimensione originale';

  @override
  String get reencodeJpeg => 'Ricodifica i JPEG (più leggeri, con perdita)';

  @override
  String get metadataLabel => 'Metadati';

  @override
  String get metadataKeep => 'Mantieni tutto';

  @override
  String get metadataStripGps => 'Rimuovi posizione';

  @override
  String get metadataStripAll => 'Rimuovi tutto';

  @override
  String get statusWaiting => 'In attesa';

  @override
  String get statusWorking => 'Conversione…';

  @override
  String get statusFailed => 'Errore';

  @override
  String get fidelityReversible => 'Senza perdita · originale ripristinabile';

  @override
  String get fidelityPixelExact => 'Senza perdita';

  @override
  String get fidelityLossy => 'Con perdita';

  @override
  String savedTo(Object folder) {
    return 'Salvato in $folder';
  }

  @override
  String get showInFolder => 'Mostra';

  @override
  String get share => 'Condividi';

  @override
  String totalSaving(Object saved, Object percent) {
    return '$saved risparmiati ($percent)';
  }

  @override
  String totalGrowth(Object size, Object percent) {
    return '$size in più ($percent)';
  }

  @override
  String get beforeAfter => 'Prima / dopo';

  @override
  String get original => 'Originale';

  @override
  String get result => 'Risultato';

  @override
  String get notes => 'Note';

  @override
  String get webIntro =>
      'Crea JPEG XL in tutte le dimensioni utili, le alternative per i browser datati e il codice da incollare nel tuo sito.';

  @override
  String get widthsLabel => 'Dimensioni (px)';

  @override
  String get fallbacksLabel => 'Alternativa per i browser datati';

  @override
  String get heroLabel => 'Immagine principale (in cima alla pagina)';

  @override
  String get heroHint => 'Viene caricata per prima, con priorità alta, per il miglior Largest Contentful Paint.';

  @override
  String get altLabel => 'Testo alternativo (accessibilità e SEO)';

  @override
  String get baseUrlLabel => 'Percorso URL pubblico';

  @override
  String get nameLabel => 'Nome file';

  @override
  String get autoFocusLabel => 'Carica prima il soggetto';

  @override
  String get autoFocusHint => 'Con connessioni lente il soggetto diventa nitido prima dello sfondo.';

  @override
  String get exportButton => 'Esporta';

  @override
  String get exporting => 'Esportazione…';

  @override
  String webResultTitle(Object count) {
    return '$count file pronti';
  }

  @override
  String smallerThan(Object percent, Object format) {
    return '$percent più leggero di $format';
  }

  @override
  String largerThan(Object percent, Object format) {
    return '$percent più pesante di $format';
  }

  @override
  String get copy => 'Copia';

  @override
  String get copied => 'Copiato negli appunti';

  @override
  String get webHowTo => 'Carica la cartella sul server al percorso URL indicato, poi incolla il codice nella pagina.';

  @override
  String get studioEmpty => 'Apri un\'immagine per scoprire cosa può farne JPEG XL.';

  @override
  String get toolCompare => 'Confronta';

  @override
  String get toolSubject => 'Soggetto';

  @override
  String get toolBackground => 'Sfondo';

  @override
  String get toolCrop => 'Ritaglio e riquadri';

  @override
  String get toolProgressive => 'Caricamento';

  @override
  String get toolBattle => 'Sfida tra formati';

  @override
  String get scoreLabel => 'Punteggio qualità';

  @override
  String get scoreVisuallyLossless => 'visivamente identica';

  @override
  String get scoreVeryHigh => 'molto alta';

  @override
  String get scoreHigh => 'alta';

  @override
  String get scoreMedium => 'media';

  @override
  String get scoreLow => 'bassa';

  @override
  String get scoreExplain => 'SSIMULACRA2, la metrica usata per calibrare JPEG XL: sopra 90 la differenza non si vede.';

  @override
  String get saveResult => 'Salva';

  @override
  String get working => 'Elaborazione…';

  @override
  String get subjectIntro =>
      'JPEG XL può caricare prima il soggetto e dargli più dettaglio dello sfondo, in un normale file immagine.';

  @override
  String get detectSubject => 'Trova soggetto';

  @override
  String get boostRun => 'Crea file ottimizzato';

  @override
  String get boostStandard => 'Standard';

  @override
  String get boostLayered => 'Soggetto potenziato';

  @override
  String boostSubjectScore(Object score) {
    return 'punteggio soggetto $score';
  }

  @override
  String get boostRecommended =>
      'Vince il soggetto potenziato: soggetto più nitido, file uguale o più leggero. Salvato.';

  @override
  String get boostNotWorth =>
      'Per questa immagine è meglio il file standard. Salvato, con il soggetto caricato per primo.';

  @override
  String get boostWholeImage => 'Il soggetto riempie quasi tutta l\'immagine: basta caricarlo per primo.';

  @override
  String get bgTransparent => 'Trasparente';

  @override
  String get bgWhite => 'Bianco';

  @override
  String get bgBlack => 'Nero';

  @override
  String get hardEdges => 'Bordi netti';

  @override
  String get removeBackgroundButton => 'Rimuovi sfondo';

  @override
  String get bgUnavailable => 'La rimozione dello sfondo non è inclusa in questa versione.';

  @override
  String get cropIntro =>
      'Ritagli e riquadri sono identici al pixel: ogni pixel resta uguale all\'originale, qualunque sia il formato.';

  @override
  String get cropButton => 'Salva ritaglio';

  @override
  String get cropHint => 'Trascina sull\'immagine per selezionare un\'area.';

  @override
  String gridLabel(Object cols, Object rows) {
    return 'Griglia $cols × $rows';
  }

  @override
  String get tilesButton => 'Dividi in riquadri';

  @override
  String get deepZoomButton => 'Piramide Deep Zoom';

  @override
  String get deepZoomHint =>
      'Per visualizzatori con zoom come OpenSeadragon: mappe, planimetrie, dettagli di prodotto.';

  @override
  String tilesDone(Object count) {
    return '$count riquadri salvati';
  }

  @override
  String deepZoomDone(Object levels, Object count) {
    return '$levels livelli, $count riquadri salvati';
  }

  @override
  String get progressiveIntro =>
      'Trascina il cursore per vedere cosa vede un visitatore mentre l\'immagine si scarica.';

  @override
  String downloaded(Object percent, Object size) {
    return '$percent scaricato · $size';
  }

  @override
  String get nothingYet => 'Ancora niente da mostrare';

  @override
  String get battleIntro => 'Ogni formato viene regolato sulla stessa qualità visiva. Chi produce il file più leggero?';

  @override
  String battleTarget(Object score) {
    return 'Punteggio obiettivo $score';
  }

  @override
  String get battleRun => 'Avvia';

  @override
  String get notScored => 'non valutato';

  @override
  String vsJpeg(Object percent) {
    return '$percent rispetto a JPEG';
  }

  @override
  String get winner => 'Vincitore';

  @override
  String get inspectEmpty => 'Apri un file per vedere cosa contiene.';

  @override
  String get dimensions => 'Dimensioni';

  @override
  String get fileSize => 'Peso del file';

  @override
  String get bitDepth => 'Profondità colore';

  @override
  String get bitsPerPixel => 'Bit per pixel';

  @override
  String get transparency => 'Trasparenza';

  @override
  String get colorProfile => 'Profilo colore';

  @override
  String get exif => 'EXIF';

  @override
  String get location => 'Posizione (GPS)';

  @override
  String get frames => 'Fotogrammi';

  @override
  String get yes => 'Sì';

  @override
  String get no => 'No';

  @override
  String get unused => 'presente ma inutilizzata';

  @override
  String get embedded => 'Incorporato';

  @override
  String get srgbDefault => 'sRGB (predefinito)';

  @override
  String get jxlDetails => 'Dettagli JPEG XL';

  @override
  String get jxlMode => 'Modalità';

  @override
  String get jxlLossless => 'Senza perdita / spazio colore originale';

  @override
  String get jxlLossy => 'Con perdita (XYB)';

  @override
  String get jxlContainsJpeg => 'Contiene il JPEG originale';

  @override
  String get hdr => 'HDR';

  @override
  String get restoreJpeg => 'Ripristina il JPEG originale';

  @override
  String get suggestions => 'Suggerimenti';

  @override
  String get tipJpeg =>
      'Converti in JPEG XL con le impostazioni predefinite: circa il 20% più leggero senza perdita, e il JPEG originale si può ripristinare identico.';

  @override
  String get tipLosslessPng => 'JPEG XL senza perdita mantiene ogni pixel ed è di solito il 30–50% più leggero di PNG.';

  @override
  String get tipAnimatedGif =>
      'JPEG XL animato mantiene tutti i colori (GIF ne ha solo 256) ed è di solito molto più leggero.';

  @override
  String get tipWebp =>
      'WebP è limitato a 8 bit e riduce il dettaglio del colore; JPEG XL a parità di peso conserva più dettagli.';

  @override
  String get tipGps =>
      'Questa foto contiene la posizione GPS in cui è stata scattata. Jixel la rimuove per impostazione predefinita.';

  @override
  String get tipOpaqueAlpha =>
      'Il canale di trasparenza è completamente opaco: rimuoverlo fa risparmiare spazio (Jixel lo fa in automatico).';

  @override
  String get tipReconstruct =>
      'Questo JPEG XL contiene un JPEG ricompresso: si può riconvertire nell\'originale identico.';

  @override
  String get tipHdr => 'È un\'immagine HDR. Conservala in JPEG XL (o PNG/TIFF a 16 bit) per preservare le alte luci.';

  @override
  String tipLarge(Object width, Object height) {
    return 'Con $width×$height pixel questa immagine è più grande della maggior parte degli schermi: Ottimizza per il web crea le versioni giuste.';
  }

  @override
  String get appearance => 'Aspetto';

  @override
  String get themeSystem => 'Sistema';

  @override
  String get themeLight => 'Chiaro';

  @override
  String get themeDark => 'Scuro';

  @override
  String get language => 'Lingua';

  @override
  String get languageSystem => 'Sistema';

  @override
  String get outputFolder => 'Cartella di destinazione';

  @override
  String get change => 'Cambia';

  @override
  String get defaults => 'Predefiniti';

  @override
  String get about => 'Informazioni';

  @override
  String get aboutBody => 'Jixel è open source (Apache-2.0). Le tue immagini non lasciano mai il dispositivo.';

  @override
  String engineVersion(Object version, Object libjxl) {
    return 'Jixel $version · libjxl $libjxl';
  }

  @override
  String get licenses => 'Licenze open source';

  @override
  String errorGeneric(Object message) {
    return 'Qualcosa è andato storto: $message';
  }

  @override
  String unsupportedFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count file ignorati: formato non supportato',
      one: '1 file ignorato: formato non supportato',
    );
    return '$_temp0';
  }
}
