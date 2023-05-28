import DistortionType from "../types/Distortion";
import Effect from "../types/Effect";
import Distortion from "./Distortion";

type EffectComponentParams = {
  effect: Effect;
  onEffectUpdate: (effect: Effect) => void;
}

function EffectComponent({ effect, onEffectUpdate }: EffectComponentParams) {
  return (
    <>
      <div style={{ border: '1px solid red', flexDirection: 'column', display: 'flex' }}>
        <span>
          {effect._id} | {effect._type}
        </span>
        <button onClick={() => {
          effect._enabled = !effect._enabled;
          onEffectUpdate(effect);
        }}>
          { effect._enabled ? 'Disable' : 'Enable' }
        </button>
        { effect._type === "distortion" ? <Distortion effect={effect as DistortionType} /> : null }
      </div>
    </>
  )
}

export default EffectComponent
