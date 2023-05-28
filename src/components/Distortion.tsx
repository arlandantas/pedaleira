import { useState } from 'react';
import { Distortion as DistortionType } from '../types/Distortion';

type DistortionParams = {
  effect: DistortionType;
}

function Distortion({ effect }: DistortionParams) {
  const [distortion, setDistortion] = useState(effect._effect.distortion);

  return (
    <>
      <div style={{ border: '1px solid blue', flexDirection: 'column', display: 'flex' }}>
        <label>Distortion Value:</label>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={distortion}
          onChange={({ target }) => {
            effect._effect.distortion = Number(target.value)
            setDistortion(Number(target.value))
          }}
        />
      </div>
    </>
  )
}

export default Distortion
