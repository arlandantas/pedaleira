import GainType from "../types/Gain";
import DistortionType from "../types/Distortion";
import ReverbType from "../types/Reverb";
import Effect from "../types/Effect";
import Distortion from "./Distortion";
import Gain from "./Gain";
import Reverb from "./Reverb";

type EffectComponentParams = {
  effect: Effect;
  onEffectUpdate: (effect: Effect) => void;
}

function EffectComponent({ effect, onEffectUpdate }: EffectComponentParams) {
  return (
    <div style={{ border: `1px solid ${effect._enabled ? "red" : "#888"}`, flexDirection: 'column', display: 'flex', padding: '10px', borderRadius: '5px', margin: '10px 0' }}>
      <div style={{ flexDirection: 'row', display: 'flex' }}>
        <div style={{ flex: 1 }}><b>{effect._type}</b></div>
        <div style={{
          borderRadius: '50%',
          background: effect._enabled ? 'red' : '#888',
          width: '15px',
          height: '15px',
          margin: '4px',
          boxShadow: effect._enabled ? '0px 0px 10px 0px red' : undefined
        }}></div>
      </div>
      <button onClick={() => {
        effect._enabled = !effect._enabled;
        onEffectUpdate(effect);
      }}>
        { effect._enabled ? 'Disable' : 'Enable' }
      </button>
      { effect._type === "distortion" ? <Distortion effect={effect as DistortionType} /> : null }
      { effect._type === "gain" ? <Gain effect={effect as GainType} /> : null }
      { effect._type === "reverb" ? <Reverb effect={effect as ReverbType} /> : null }
    </div>
  )
}

export default EffectComponent
