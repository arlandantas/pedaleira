import { useState } from 'react';
import { Gain as GainType } from '../types/Gain';

type GainParams = {
  effect: GainType;
}

function Gain({ effect }: GainParams) {
  const [gainAmount, setGainAmount] = useState(effect._effect.gain.value);

  return (
    <div style={{ flexDirection: 'column', display: 'flex' }}>
      <label>Gain Amount:</label>
      <input
        type="range"
        min="0"
        max="10"
        step="0.1"
        value={gainAmount}
        onChange={({ target }) => {
          effect._effect.gain.value = Number(target.value)
          setGainAmount(Number(target.value))
        }}
      />
    </div>
  )
}

export default Gain
