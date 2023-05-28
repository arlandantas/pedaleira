import { Tremolo as ToneTremolo, TremoloOptions } from 'tone'
import Effect from './Effect';

export class Tremolo extends Effect<ToneTremolo> {
  constructor(
    options: Partial<TremoloOptions> = { frequency: 9, depth: 0.75 },
    enabled = false
  ) {
    super("tremolo", new ToneTremolo(options), enabled);
  }
}

export default Tremolo
