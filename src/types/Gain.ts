import { Gain as ToneGain } from 'tone'
import Effect from './Effect';

export class Gain extends Effect {
  constructor(
    options: Partial<GainOptions> | undefined = undefined,
    enabled = false
  ) {
    super("gain", new ToneGain(options), enabled);
  }
}

export default Gain
