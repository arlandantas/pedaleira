from audiolazy import *

rate = 44100 # Sampling rate, in samples/second
s, Hz = sHz(rate) # Seconds and hertz
ms = 1e-3 * s
note1 = karplus_strong(440 * Hz) # Pluck "digitar" synth
note2 = zeros(300 * ms).append(karplus_strong(880 * Hz))
note3 = zeros(600 * ms).append(karplus_strong(220 * Hz)) # Pluck "digitar" synth
notes = (note1 + note2 + note3) * .5
sound = notes.take(int(2 * s)) # 2 seconds of a Karplus-Strong note

with AudioIO(True, 'alsa') as player: # True means "wait for all sounds to stop"
#   player.play(player.record(input_device_index=5), output_device_index=5)
  player.play(sound, rate=rate, output_device_index=5)