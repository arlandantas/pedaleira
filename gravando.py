import pyaudio
import wave

pa = pyaudio.PyAudio()

sample_rate = 44100

stream_in = pa.open(
    rate=sample_rate,
    channels=2,
    format=pyaudio.paInt16,
    input=True,                   # input stream flag
    input_device_index=5,         # input device index
    frames_per_buffer=1024
)

# read 5 seconds of the input stream
tempo = 15 # time in seconds
input_audio = stream_in.read(tempo * sample_rate)

output_filename = 'data/audio-recording.wav'
wav_file = wave.open(output_filename, 'wb')

# define audio stream properties
wav_file.setnchannels(2)        # number of channels
wav_file.setsampwidth(2)        # sample width in bytes
wav_file.setframerate(sample_rate)    # sampling rate in Hz

# write samples to the file
wav_file.writeframes(input_audio)