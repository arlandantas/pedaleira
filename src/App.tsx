import { useEffect, useMemo, useState } from 'react'
import './App.css'
import { Loop, MonoSynth, Sampler, ToneAudioNode, Transport, UserMedia, start as startTone } from 'tone'
import EffectList from './components/EffectList';
import Effect, { EffectType } from './types/Effect';
import Distortion from './types/Distortion';
import Reverb from './types/Reverb';
import Gain from './types/Gain';
import Tremolo from './types/Tremolo';
import Chorus from './types/Chorus';

type SourceType = "mic" | "synth" | "eletric guitar" | "eletric bass";

function App() {
  const [isListening, setListening] = useState(false);
  const [currentSource, setCurrentSource] = useState<SourceType>("synth");
  const [effectType, setEffectType] = useState<EffectType>("distortion");
  const [effects, updateEffects] = useState<Effect[]>([
    new Distortion(),
    new Gain(),
    new Reverb(),
    new Tremolo(),
    new Chorus()
  ]);

  const micSource = useMemo(() => new UserMedia(), []);

  const synthSource = useMemo(() => {
    const synth = new MonoSynth({  detune: 1200, oscillator: { type: 'sine' } });
    new Loop(time => {
      synth.triggerAttackRelease("C3", "4n", time);
      synth.triggerAttackRelease("E3", "4n", time + 0.25);
      synth.triggerAttackRelease("G3", "4n", time + 0.5);
    }, "1n").start(0);
    
    return synth;
  }, []);

  const guitarSource = useMemo(() => {
    const guitar = new Sampler({
      urls: {
        'D#3': 'Ds3.[mp3|ogg]',
        'D#4': 'Ds4.[mp3|ogg]',
        'D#5': 'Ds5.[mp3|ogg]',
        'E2': 'E2.[mp3|ogg]',
        'F#2': 'Fs2.[mp3|ogg]',
        'F#3': 'Fs3.[mp3|ogg]',
        'F#4': 'Fs4.[mp3|ogg]',
        'F#5': 'Fs5.[mp3|ogg]',
        'A2': 'A2.[mp3|ogg]',
        'A3': 'A3.[mp3|ogg]',
        'A4': 'A4.[mp3|ogg]',
        'A5': 'A5.[mp3|ogg]',
        'C3': 'C3.[mp3|ogg]',
        'C4': 'C4.[mp3|ogg]',
        'C5': 'C5.[mp3|ogg]',
        'C6': 'C6.[mp3|ogg]',
        'C#2': 'Cs2.[mp3|ogg]'
      },
      baseUrl: "./guitar-eletric/",
    });

    new Loop(time => {
      guitar.triggerAttackRelease("C3", "4n", time);
      guitar.triggerAttackRelease("E3", "4n", time + 0.25);
      guitar.triggerAttackRelease("G3", "4n", time + 0.5);
    }, "1n").start(0);

    return guitar;
  }, []);

  const bassSource = useMemo(() => {
    const bass = new Sampler({
      urls: {
        'A#1': 'As1.[mp3|ogg]',
        'A#2': 'As2.[mp3|ogg]',
        'A#3': 'As3.[mp3|ogg]',
        'A#4': 'As4.[mp3|ogg]',
        'C#1': 'Cs1.[mp3|ogg]',
        'C#2': 'Cs2.[mp3|ogg]',
        'C#3': 'Cs3.[mp3|ogg]',
        'C#4': 'Cs4.[mp3|ogg]',
        'E1': 'E1.[mp3|ogg]',
        'E2': 'E2.[mp3|ogg]',
        'E3': 'E3.[mp3|ogg]',
        'E4': 'E4.[mp3|ogg]',
        'G1': 'G1.[mp3|ogg]',
        'G2': 'G2.[mp3|ogg]',
        'G3': 'G3.[mp3|ogg]',
        'G4': 'G4.[mp3|ogg]'
      },
      baseUrl: "./bass-eletric/",
    });

    new Loop(time => {
      bass.triggerAttackRelease("C3", "4n", time);
      bass.triggerAttackRelease("E3", "4n", time + 0.25);
      bass.triggerAttackRelease("G3", "8n", time + 0.825);
    }, "1n").start(0);

    return bass;
  }, []);

  const toggleListening = async () => {
    if (isListening) {
      Transport.stop();
      micSource.close();
    } else {
      startTone();
      Transport.start();
      Transport.bpm.value = 80;
      await micSource.open();
    }
    setListening(!isListening);
  };

  const setSource = (source: SourceType) => {
    setCurrentSource(source);
    onEffectsUpdate(effects, source);
  };

  const onEffectsUpdate = (updatedEffects: Effect[], source: SourceType = currentSource) => {
    updateEffects(updatedEffects);

    micSource.disconnect();
    guitarSource.disconnect();
    bassSource.disconnect();
    synthSource.disconnect();

    let currentEffect: ToneAudioNode = micSource;
    if (source === "eletric guitar") {
      currentEffect = guitarSource;
    } else if (source === "eletric bass") {
      currentEffect = bassSource;
    } else if (source === "synth") {
      currentEffect = synthSource;
    }

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
    } else if (effectType == "tremolo") {
      effect = new Tremolo()
    } else if (effectType == "chorus") {
      effect = new Chorus()
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
        Source:
        <select value={currentSource} onChange={(evt) => setSource(evt.target.value as SourceType)}>
          {["mic", "synth", "eletric guitar", "eletric bass"].map(e => <option value={e} key={e}>{e}</option>)}
        </select>
        <br />
        Effect:
        <select onChange={(evt) => setEffectType(evt.target.value as EffectType)}>
          {["distortion", "gain", "reverb", "tremolo"].map(e => <option value={e} key={e}>{e}</option>)}
        </select>
        <button onClick={newEffect}>Add</button><br />
        <EffectList effects={effects} onEffectsUpdate={onEffectsUpdate} />
      </div>
    </>
  )
}

export default App
