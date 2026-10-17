// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'Jixel';

  @override
  String get navHome => 'Home';

  @override
  String get navConvert => 'Convert';

  @override
  String get navWeb => 'Web';

  @override
  String get navStudio => 'Studio';

  @override
  String get navInspect => 'Inspect';

  @override
  String get navSettings => 'Settings';

  @override
  String get homeHeadline => 'Smaller images. Same quality.';

  @override
  String get homeSubtitle =>
      'Convert, optimise and inspect images with JPEG XL. Everything happens on your device: nothing is uploaded.';

  @override
  String get dropTitle => 'Drop images here';

  @override
  String get dropOr => 'or';

  @override
  String get chooseFiles => 'Choose files';

  @override
  String get chooseImage => 'Choose image';

  @override
  String get changeImage => 'Change image';

  @override
  String get supportedFormats => 'Reads JPEG XL, JPEG, PNG, WebP, GIF, TIFF, BMP and more.';

  @override
  String get whatToDo => 'What do you want to do?';

  @override
  String get taskConvertTitle => 'Convert';

  @override
  String get taskConvertBody => 'Any format to JPEG XL and back. JPEGs shrink by about 20% with zero loss.';

  @override
  String get taskWebTitle => 'Optimize for web';

  @override
  String get taskWebBody => 'Every screen size, fallbacks for old browsers, and code for HTML, Next.js and WordPress.';

  @override
  String get taskBackgroundTitle => 'Remove background';

  @override
  String get taskBackgroundBody => 'On-device AI. Transparent or solid colour, ready for your shop.';

  @override
  String get taskStudioTitle => 'Studio';

  @override
  String get taskStudioBody => 'Compare quality, sharpen the subject, crop, split into tiles, preview loading.';

  @override
  String get addFiles => 'Add files';

  @override
  String get clearList => 'Clear';

  @override
  String convertCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Convert $count images',
      one: 'Convert 1 image',
    );
    return '$_temp0';
  }

  @override
  String get emptyQueue => 'Add images to start';

  @override
  String get formatLabel => 'Format';

  @override
  String get qualityLabel => 'Quality';

  @override
  String get qualityLossless => 'Lossless';

  @override
  String get qualityVisuallyLossless => 'Visually lossless';

  @override
  String get qualityHigh => 'High';

  @override
  String get qualityBalanced => 'Balanced';

  @override
  String get qualitySmall => 'Smallest';

  @override
  String get jpegLosslessHint =>
      'JPEG files are repacked losslessly: about 20% smaller, and the original can be restored exactly.';

  @override
  String get advanced => 'Advanced';

  @override
  String get speedLabel => 'Encoding speed';

  @override
  String get speedFast => 'Fast';

  @override
  String get speedBalanced => 'Balanced';

  @override
  String get speedMax => 'Smallest file';

  @override
  String get progressiveLabel => 'Progressive loading';

  @override
  String get progressiveHint => 'A preview appears while the image downloads.';

  @override
  String get maxWidthLabel => 'Maximum width';

  @override
  String get noLimit => 'Original size';

  @override
  String get reencodeJpeg => 'Re-encode JPEGs instead (smaller, lossy)';

  @override
  String get metadataLabel => 'Metadata';

  @override
  String get metadataKeep => 'Keep all';

  @override
  String get metadataStripGps => 'Remove location';

  @override
  String get metadataStripAll => 'Remove all';

  @override
  String get statusWaiting => 'Waiting';

  @override
  String get statusWorking => 'Converting…';

  @override
  String get statusFailed => 'Failed';

  @override
  String get fidelityReversible => 'Lossless · original restorable';

  @override
  String get fidelityPixelExact => 'Lossless';

  @override
  String get fidelityLossy => 'Lossy';

  @override
  String savedTo(Object folder) {
    return 'Saved to $folder';
  }

  @override
  String get showInFolder => 'Show';

  @override
  String get share => 'Share';

  @override
  String totalSaving(Object saved, Object percent) {
    return '$saved saved ($percent)';
  }

  @override
  String totalGrowth(Object size, Object percent) {
    return '$size larger ($percent)';
  }

  @override
  String get beforeAfter => 'Before / after';

  @override
  String get original => 'Original';

  @override
  String get result => 'Result';

  @override
  String get notes => 'Notes';

  @override
  String get webIntro =>
      'Creates JPEG XL in every useful size, fallbacks for older browsers, and the code to paste into your site.';

  @override
  String get widthsLabel => 'Sizes (px)';

  @override
  String get fallbacksLabel => 'Fallback for older browsers';

  @override
  String get heroLabel => 'Hero image (top of the page)';

  @override
  String get heroHint => 'Loads first, with high priority, for the best Largest Contentful Paint.';

  @override
  String get altLabel => 'Alt text (accessibility and SEO)';

  @override
  String get baseUrlLabel => 'Public URL path';

  @override
  String get nameLabel => 'File name';

  @override
  String get autoFocusLabel => 'Load the subject first';

  @override
  String get autoFocusHint => 'The subject sharpens before the background on slow connections.';

  @override
  String get exportButton => 'Export';

  @override
  String get exporting => 'Exporting…';

  @override
  String webResultTitle(Object count) {
    return '$count files ready';
  }

  @override
  String smallerThan(Object percent, Object format) {
    return '$percent smaller than $format';
  }

  @override
  String largerThan(Object percent, Object format) {
    return '$percent larger than $format';
  }

  @override
  String get copy => 'Copy';

  @override
  String get copied => 'Copied to clipboard';

  @override
  String get webHowTo => 'Upload the folder to your server at the URL path above, then paste the code into your page.';

  @override
  String get studioEmpty => 'Open an image to explore what JPEG XL can do with it.';

  @override
  String get toolCompare => 'Compare';

  @override
  String get toolSubject => 'Subject';

  @override
  String get toolBackground => 'Background';

  @override
  String get toolCrop => 'Crop & tiles';

  @override
  String get toolProgressive => 'Loading';

  @override
  String get toolBattle => 'Format battle';

  @override
  String get scoreLabel => 'Quality score';

  @override
  String get scoreVisuallyLossless => 'visually lossless';

  @override
  String get scoreVeryHigh => 'very high';

  @override
  String get scoreHigh => 'high';

  @override
  String get scoreMedium => 'medium';

  @override
  String get scoreLow => 'low';

  @override
  String get scoreExplain => 'SSIMULACRA2, the metric used to tune JPEG XL: 90+ means you cannot tell the difference.';

  @override
  String get saveResult => 'Save';

  @override
  String get working => 'Working…';

  @override
  String get subjectIntro =>
      'JPEG XL can load the subject first and give it more detail than the background, in one normal image file.';

  @override
  String get detectSubject => 'Find subject';

  @override
  String get boostRun => 'Create optimised file';

  @override
  String get boostStandard => 'Standard';

  @override
  String get boostLayered => 'Subject boost';

  @override
  String boostSubjectScore(Object score) {
    return 'subject score $score';
  }

  @override
  String get boostRecommended => 'Subject boost wins: sharper subject, smaller or equal file. Saved.';

  @override
  String get boostNotWorth => 'The standard file is better for this image. Saved, with the subject loading first.';

  @override
  String get boostWholeImage => 'The subject fills most of the image: subject-first loading is enough.';

  @override
  String get bgTransparent => 'Transparent';

  @override
  String get bgWhite => 'White';

  @override
  String get bgBlack => 'Black';

  @override
  String get hardEdges => 'Hard edges';

  @override
  String get removeBackgroundButton => 'Remove background';

  @override
  String get bgUnavailable => 'Background removal is not included in this build.';

  @override
  String get cropIntro =>
      'Crops and tiles are pixel-exact: every pixel is identical to the original, whatever its format.';

  @override
  String get cropButton => 'Save crop';

  @override
  String get cropHint => 'Drag on the image to select an area.';

  @override
  String gridLabel(Object cols, Object rows) {
    return 'Grid $cols × $rows';
  }

  @override
  String get tilesButton => 'Split into tiles';

  @override
  String get deepZoomButton => 'Deep Zoom pyramid';

  @override
  String get deepZoomHint => 'For zoomable viewers such as OpenSeadragon: maps, plans, product close-ups.';

  @override
  String tilesDone(Object count) {
    return '$count tiles saved';
  }

  @override
  String deepZoomDone(Object levels, Object count) {
    return '$levels levels, $count tiles saved';
  }

  @override
  String get progressiveIntro => 'Drag the slider to see what a visitor sees while the image downloads.';

  @override
  String downloaded(Object percent, Object size) {
    return '$percent downloaded · $size';
  }

  @override
  String get nothingYet => 'Nothing to show yet';

  @override
  String get battleIntro => 'Each format is tuned to the same visual quality. Who makes the smallest file?';

  @override
  String battleTarget(Object score) {
    return 'Target score $score';
  }

  @override
  String get battleRun => 'Start';

  @override
  String get notScored => 'not scored';

  @override
  String vsJpeg(Object percent) {
    return '$percent vs JPEG';
  }

  @override
  String get winner => 'Winner';

  @override
  String get inspectEmpty => 'Open a file to see what is inside.';

  @override
  String get dimensions => 'Dimensions';

  @override
  String get fileSize => 'File size';

  @override
  String get bitDepth => 'Bit depth';

  @override
  String get bitsPerPixel => 'Bits per pixel';

  @override
  String get transparency => 'Transparency';

  @override
  String get colorProfile => 'Colour profile';

  @override
  String get exif => 'EXIF';

  @override
  String get location => 'Location (GPS)';

  @override
  String get frames => 'Frames';

  @override
  String get yes => 'Yes';

  @override
  String get no => 'No';

  @override
  String get unused => 'present but unused';

  @override
  String get embedded => 'Embedded';

  @override
  String get srgbDefault => 'sRGB (default)';

  @override
  String get jxlDetails => 'JPEG XL details';

  @override
  String get jxlMode => 'Mode';

  @override
  String get jxlLossless => 'Lossless / original colour space';

  @override
  String get jxlLossy => 'Lossy (XYB)';

  @override
  String get jxlContainsJpeg => 'Contains the original JPEG';

  @override
  String get hdr => 'HDR';

  @override
  String get restoreJpeg => 'Restore original JPEG';

  @override
  String get suggestions => 'Suggestions';

  @override
  String get tipJpeg =>
      'Convert to JPEG XL with default settings: about 20% smaller with zero loss, and the original JPEG can be restored bit for bit.';

  @override
  String get tipLosslessPng => 'Lossless JPEG XL keeps every pixel and is usually 30–50% smaller than PNG.';

  @override
  String get tipAnimatedGif =>
      'Animated JPEG XL keeps full colour (GIF is limited to 256) and is usually much smaller.';

  @override
  String get tipWebp =>
      'WebP is limited to 8 bits and reduced colour detail; JPEG XL at the same size keeps more detail.';

  @override
  String get tipGps =>
      'This photo contains the GPS location where it was taken. Jixel removes it by default when exporting.';

  @override
  String get tipOpaqueAlpha =>
      'The transparency channel is fully opaque: dropping it saves space (Jixel does this automatically).';

  @override
  String get tipReconstruct =>
      'This JPEG XL contains a recompressed JPEG: it can be turned back into the identical original.';

  @override
  String get tipHdr => 'This is an HDR image. Keep it as JPEG XL (or 16-bit PNG/TIFF) to preserve the highlights.';

  @override
  String tipLarge(Object width, Object height) {
    return 'At $width×$height this image is larger than most screens: Optimize for web creates right-sized versions.';
  }

  @override
  String get appearance => 'Appearance';

  @override
  String get themeSystem => 'System';

  @override
  String get themeLight => 'Light';

  @override
  String get themeDark => 'Dark';

  @override
  String get language => 'Language';

  @override
  String get languageSystem => 'System';

  @override
  String get outputFolder => 'Output folder';

  @override
  String get change => 'Change';

  @override
  String get defaults => 'Defaults';

  @override
  String get about => 'About';

  @override
  String get aboutBody => 'Jixel is open source (Apache-2.0). Your images never leave your device.';

  @override
  String engineVersion(Object version, Object libjxl) {
    return 'Jixel $version · libjxl $libjxl';
  }

  @override
  String get licenses => 'Open-source licences';

  @override
  String errorGeneric(Object message) {
    return 'Something went wrong: $message';
  }

  @override
  String unsupportedFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count files skipped: not supported images',
      one: '1 file skipped: not a supported image',
    );
    return '$_temp0';
  }
}
