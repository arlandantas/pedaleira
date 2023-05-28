import { useState } from 'react';
import ChorusType from '../types/Chorus';

type ChorusParams = {
  effect: ChorusType;
}

function Chorus({ effect }: ChorusParams) {
  const [depth, setDepth] = useState(effect._effect.depth);
  const [frequency, setFrequency] = useState(effect._effect.frequency.value);
  const [delayTime, setDelayTime] = useState(effect._effect.delayTime);

  return (
    <div style={{ flexDirection: 'column', display: 'flex' }}>
      <label>Depth:</label>
      <input
        type="range"
        min="0"
        max="1"
        step="0.1"
        value={depth}
        onChange={({ target }) => {
          effect._effect.depth = Number(target.value)
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
      <label>Delay:</label>
      <input
        type="range"
        min="0"
        max="5"
        step="0.1"
        value={delayTime}
        onChange={({ target }) => {
          effect._effect.delayTime = Number(target.value)
          setDelayTime(Number(target.value))
        }}
      />
    </div>
  )
}

export default Chorus
