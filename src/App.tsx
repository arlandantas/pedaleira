import { useEffect, useMemo, useState } from 'react'
import './App.css'
import { ToneAudioNode, UserMedia, start as startTone } from 'tone'
import EffectList from './components/EffectList';
import Effect, { EffectType } from './types/Effect';
import Distortion from './types/Distortion';
import Reverb from './types/Reverb';
import Gain from './types/Gain';

function App() {
  const [isListening, setListening] = useState(false);
  const [effectType, setEffectType] = useState<EffectType>("distortion");
  const [effects, updateEffects] = useState<Effect[]>([]);

  const mic = useMemo(() => new UserMedia(), []);

  const toggleListening = async () => {
    if (isListening) {
      mic.close();
    } else {
      startTone();
      await mic.open();
    }
    setListening(!isListening);
  };

  const onEffectsUpdate = (updatedEffects: Effect[]) => {
    updateEffects(updatedEffects);
    console.log("Updating effect", { updatedEffects });

    let currentEffect: ToneAudioNode = mic;
    currentEffect.disconnect();
    for (const effect of updatedEffects) {
      effect._effect.disconnect();
      if (effect._enabled) {
        currentEffect.connect(effect._effect);
        currentEffect = effect._effect;
      }
    }
    currentEffect.toDestination();
  };

  useEffect(() => {
    onEffectsUpdate(effects);
  }, []);

  const newEffect = () => {
    let effect = null;
    if (effectType == "distortion") {
      effect = new Distortion();
    } else if (effectType == "reverb") {
      effect = new Reverb()
    } else if (effectType == "gain") {
      effect = new Gain()
    } else {
      return;
    }
    onEffectsUpdate([
      ...effects,
      effect,
    ]);
  };

  return (
    <>
      <div>
        <h1>Pedaleira</h1>
        <button onClick={toggleListening}>{isListening ? 'Stop' : 'Start'} listening</button>
        <br />
        <select onChange={(evt) => setEffectType(evt.target.value as EffectType)}>
          {["distortion", "gain", "reverb"].map(e => <option value={e} key={e}>{e}</option>)}
        </select>
        <button onClick={newEffect}>Add Effect</button><br />
        <EffectList effects={effects} onEffectsUpdate={onEffectsUpdate} />
      </div>
    </>
  )
}

export default App
