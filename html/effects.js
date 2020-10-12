export class tremolo extends GainNode {
    constructor (ctx, _freq = 60) {
        super(ctx)
        this.freq = _freq

        this.oscillatorNode = ctx.createOscillator()

        this.oscillatorNode.connect(this.gain)
        this.oscillatorNode.type = 'sine'
        this.oscillatorNode.start(0)
        this.oscillatorNode.frequency.value = _freq
    }

    setFrequency (_freq) {
        this.freq = _freq
        this.oscillatorNode.frequency.value = _freq
    }
}

export default {
    tremolo
}