import pyaudio
pa = pyaudio.PyAudio()

# api_count = pa.get_host_api_count()
# apis_gen = map(pa.get_host_api_info_by_index, range(api_count))
# for el in apis_gen:
#     print(el)


# chosen_device_index = -1
print ("Index\tName\t\t\t\tInput\tOutput\tSampleRate")
for x in range(0,pa.get_device_count()):
    info = pa.get_device_info_by_index(x)
    print ("{}\t{}\t{}\t{}\t{}".format(
        info['index'],
        info['name'],
        info['maxInputChannels'],
        info['maxOutputChannels'],
        info['defaultSampleRate']))
    # print (info)
#     if info["name"] == "pulse":
#         chosen_device_index = info["index"]
#         print ("Chosen index: ", chosen_device_index)

# p = pyaudio.PyAudio()
# stream = p.open(format=pyaudio.paInt16, channels=1, rate=16000, input_device_index=chosen_device_index, input=True, output=False)
# stream.start_stream()