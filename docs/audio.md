# Audio Input

Audio is captured and written to a buffer from the configured audio sink using Pulseaudio.

## Initialization

A new `Buffer` is created which samples will be written to. A temporary PulseAudio main loop and context are being created for connecting to the configured audio source. The PulseAudio main loop it thread safe and iteratively executes `poll` system calls and dispatches fired events. The PulseAudio context is used for asynchronously communicating with the PulseAudio server.

The configuration file specifies the name of the sink audio is received from. This name is used for finding the audio source owning the sink and connecting to it.

## Connecting to source and stream

As a threaded PulseAudio is used, it is necessary for locks to be held whenever a PulseAudio function is called that used an object assicated with the main loop. Callbacks are by default asynchronous. To make them synchronous the callback needs to signal to the main loop when it has been called.

Once connected to the correct audio source, the audio stream is initialized. The stream is connected to the configured source.

## Reading from stream

After the stream has been successfully connected to the source, bytes can be read from the stream. Data is read from the stream in batches, so reading from the stream is paused until a certain threshold of bytes available is reached. Data read from the stream is written into the previously created buffer.
Since the buffer has a limited size, data is removed when reading from it. It is possible that the buffer does not have enough space left when trying to write to it. If that is the case then writing is paused for a short period of time and retried for a few times after. 

Reading data from the audio source is moved into a separate thread in order to ensure reading audio data and processing it can happen concurrently.

The data written to the buffer is used by `AudioTransformer` to transform the bytes into audio bands that can be visualized.
