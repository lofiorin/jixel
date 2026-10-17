/// An image you can draw a rectangle on (crop, manual subject).
library;

import 'package:flutter/material.dart';

import '../../engine/models.dart';
import 'image_views.dart';

/// Shows [preview] and lets the user drag out a rectangle. Reports it in
/// the pixels of the full image ([imageWidth] × [imageHeight]), which may be
/// larger than the preview.
class RectSelector extends StatefulWidget {
  const RectSelector({
    super.key,
    required this.preview,
    required this.imageWidth,
    required this.imageHeight,
    this.rect,
    required this.onChanged,
    this.color,
  });

  final RgbaPreview preview;
  final int imageWidth;
  final int imageHeight;
  final PixelRect? rect;
  final ValueChanged<PixelRect> onChanged;
  final Color? color;

  @override
  State<RectSelector> createState() => _RectSelectorState();
}

class _RectSelectorState extends State<RectSelector> {
  Offset? _start;

  PixelRect _toPixels(Offset a, Offset b, Size box) {
    double cx(double v) => (v / box.width).clamp(0.0, 1.0) * widget.imageWidth;
    double cy(double v) => (v / box.height).clamp(0.0, 1.0) * widget.imageHeight;
    final x0 = cx(a.dx < b.dx ? a.dx : b.dx).floor();
    final y0 = cy(a.dy < b.dy ? a.dy : b.dy).floor();
    final x1 = cx(a.dx > b.dx ? a.dx : b.dx).ceil();
    final y1 = cy(a.dy > b.dy ? a.dy : b.dy).ceil();
    return PixelRect(x0, y0, (x1 - x0).clamp(1, widget.imageWidth), (y1 - y0).clamp(1, widget.imageHeight));
  }

  @override
  Widget build(BuildContext context) {
    final color = widget.color ?? Theme.of(context).colorScheme.primary;
    return AspectRatio(
      aspectRatio: widget.imageWidth / widget.imageHeight,
      child: LayoutBuilder(
        builder: (context, box) {
          final size = box.biggest;
          return GestureDetector(
            onPanStart: (d) => _start = d.localPosition,
            onPanUpdate: (d) => widget.onChanged(_toPixels(_start!, d.localPosition, size)),
            child: MouseRegion(
              cursor: SystemMouseCursors.precise,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  PreviewImage(preview: widget.preview, fit: BoxFit.fill),
                  if (widget.rect case final r?)
                    CustomPaint(
                      painter: _RectPainter(
                        Rect.fromLTWH(
                          r.x / widget.imageWidth * size.width,
                          r.y / widget.imageHeight * size.height,
                          r.width / widget.imageWidth * size.width,
                          r.height / widget.imageHeight * size.height,
                        ),
                        color,
                      ),
                    ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

/// Dims everything outside the rectangle and outlines it.
class _RectPainter extends CustomPainter {
  _RectPainter(this.rect, this.color);
  final Rect rect;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final outside = Path()
      ..fillType = PathFillType.evenOdd
      ..addRect(Offset.zero & size)
      ..addRect(rect);
    canvas.drawPath(outside, Paint()..color = Colors.black45);
    canvas.drawRect(
      rect,
      Paint()
        ..color = color
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
  }

  @override
  bool shouldRepaint(_RectPainter old) => old.rect != rect || old.color != color;
}
