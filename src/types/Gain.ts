import { Gain as ToneGain } from 'tone'
import Effect from './Effect';

export class Gain extends Effect<ToneGain> {
  constructor(
    options: Partial<GainOptions> = { gain: 1.5 },
    enabled = false
  ) {
    super("gain", new ToneGain(options), enabled);
  }
}

export default Gain
