import { useState } from 'react';
import { Reverb as ReverbType } from '../types/Reverb';

type ReverbParams = {
  effect: ReverbType;
}

function Reverb({ effect }: ReverbParams) {
  const [decay, setDecay] = useState(effect._effect.decay);
  const [wet, setWet] = useState(effect._effect.wet.value);

  return (
    <div style={{ flexDirection: 'column', display: 'flex' }}>
      <label>Decay:</label>
      <input
        type="range"
        min="0.1"
        max="10"
        step="0.1"
        value={Number(decay)}
        onChange={({ target }) => {
          effect._effect.decay = Number(target.value)
          setDecay(Number(target.value))
        }}
      />
      <label>Wet:</label>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={wet}
        onChange={({ target }) => {
          effect._effect.wet.value = Number(target.value)
          setWet(Number(target.value))
        }}
      />
    </div>
  )
}

export default Reverb
