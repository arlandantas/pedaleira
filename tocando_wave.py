import pyaudio
import wave

pa = pyaudio.PyAudio()
wav_file = wave.open('data/audio-recording.wav')

print(wav_file.getframerate())

frame_rate = 44100

stream_out = pa.open(
    rate=wav_file.getframerate(),     # sampling rate
    # rate=frame_rate,
    channels=wav_file.getnchannels(), # number of output channels
    format=pa.get_format_from_width(wav_file.getsampwidth()),  # sample format and length
    output=True,             # output stream flag
    output_device_index=5,   # output device index
    frames_per_buffer=1024,  # buffer length
)

time = 10 # seconds
output_audio = wav_file.readframes(time * frame_rate)
stream_out.write(output_audio)