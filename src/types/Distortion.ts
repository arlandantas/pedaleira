import { Distortion as ToneDistortion, DistortionOptions } from 'tone'
import Effect from './Effect';

export class Distortion extends Effect<ToneDistortion> {
  constructor(
    options: Partial<DistortionOptions> | undefined = undefined,
    enabled = false
  ) {
    super("distortion", new ToneDistortion(options), enabled);
  }
}

export default Distortion