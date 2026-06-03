import 'dart:convert';

enum PedalSlot {
  noiseGate,
  compressor,
  overdrive,
  distortion,
  fuzz,
  chorus,
  tremolo,
  delay,
  reverb,
}

const Map<PedalSlot, String> kPedalNames = {
  PedalSlot.noiseGate: 'Noise Gate',
  PedalSlot.compressor: 'Compressor',
  PedalSlot.overdrive: 'Overdrive',
  PedalSlot.distortion: 'Distortion',
  PedalSlot.fuzz: 'Fuzz',
  PedalSlot.chorus: 'Chorus',
  PedalSlot.tremolo: 'Tremolo',
  PedalSlot.delay: 'Delay',
  PedalSlot.reverb: 'Reverb',
};

const Map<PedalSlot, Map<String, double>> kDefaultParams = {
  PedalSlot.noiseGate: {'threshold': 0.01},
  PedalSlot.compressor: {
    'threshold_db': -18.0,
    'ratio': 4.0,
    'attack': 0.01,
    'release': 0.1,
  },
  PedalSlot.overdrive: {'drive': 3.0, 'tone': 0.5},
  PedalSlot.distortion: {'drive': 8.0, 'level': 0.5},
  PedalSlot.fuzz: {'fuzz': 0.7, 'level': 0.7},
  PedalSlot.chorus: {'rate': 0.5, 'depth': 1.5, 'mix': 0.5},
  PedalSlot.tremolo: {'rate': 4.0, 'depth': 0.5},
  PedalSlot.delay: {'time_ms': 300.0, 'feedback': 0.4, 'mix': 0.4},
  PedalSlot.reverb: {'room_size': 0.5, 'mix': 0.3},
};

// Min/max ranges per param key — used by KnobWidget
const Map<String, (double, double)> kParamRanges = {
  'threshold': (0.0, 0.5),
  'threshold_db': (-60.0, 0.0),
  'ratio': (1.0, 20.0),
  'attack': (0.001, 0.5),
  'release': (0.01, 2.0),
  'drive': (1.0, 20.0),
  'tone': (0.0, 1.0),
  'level': (0.0, 1.0),
  'fuzz': (0.0, 1.0),
  'rate': (0.1, 10.0),
  'depth': (0.0, 3.0),
  'mix': (0.0, 1.0),
  'time_ms': (50.0, 1000.0),
  'feedback': (0.0, 0.95),
  'room_size': (0.0, 1.0),
};

class PedalState {
  final PedalSlot slot;
  final bool bypassed;
  final Map<String, double> params;

  const PedalState({
    required this.slot,
    required this.bypassed,
    required this.params,
  });

  PedalState copyWith({bool? bypassed, Map<String, double>? params}) {
    return PedalState(
      slot: slot,
      bypassed: bypassed ?? this.bypassed,
      params: params ?? this.params,
    );
  }

  Map<String, dynamic> toJson() => {
    'slot': slot.index,
    'bypassed': bypassed,
    'params': params,
  };

  factory PedalState.fromJson(Map<String, dynamic> json) {
    return PedalState(
      slot: PedalSlot.values[json['slot'] as int],
      bypassed: json['bypassed'] as bool,
      params: Map<String, double>.from(
        (json['params'] as Map<String, dynamic>).map(
          (k, v) => MapEntry(k, (v as num).toDouble()),
        ),
      ),
    );
  }
}

class Preset {
  final String name;
  final List<PedalState> pedals;

  const Preset({required this.name, required this.pedals});

  Map<String, dynamic> toJson() => {
    'name': name,
    'pedals': pedals.map((p) => p.toJson()).toList(),
  };

  factory Preset.fromJson(Map<String, dynamic> json) {
    return Preset(
      name: json['name'] as String,
      pedals: (json['pedals'] as List<dynamic>)
          .map((p) => PedalState.fromJson(p as Map<String, dynamic>))
          .toList(),
    );
  }
}
