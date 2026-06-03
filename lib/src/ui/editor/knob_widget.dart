import 'dart:math';
import 'package:flutter/material.dart';

class KnobWidget extends StatefulWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final ValueChanged<double> onChanged;

  const KnobWidget({
    super.key,
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.onChanged,
  });

  @override
  State<KnobWidget> createState() => _KnobWidgetState();
}

class _KnobWidgetState extends State<KnobWidget> {
  double? _dragStartY;
  double? _dragStartValue;

  @override
  Widget build(BuildContext context) {
    final normalized =
        ((widget.value - widget.min) / (widget.max - widget.min)).clamp(0.0, 1.0);
    final activeColor = Theme.of(context).colorScheme.primary;

    return GestureDetector(
      onPanStart: (d) {
        _dragStartY = d.localPosition.dy;
        _dragStartValue = widget.value;
      },
      onPanUpdate: (d) {
        final delta = (_dragStartY! - d.localPosition.dy) / 150.0;
        final newValue =
            (_dragStartValue! + delta * (widget.max - widget.min))
                .clamp(widget.min, widget.max);
        widget.onChanged(newValue);
      },
      child: SizedBox(
        width: 80,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CustomPaint(
              size: const Size(64, 64),
              painter: _KnobPainter(
                normalized: normalized,
                activeColor: activeColor,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              widget.label,
              style: const TextStyle(fontSize: 10, color: Colors.grey),
              textAlign: TextAlign.center,
            ),
            Text(
              widget.value.toStringAsFixed(2),
              style: const TextStyle(fontSize: 10, color: Colors.white),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }
}

class _KnobPainter extends CustomPainter {
  final double normalized; // 0.0 – 1.0
  final Color activeColor;

  const _KnobPainter({required this.normalized, required this.activeColor});

  // Arc from 7 o'clock (120° in Flutter canvas) to 5 o'clock — 300° sweep clockwise.
  static const double _startRad = 2.0944; // 120° = 2π/3
  static const double _sweepRad = 5.2360; // 300° = 5π/3

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = size.width / 2 - 10;
    final rect = Rect.fromCircle(center: center, radius: radius);

    final trackPaint = Paint()
      ..color = Colors.grey.shade800
      ..style = PaintingStyle.stroke
      ..strokeWidth = 4
      ..strokeCap = StrokeCap.round;

    canvas.drawArc(rect, _startRad, _sweepRad, false, trackPaint);

    if (normalized > 0) {
      final valuePaint = Paint()
        ..color = activeColor
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4
        ..strokeCap = StrokeCap.round;
      canvas.drawArc(
          rect, _startRad, _sweepRad * normalized, false, valuePaint);
    }

    final dotAngle = _startRad + _sweepRad * normalized;
    canvas.drawCircle(
      Offset(
        center.dx + radius * cos(dotAngle),
        center.dy + radius * sin(dotAngle),
      ),
      4,
      Paint()..color = activeColor,
    );
  }

  @override
  bool shouldRepaint(_KnobPainter old) =>
      old.normalized != normalized || old.activeColor != activeColor;
}
