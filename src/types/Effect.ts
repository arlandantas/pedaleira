import { ToneAudioNode } from 'tone';

export type EffectType = "distortion" | "gain" | "reverb";

export class Effect<NodeType extends ToneAudioNode = ToneAudioNode> {
  _id: string;
  _type: EffectType;
  _effect: NodeType;
  _enabled: boolean;

  constructor(
    type: EffectType,
    effect: NodeType,
    enabled = false,
  ) {
    this._type = type;
    this._effect = effect;
    this._enabled = enabled;
    this._id = '';
    for (let i = 0; i < 16; i++) {
      const charCode = Math.ceil(Math.random() * 26) + 96;
      this._id = `${this._id}${String.fromCharCode(charCode)}`;
    }
  }
}

export default Effect