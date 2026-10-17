import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_it.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale) : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate = _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates = <LocalizationsDelegate<dynamic>>[
    delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
  ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[Locale('en'), Locale('it')];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'Jixel'**
  String get appTitle;

  /// No description provided for @navHome.
  ///
  /// In en, this message translates to:
  /// **'Home'**
  String get navHome;

  /// No description provided for @navConvert.
  ///
  /// In en, this message translates to:
  /// **'Convert'**
  String get navConvert;

  /// No description provided for @navWeb.
  ///
  /// In en, this message translates to:
  /// **'Web'**
  String get navWeb;

  /// No description provided for @navStudio.
  ///
  /// In en, this message translates to:
  /// **'Studio'**
  String get navStudio;

  /// No description provided for @navInspect.
  ///
  /// In en, this message translates to:
  /// **'Inspect'**
  String get navInspect;

  /// No description provided for @navSettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get navSettings;

  /// Big headline on the home screen.
  ///
  /// In en, this message translates to:
  /// **'Smaller images. Same quality.'**
  String get homeHeadline;

  /// No description provided for @homeSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Convert, optimise and inspect images with JPEG XL. Everything happens on your device: nothing is uploaded.'**
  String get homeSubtitle;

  /// No description provided for @dropTitle.
  ///
  /// In en, this message translates to:
  /// **'Drop images here'**
  String get dropTitle;

  /// No description provided for @dropOr.
  ///
  /// In en, this message translates to:
  /// **'or'**
  String get dropOr;

  /// No description provided for @chooseFiles.
  ///
  /// In en, this message translates to:
  /// **'Choose files'**
  String get chooseFiles;

  /// No description provided for @chooseImage.
  ///
  /// In en, this message translates to:
  /// **'Choose image'**
  String get chooseImage;

  /// No description provided for @changeImage.
  ///
  /// In en, this message translates to:
  /// **'Change image'**
  String get changeImage;

  /// No description provided for @supportedFormats.
  ///
  /// In en, this message translates to:
  /// **'Reads JPEG XL, JPEG, PNG, WebP, GIF, TIFF, BMP and more.'**
  String get supportedFormats;

  /// No description provided for @whatToDo.
  ///
  /// In en, this message translates to:
  /// **'What do you want to do?'**
  String get whatToDo;

  /// No description provided for @taskConvertTitle.
  ///
  /// In en, this message translates to:
  /// **'Convert'**
  String get taskConvertTitle;

  /// No description provided for @taskConvertBody.
  ///
  /// In en, this message translates to:
  /// **'Any format to JPEG XL and back. JPEGs shrink by about 20% with zero loss.'**
  String get taskConvertBody;

  /// No description provided for @taskWebTitle.
  ///
  /// In en, this message translates to:
  /// **'Optimize for web'**
  String get taskWebTitle;

  /// No description provided for @taskWebBody.
  ///
  /// In en, this message translates to:
  /// **'Every screen size, fallbacks for old browsers, and code for HTML, Next.js and WordPress.'**
  String get taskWebBody;

  /// No description provided for @taskBackgroundTitle.
  ///
  /// In en, this message translates to:
  /// **'Remove background'**
  String get taskBackgroundTitle;

  /// No description provided for @taskBackgroundBody.
  ///
  /// In en, this message translates to:
  /// **'On-device AI. Transparent or solid colour, ready for your shop.'**
  String get taskBackgroundBody;

  /// No description provided for @taskStudioTitle.
  ///
  /// In en, this message translates to:
  /// **'Studio'**
  String get taskStudioTitle;

  /// No description provided for @taskStudioBody.
  ///
  /// In en, this message translates to:
  /// **'Compare quality, sharpen the subject, crop, split into tiles, preview loading.'**
  String get taskStudioBody;

  /// No description provided for @addFiles.
  ///
  /// In en, this message translates to:
  /// **'Add files'**
  String get addFiles;

  /// No description provided for @clearList.
  ///
  /// In en, this message translates to:
  /// **'Clear'**
  String get clearList;

  /// Primary button of the Convert screen.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{Convert 1 image} other{Convert {count} images}}'**
  String convertCount(int count);

  /// No description provided for @emptyQueue.
  ///
  /// In en, this message translates to:
  /// **'Add images to start'**
  String get emptyQueue;

  /// No description provided for @formatLabel.
  ///
  /// In en, this message translates to:
  /// **'Format'**
  String get formatLabel;

  /// No description provided for @qualityLabel.
  ///
  /// In en, this message translates to:
  /// **'Quality'**
  String get qualityLabel;

  /// No description provided for @qualityLossless.
  ///
  /// In en, this message translates to:
  /// **'Lossless'**
  String get qualityLossless;

  /// No description provided for @qualityVisuallyLossless.
  ///
  /// In en, this message translates to:
  /// **'Visually lossless'**
  String get qualityVisuallyLossless;

  /// No description provided for @qualityHigh.
  ///
  /// In en, this message translates to:
  /// **'High'**
  String get qualityHigh;

  /// No description provided for @qualityBalanced.
  ///
  /// In en, this message translates to:
  /// **'Balanced'**
  String get qualityBalanced;

  /// No description provided for @qualitySmall.
  ///
  /// In en, this message translates to:
  /// **'Smallest'**
  String get qualitySmall;

  /// No description provided for @jpegLosslessHint.
  ///
  /// In en, this message translates to:
  /// **'JPEG files are repacked losslessly: about 20% smaller, and the original can be restored exactly.'**
  String get jpegLosslessHint;

  /// No description provided for @advanced.
  ///
  /// In en, this message translates to:
  /// **'Advanced'**
  String get advanced;

  /// No description provided for @speedLabel.
  ///
  /// In en, this message translates to:
  /// **'Encoding speed'**
  String get speedLabel;

  /// No description provided for @speedFast.
  ///
  /// In en, this message translates to:
  /// **'Fast'**
  String get speedFast;

  /// No description provided for @speedBalanced.
  ///
  /// In en, this message translates to:
  /// **'Balanced'**
  String get speedBalanced;

  /// No description provided for @speedMax.
  ///
  /// In en, this message translates to:
  /// **'Smallest file'**
  String get speedMax;

  /// No description provided for @progressiveLabel.
  ///
  /// In en, this message translates to:
  /// **'Progressive loading'**
  String get progressiveLabel;

  /// No description provided for @progressiveHint.
  ///
  /// In en, this message translates to:
  /// **'A preview appears while the image downloads.'**
  String get progressiveHint;

  /// No description provided for @maxWidthLabel.
  ///
  /// In en, this message translates to:
  /// **'Maximum width'**
  String get maxWidthLabel;

  /// No description provided for @noLimit.
  ///
  /// In en, this message translates to:
  /// **'Original size'**
  String get noLimit;

  /// No description provided for @reencodeJpeg.
  ///
  /// In en, this message translates to:
  /// **'Re-encode JPEGs instead (smaller, lossy)'**
  String get reencodeJpeg;

  /// No description provided for @metadataLabel.
  ///
  /// In en, this message translates to:
  /// **'Metadata'**
  String get metadataLabel;

  /// No description provided for @metadataKeep.
  ///
  /// In en, this message translates to:
  /// **'Keep all'**
  String get metadataKeep;

  /// No description provided for @metadataStripGps.
  ///
  /// In en, this message translates to:
  /// **'Remove location'**
  String get metadataStripGps;

  /// No description provided for @metadataStripAll.
  ///
  /// In en, this message translates to:
  /// **'Remove all'**
  String get metadataStripAll;

  /// No description provided for @statusWaiting.
  ///
  /// In en, this message translates to:
  /// **'Waiting'**
  String get statusWaiting;

  /// No description provided for @statusWorking.
  ///
  /// In en, this message translates to:
  /// **'Converting…'**
  String get statusWorking;

  /// No description provided for @statusFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed'**
  String get statusFailed;

  /// No description provided for @fidelityReversible.
  ///
  /// In en, this message translates to:
  /// **'Lossless · original restorable'**
  String get fidelityReversible;

  /// No description provided for @fidelityPixelExact.
  ///
  /// In en, this message translates to:
  /// **'Lossless'**
  String get fidelityPixelExact;

  /// No description provided for @fidelityLossy.
  ///
  /// In en, this message translates to:
  /// **'Lossy'**
  String get fidelityLossy;

  /// No description provided for @savedTo.
  ///
  /// In en, this message translates to:
  /// **'Saved to {folder}'**
  String savedTo(Object folder);

  /// No description provided for @showInFolder.
  ///
  /// In en, this message translates to:
  /// **'Show'**
  String get showInFolder;

  /// No description provided for @share.
  ///
  /// In en, this message translates to:
  /// **'Share'**
  String get share;

  /// No description provided for @totalSaving.
  ///
  /// In en, this message translates to:
  /// **'{saved} saved ({percent})'**
  String totalSaving(Object saved, Object percent);

  /// No description provided for @totalGrowth.
  ///
  /// In en, this message translates to:
  /// **'{size} larger ({percent})'**
  String totalGrowth(Object size, Object percent);

  /// No description provided for @beforeAfter.
  ///
  /// In en, this message translates to:
  /// **'Before / after'**
  String get beforeAfter;

  /// No description provided for @original.
  ///
  /// In en, this message translates to:
  /// **'Original'**
  String get original;

  /// No description provided for @result.
  ///
  /// In en, this message translates to:
  /// **'Result'**
  String get result;

  /// No description provided for @notes.
  ///
  /// In en, this message translates to:
  /// **'Notes'**
  String get notes;

  /// No description provided for @webIntro.
  ///
  /// In en, this message translates to:
  /// **'Creates JPEG XL in every useful size, fallbacks for older browsers, and the code to paste into your site.'**
  String get webIntro;

  /// No description provided for @widthsLabel.
  ///
  /// In en, this message translates to:
  /// **'Sizes (px)'**
  String get widthsLabel;

  /// No description provided for @fallbacksLabel.
  ///
  /// In en, this message translates to:
  /// **'Fallback for older browsers'**
  String get fallbacksLabel;

  /// No description provided for @heroLabel.
  ///
  /// In en, this message translates to:
  /// **'Hero image (top of the page)'**
  String get heroLabel;

  /// No description provided for @heroHint.
  ///
  /// In en, this message translates to:
  /// **'Loads first, with high priority, for the best Largest Contentful Paint.'**
  String get heroHint;

  /// No description provided for @altLabel.
  ///
  /// In en, this message translates to:
  /// **'Alt text (accessibility and SEO)'**
  String get altLabel;

  /// No description provided for @baseUrlLabel.
  ///
  /// In en, this message translates to:
  /// **'Public URL path'**
  String get baseUrlLabel;

  /// No description provided for @nameLabel.
  ///
  /// In en, this message translates to:
  /// **'File name'**
  String get nameLabel;

  /// No description provided for @autoFocusLabel.
  ///
  /// In en, this message translates to:
  /// **'Load the subject first'**
  String get autoFocusLabel;

  /// No description provided for @autoFocusHint.
  ///
  /// In en, this message translates to:
  /// **'The subject sharpens before the background on slow connections.'**
  String get autoFocusHint;

  /// No description provided for @exportButton.
  ///
  /// In en, this message translates to:
  /// **'Export'**
  String get exportButton;

  /// No description provided for @exporting.
  ///
  /// In en, this message translates to:
  /// **'Exporting…'**
  String get exporting;

  /// No description provided for @webResultTitle.
  ///
  /// In en, this message translates to:
  /// **'{count} files ready'**
  String webResultTitle(Object count);

  /// No description provided for @smallerThan.
  ///
  /// In en, this message translates to:
  /// **'{percent} smaller than {format}'**
  String smallerThan(Object percent, Object format);

  /// No description provided for @largerThan.
  ///
  /// In en, this message translates to:
  /// **'{percent} larger than {format}'**
  String largerThan(Object percent, Object format);

  /// No description provided for @copy.
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get copy;

  /// No description provided for @copied.
  ///
  /// In en, this message translates to:
  /// **'Copied to clipboard'**
  String get copied;

  /// No description provided for @webHowTo.
  ///
  /// In en, this message translates to:
  /// **'Upload the folder to your server at the URL path above, then paste the code into your page.'**
  String get webHowTo;

  /// No description provided for @studioEmpty.
  ///
  /// In en, this message translates to:
  /// **'Open an image to explore what JPEG XL can do with it.'**
  String get studioEmpty;

  /// No description provided for @toolCompare.
  ///
  /// In en, this message translates to:
  /// **'Compare'**
  String get toolCompare;

  /// No description provided for @toolSubject.
  ///
  /// In en, this message translates to:
  /// **'Subject'**
  String get toolSubject;

  /// No description provided for @toolBackground.
  ///
  /// In en, this message translates to:
  /// **'Background'**
  String get toolBackground;

  /// No description provided for @toolCrop.
  ///
  /// In en, this message translates to:
  /// **'Crop & tiles'**
  String get toolCrop;

  /// No description provided for @toolProgressive.
  ///
  /// In en, this message translates to:
  /// **'Loading'**
  String get toolProgressive;

  /// No description provided for @toolBattle.
  ///
  /// In en, this message translates to:
  /// **'Format battle'**
  String get toolBattle;

  /// No description provided for @scoreLabel.
  ///
  /// In en, this message translates to:
  /// **'Quality score'**
  String get scoreLabel;

  /// No description provided for @scoreVisuallyLossless.
  ///
  /// In en, this message translates to:
  /// **'visually lossless'**
  String get scoreVisuallyLossless;

  /// No description provided for @scoreVeryHigh.
  ///
  /// In en, this message translates to:
  /// **'very high'**
  String get scoreVeryHigh;

  /// No description provided for @scoreHigh.
  ///
  /// In en, this message translates to:
  /// **'high'**
  String get scoreHigh;

  /// No description provided for @scoreMedium.
  ///
  /// In en, this message translates to:
  /// **'medium'**
  String get scoreMedium;

  /// No description provided for @scoreLow.
  ///
  /// In en, this message translates to:
  /// **'low'**
  String get scoreLow;

  /// No description provided for @scoreExplain.
  ///
  /// In en, this message translates to:
  /// **'SSIMULACRA2, the metric used to tune JPEG XL: 90+ means you cannot tell the difference.'**
  String get scoreExplain;

  /// No description provided for @saveResult.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get saveResult;

  /// No description provided for @working.
  ///
  /// In en, this message translates to:
  /// **'Working…'**
  String get working;

  /// No description provided for @subjectIntro.
  ///
  /// In en, this message translates to:
  /// **'JPEG XL can load the subject first and give it more detail than the background, in one normal image file.'**
  String get subjectIntro;

  /// No description provided for @detectSubject.
  ///
  /// In en, this message translates to:
  /// **'Find subject'**
  String get detectSubject;

  /// No description provided for @boostRun.
  ///
  /// In en, this message translates to:
  /// **'Create optimised file'**
  String get boostRun;

  /// No description provided for @boostStandard.
  ///
  /// In en, this message translates to:
  /// **'Standard'**
  String get boostStandard;

  /// No description provided for @boostLayered.
  ///
  /// In en, this message translates to:
  /// **'Subject boost'**
  String get boostLayered;

  /// No description provided for @boostSubjectScore.
  ///
  /// In en, this message translates to:
  /// **'subject score {score}'**
  String boostSubjectScore(Object score);

  /// No description provided for @boostRecommended.
  ///
  /// In en, this message translates to:
  /// **'Subject boost wins: sharper subject, smaller or equal file. Saved.'**
  String get boostRecommended;

  /// No description provided for @boostNotWorth.
  ///
  /// In en, this message translates to:
  /// **'The standard file is better for this image. Saved, with the subject loading first.'**
  String get boostNotWorth;

  /// No description provided for @boostWholeImage.
  ///
  /// In en, this message translates to:
  /// **'The subject fills most of the image: subject-first loading is enough.'**
  String get boostWholeImage;

  /// No description provided for @bgTransparent.
  ///
  /// In en, this message translates to:
  /// **'Transparent'**
  String get bgTransparent;

  /// No description provided for @bgWhite.
  ///
  /// In en, this message translates to:
  /// **'White'**
  String get bgWhite;

  /// No description provided for @bgBlack.
  ///
  /// In en, this message translates to:
  /// **'Black'**
  String get bgBlack;

  /// No description provided for @hardEdges.
  ///
  /// In en, this message translates to:
  /// **'Hard edges'**
  String get hardEdges;

  /// No description provided for @removeBackgroundButton.
  ///
  /// In en, this message translates to:
  /// **'Remove background'**
  String get removeBackgroundButton;

  /// No description provided for @bgUnavailable.
  ///
  /// In en, this message translates to:
  /// **'Background removal is not included in this build.'**
  String get bgUnavailable;

  /// No description provided for @cropIntro.
  ///
  /// In en, this message translates to:
  /// **'Crops and tiles are pixel-exact: every pixel is identical to the original, whatever its format.'**
  String get cropIntro;

  /// No description provided for @cropButton.
  ///
  /// In en, this message translates to:
  /// **'Save crop'**
  String get cropButton;

  /// No description provided for @cropHint.
  ///
  /// In en, this message translates to:
  /// **'Drag on the image to select an area.'**
  String get cropHint;

  /// No description provided for @gridLabel.
  ///
  /// In en, this message translates to:
  /// **'Grid {cols} × {rows}'**
  String gridLabel(Object cols, Object rows);

  /// No description provided for @tilesButton.
  ///
  /// In en, this message translates to:
  /// **'Split into tiles'**
  String get tilesButton;

  /// No description provided for @deepZoomButton.
  ///
  /// In en, this message translates to:
  /// **'Deep Zoom pyramid'**
  String get deepZoomButton;

  /// No description provided for @deepZoomHint.
  ///
  /// In en, this message translates to:
  /// **'For zoomable viewers such as OpenSeadragon: maps, plans, product close-ups.'**
  String get deepZoomHint;

  /// No description provided for @tilesDone.
  ///
  /// In en, this message translates to:
  /// **'{count} tiles saved'**
  String tilesDone(Object count);

  /// No description provided for @deepZoomDone.
  ///
  /// In en, this message translates to:
  /// **'{levels} levels, {count} tiles saved'**
  String deepZoomDone(Object levels, Object count);

  /// No description provided for @progressiveIntro.
  ///
  /// In en, this message translates to:
  /// **'Drag the slider to see what a visitor sees while the image downloads.'**
  String get progressiveIntro;

  /// No description provided for @downloaded.
  ///
  /// In en, this message translates to:
  /// **'{percent} downloaded · {size}'**
  String downloaded(Object percent, Object size);

  /// No description provided for @nothingYet.
  ///
  /// In en, this message translates to:
  /// **'Nothing to show yet'**
  String get nothingYet;

  /// No description provided for @battleIntro.
  ///
  /// In en, this message translates to:
  /// **'Each format is tuned to the same visual quality. Who makes the smallest file?'**
  String get battleIntro;

  /// No description provided for @battleTarget.
  ///
  /// In en, this message translates to:
  /// **'Target score {score}'**
  String battleTarget(Object score);

  /// No description provided for @battleRun.
  ///
  /// In en, this message translates to:
  /// **'Start'**
  String get battleRun;

  /// No description provided for @notScored.
  ///
  /// In en, this message translates to:
  /// **'not scored'**
  String get notScored;

  /// No description provided for @vsJpeg.
  ///
  /// In en, this message translates to:
  /// **'{percent} vs JPEG'**
  String vsJpeg(Object percent);

  /// No description provided for @winner.
  ///
  /// In en, this message translates to:
  /// **'Winner'**
  String get winner;

  /// No description provided for @inspectEmpty.
  ///
  /// In en, this message translates to:
  /// **'Open a file to see what is inside.'**
  String get inspectEmpty;

  /// No description provided for @dimensions.
  ///
  /// In en, this message translates to:
  /// **'Dimensions'**
  String get dimensions;

  /// No description provided for @fileSize.
  ///
  /// In en, this message translates to:
  /// **'File size'**
  String get fileSize;

  /// No description provided for @bitDepth.
  ///
  /// In en, this message translates to:
  /// **'Bit depth'**
  String get bitDepth;

  /// No description provided for @bitsPerPixel.
  ///
  /// In en, this message translates to:
  /// **'Bits per pixel'**
  String get bitsPerPixel;

  /// No description provided for @transparency.
  ///
  /// In en, this message translates to:
  /// **'Transparency'**
  String get transparency;

  /// No description provided for @colorProfile.
  ///
  /// In en, this message translates to:
  /// **'Colour profile'**
  String get colorProfile;

  /// No description provided for @exif.
  ///
  /// In en, this message translates to:
  /// **'EXIF'**
  String get exif;

  /// No description provided for @location.
  ///
  /// In en, this message translates to:
  /// **'Location (GPS)'**
  String get location;

  /// No description provided for @frames.
  ///
  /// In en, this message translates to:
  /// **'Frames'**
  String get frames;

  /// No description provided for @yes.
  ///
  /// In en, this message translates to:
  /// **'Yes'**
  String get yes;

  /// No description provided for @no.
  ///
  /// In en, this message translates to:
  /// **'No'**
  String get no;

  /// No description provided for @unused.
  ///
  /// In en, this message translates to:
  /// **'present but unused'**
  String get unused;

  /// No description provided for @embedded.
  ///
  /// In en, this message translates to:
  /// **'Embedded'**
  String get embedded;

  /// No description provided for @srgbDefault.
  ///
  /// In en, this message translates to:
  /// **'sRGB (default)'**
  String get srgbDefault;

  /// No description provided for @jxlDetails.
  ///
  /// In en, this message translates to:
  /// **'JPEG XL details'**
  String get jxlDetails;

  /// No description provided for @jxlMode.
  ///
  /// In en, this message translates to:
  /// **'Mode'**
  String get jxlMode;

  /// No description provided for @jxlLossless.
  ///
  /// In en, this message translates to:
  /// **'Lossless / original colour space'**
  String get jxlLossless;

  /// No description provided for @jxlLossy.
  ///
  /// In en, this message translates to:
  /// **'Lossy (XYB)'**
  String get jxlLossy;

  /// No description provided for @jxlContainsJpeg.
  ///
  /// In en, this message translates to:
  /// **'Contains the original JPEG'**
  String get jxlContainsJpeg;

  /// No description provided for @hdr.
  ///
  /// In en, this message translates to:
  /// **'HDR'**
  String get hdr;

  /// No description provided for @restoreJpeg.
  ///
  /// In en, this message translates to:
  /// **'Restore original JPEG'**
  String get restoreJpeg;

  /// No description provided for @suggestions.
  ///
  /// In en, this message translates to:
  /// **'Suggestions'**
  String get suggestions;

  /// Inspect suggestion for JPEG files.
  ///
  /// In en, this message translates to:
  /// **'Convert to JPEG XL with default settings: about 20% smaller with zero loss, and the original JPEG can be restored bit for bit.'**
  String get tipJpeg;

  /// No description provided for @tipLosslessPng.
  ///
  /// In en, this message translates to:
  /// **'Lossless JPEG XL keeps every pixel and is usually 30–50% smaller than PNG.'**
  String get tipLosslessPng;

  /// No description provided for @tipAnimatedGif.
  ///
  /// In en, this message translates to:
  /// **'Animated JPEG XL keeps full colour (GIF is limited to 256) and is usually much smaller.'**
  String get tipAnimatedGif;

  /// No description provided for @tipWebp.
  ///
  /// In en, this message translates to:
  /// **'WebP is limited to 8 bits and reduced colour detail; JPEG XL at the same size keeps more detail.'**
  String get tipWebp;

  /// No description provided for @tipGps.
  ///
  /// In en, this message translates to:
  /// **'This photo contains the GPS location where it was taken. Jixel removes it by default when exporting.'**
  String get tipGps;

  /// No description provided for @tipOpaqueAlpha.
  ///
  /// In en, this message translates to:
  /// **'The transparency channel is fully opaque: dropping it saves space (Jixel does this automatically).'**
  String get tipOpaqueAlpha;

  /// No description provided for @tipReconstruct.
  ///
  /// In en, this message translates to:
  /// **'This JPEG XL contains a recompressed JPEG: it can be turned back into the identical original.'**
  String get tipReconstruct;

  /// No description provided for @tipHdr.
  ///
  /// In en, this message translates to:
  /// **'This is an HDR image. Keep it as JPEG XL (or 16-bit PNG/TIFF) to preserve the highlights.'**
  String get tipHdr;

  /// No description provided for @tipLarge.
  ///
  /// In en, this message translates to:
  /// **'At {width}×{height} this image is larger than most screens: Optimize for web creates right-sized versions.'**
  String tipLarge(Object width, Object height);

  /// No description provided for @appearance.
  ///
  /// In en, this message translates to:
  /// **'Appearance'**
  String get appearance;

  /// No description provided for @themeSystem.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get themeSystem;

  /// No description provided for @themeLight.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themeLight;

  /// No description provided for @themeDark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themeDark;

  /// No description provided for @language.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get language;

  /// No description provided for @languageSystem.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get languageSystem;

  /// No description provided for @outputFolder.
  ///
  /// In en, this message translates to:
  /// **'Output folder'**
  String get outputFolder;

  /// No description provided for @change.
  ///
  /// In en, this message translates to:
  /// **'Change'**
  String get change;

  /// No description provided for @defaults.
  ///
  /// In en, this message translates to:
  /// **'Defaults'**
  String get defaults;

  /// No description provided for @about.
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get about;

  /// No description provided for @aboutBody.
  ///
  /// In en, this message translates to:
  /// **'Jixel is open source (Apache-2.0). Your images never leave your device.'**
  String get aboutBody;

  /// No description provided for @engineVersion.
  ///
  /// In en, this message translates to:
  /// **'Jixel {version} · libjxl {libjxl}'**
  String engineVersion(Object version, Object libjxl);

  /// No description provided for @licenses.
  ///
  /// In en, this message translates to:
  /// **'Open-source licences'**
  String get licenses;

  /// No description provided for @errorGeneric.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong: {message}'**
  String errorGeneric(Object message);

  /// No description provided for @unsupportedFiles.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 file skipped: not a supported image} other{{count} files skipped: not supported images}}'**
  String unsupportedFiles(int count);
}

class _AppLocalizationsDelegate extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) => <String>['en', 'it'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'it':
      return AppLocalizationsIt();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
