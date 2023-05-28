import { useState } from 'react';
import { Tremolo as TremoloType } from '../types/Tremolo';

type TremoloParams = {
  effect: TremoloType;
}

function Tremolo({ effect }: TremoloParams) {
  const [depth, setDepth] = useState(effect._effect.depth.value);
  const [frequency, setFrequency] = useState(effect._effect.frequency.value);

  return (
    <div style={{ flexDirection: 'column', display: 'flex' }}>
      <label>Depth Amount:</label>
      <input
        type="range"
        min="0"
        max="1"
        step="0.1"
        value={depth}
        onChange={({ target }) => {
          effect._effect.depth.value = Number(target.value)
          setDepth(Number(target.value))
        }}
      />
      <label>Frequency:</label>
      <input
        type="range"
        min="0"
        max="20"
        step="1"
        value={frequency}
        onChange={({ target }) => {
          effect._effect.frequency.value = Number(target.value)
          setFrequency(Number(target.value))
        }}
      />
    </div>
  )
}

export default Tremolo
