import Effect from '../types/Effect';
import EffectComponent from './EffectComponent';

type EffectsListParams = {
  effects: Array<Effect>;
  onEffectsUpdate: (effects: Array<Effect>) => void;
}

function EffectList({ effects, onEffectsUpdate: onUpdate }: EffectsListParams) {
  const onEffectUpdate = (effect: Effect, index: number) => {
    onUpdate(effects.map((e, i) => i === index ? effect : e));
  };
  return (
    <div>
      <h2>Efeitos</h2>
      {effects.map(
        (effect, index) =>
          <EffectComponent
            key={effect._id}
            effect={effect}
            onEffectUpdate={(effect) => onEffectUpdate(effect, index)}
          />
      )}
    </div>
  )
}

export default EffectList
