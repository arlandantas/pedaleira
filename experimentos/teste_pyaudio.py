import pyaudio

# pa = pyaudio.PyAudio()
# pa.get_default_host_api_info()


# import pyaudio
# pa = pyaudio.PyAudio()
# chosen_device_index = -1
# for x in range(0,pa.get_device_count()):
#     info = pa.get_device_info_by_index(x)
#     print ("{}:\t {}".format(info['index'], info['name']))
#     if info["name"] == "pulse":
#         chosen_device_index = info["index"]
#         print ("Chosen index: ", chosen_device_index)

p = pyaudio.PyAudio()
stream = p.open(format=pyaudio.paInt16, channels=1, rate=48000, input_device_index=5, input=True, output=False)
stream.start_stream()
while True:
    print('hi')