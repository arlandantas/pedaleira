import { Chorus as ToneChorus, ChorusOptions } from 'tone'
import Effect from './Effect';

export class Chorus extends Effect<ToneChorus> {
  constructor(
    options: Partial<ChorusOptions> = { frequency: 4, depth: 0.5, delayTime: 2.5 },
    enabled = false
  ) {
    super("chorus", new ToneChorus(options), enabled);
  }
}

export default Chorus
