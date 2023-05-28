import { Reverb as ToneReverb } from 'tone/build/esm/effect'
import Effect from './Effect';
import { EffectOptions } from 'tone/build/esm/effect/Effect';
import { Seconds } from 'tone/build/esm/core/type/Units';

interface ReverbOptions extends EffectOptions {
  decay: Seconds;
  preDelay: Seconds;
}

export class Reverb extends Effect {
  constructor(
    options: Partial<ReverbOptions> | undefined = undefined,
    enabled = false
  ) {
    super("reverb", new ToneReverb(options), enabled);
  }
}

export default Reverb