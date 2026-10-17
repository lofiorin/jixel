/// Showing engine previews on screen: plain view and before/after slider.
library;

import 'dart:ui' as ui;

import 'package:flutter/material.dart';

import '../../engine/models.dart';

/// Turns an [RgbaPreview] into a `ui.Image` once and caches it.
class PreviewImage extends StatefulWidget {
  const PreviewImage({super.key, required this.preview, this.fit = BoxFit.contain, this.checkerboard = false});

  final RgbaPreview preview;
  final BoxFit fit;

  /// Show a checkerboard behind transparent areas.
  final bool checkerboard;

  @override
  State<PreviewImage> createState() => _PreviewImageState();
}

class _PreviewImageState extends State<PreviewImage> {
  ui.Image? _image;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void didUpdateWidget(PreviewImage old) {
    super.didUpdateWidget(old);
    if (old.preview != widget.preview) _load();
  }

  Future<void> _load() async {
    final image = await widget.preview.toImage();
    if (!mounted) return image.dispose();
    setState(() {
      _image?.dispose();
      _image = image;
    });
  }

  @override
  void dispose() {
    _image?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final image = _image;
    if (image == null) return const Center(child: CircularProgressIndicator());
    final raw = RawImage(image: image, fit: widget.fit, filterQuality: FilterQuality.medium);
    if (!widget.checkerboard) return raw;
    return AspectRatio(
      aspectRatio: image.width / image.height,
      child: CustomPaint(painter: CheckerboardPainter(Theme.of(context).colorScheme), child: raw),
    );
  }
}

/// Grey checks that reveal transparency.
class CheckerboardPainter extends CustomPainter {
  CheckerboardPainter(this.scheme);
  final ColorScheme scheme;

  @override
  void paint(Canvas canvas, Size size) {
    const cell = 12.0;
    final a = Paint()..color = scheme.surfaceContainerHighest;
    final b = Paint()..color = scheme.surfaceContainerLow;
    for (var y = 0.0; y < size.height; y += cell) {
      for (var x = 0.0; x < size.width; x += cell) {
        final even = ((x / cell).floor() + (y / cell).floor()).isEven;
        canvas.drawRect(Rect.fromLTWH(x, y, cell, cell), even ? a : b);
      }
    }
  }

  @override
  bool shouldRepaint(CheckerboardPainter old) => old.scheme != scheme;
}

/// Two images of the same scene with a draggable divider. Pinch or scroll to
/// zoom: both sides zoom together, so details can be compared.
class CompareSlider extends StatefulWidget {
  const CompareSlider({
    super.key,
    required this.before,
    required this.after,
    required this.beforeLabel,
    required this.afterLabel,
  });

  final RgbaPreview before;
  final RgbaPreview after;
  final String beforeLabel;
  final String afterLabel;

  @override
  State<CompareSlider> createState() => _CompareSliderState();
}

class _CompareSliderState extends State<CompareSlider> {
  double _split = 0.5;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return AspectRatio(
      aspectRatio: widget.before.width / widget.before.height,
      child: LayoutBuilder(
        builder: (context, box) {
          void move(Offset local) => setState(() => _split = (local.dx / box.maxWidth).clamp(0.0, 1.0));
          return GestureDetector(
            onHorizontalDragUpdate: (d) => move(d.localPosition),
            onTapDown: (d) => move(d.localPosition),
            child: Stack(
              fit: StackFit.expand,
              children: [
                InteractiveViewer(
                  maxScale: 8,
                  child: Stack(
                    fit: StackFit.expand,
                    children: [
                      PreviewImage(preview: widget.after, fit: BoxFit.fill),
                      ClipRect(
                        clipper: _LeftClipper(_split),
                        child: PreviewImage(preview: widget.before, fit: BoxFit.fill),
                      ),
                    ],
                  ),
                ),
                // Divider and handle.
                Positioned(
                  left: box.maxWidth * _split - 1,
                  top: 0,
                  bottom: 0,
                  child: IgnorePointer(child: Container(width: 2, color: Colors.white)),
                ),
                Positioned(
                  left: box.maxWidth * _split - 18,
                  top: box.maxHeight / 2 - 18,
                  child: IgnorePointer(
                    child: Container(
                      width: 36,
                      height: 36,
                      decoration: const BoxDecoration(
                        color: Colors.white,
                        shape: BoxShape.circle,
                        boxShadow: [BoxShadow(blurRadius: 8, color: Colors.black26)],
                      ),
                      child: Icon(Icons.compare_arrows, size: 20, color: scheme.primary),
                    ),
                  ),
                ),
                Positioned(left: 12, top: 12, child: _Tag(widget.beforeLabel)),
                Positioned(right: 12, top: 12, child: _Tag(widget.afterLabel)),
              ],
            ),
          );
        },
      ),
    );
  }
}

class _LeftClipper extends CustomClipper<Rect> {
  _LeftClipper(this.fraction);
  final double fraction;

  @override
  Rect getClip(Size size) => Rect.fromLTWH(0, 0, size.width * fraction, size.height);

  @override
  bool shouldReclip(_LeftClipper old) => old.fraction != fraction;
}

class _Tag extends StatelessWidget {
  const _Tag(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
    decoration: BoxDecoration(color: Colors.black54, borderRadius: BorderRadius.circular(8)),
    child: Text(
      text,
      style: const TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.w600),
    ),
  );
}
