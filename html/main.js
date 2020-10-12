import helpers from "./helpers.js";
import effects, { tremolo } from "./effects.js";

// create web audio api context
window.audioCtx = new (window.AudioContext || window.webkitAudioContext)()
window.audioSrc = null

if (navigator.mediaDevices) {
    navigator.mediaDevices.getUserMedia ({audio: true})
    .then((stream) => {
        window.audioSrc = audioCtx.createMediaStreamSource(stream);
        init()
    })
    .catch(function(err) {
        console.log('The following gUM error occured: ' + err);
    });
} else {
   alert('getUserMedia not supported on your browser!');
}

function init () {
    // create Oscillator node
    var osqr = audioCtx.createOscillator()
    osqr.type = 'square'
    osqr.frequency.setValueAtTime(440, audioCtx.currentTime) // value in hertz
    osqr.start();
    
    window.trem = new tremolo(audioCtx, 2)
    audioSrc.connect(trem)
    trem.connect(audioCtx.destination)

    // audioCtx.suspend()

    console.log("Initilized")
    
    // const osine = new OscillatorNode(audioCtx)
    // osine.frequency.setValueAtTime(.5, audioCtx.currentTime) // value in hertz
    // osine.start()
    // const gainNode = new GainNode(audioCtx)
    // osine.connect(gainNode.gain)
    // osqr.connect(gainNode);
    // gainNode.connect(audioCtx.destination)
}

window.suspend = () => {
    audioCtx.suspend()
    console.log("Suspended")
}

window.resume = () => {
    audioCtx.resume()
    console.log("Resumed")
}

window.change_tremolo = (_vlr) => {
    console.log(_vlr)
    window.trem.setFrequency(_vlr)
}