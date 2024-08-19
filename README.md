# Caldi

Voice commanded calculator in the terminal.

## Demo

https://github.com/Gnarus-G/caldi/assets/37311893/f7f8dfcc-f058-4769-929e-f59247895ed0

## Setup Whisper Dependencies

[OpenCL](https://github.com/ggerganov/whisper.cpp/#opencl-gpu-support-via-clblast)
Assuming a non Nvidia GPU.

```sh
sudo pacman -S opencl clinfo clblast
```

[OpenBlas](https://github.com/ggerganov/whisper.cpp/#blas-cpu-support-via-openblas)

```sh
sudo pacman -S openblas
```

### To get the required Whisper models

Follow the [quickstart](https://github.com/ggerganov/whisper.cpp/#quick-start) from [whisper.cpp](https://github.com/ggerganov/whisper.cpp)

## Setup TTS

References:

- https://wiki.archlinux.org/title/Speech_dispatcher#Using_TTS_causes_the_dummy_output_module_to_speak_an_error_message

### Dependencies

```sh
sudo pacman -S speech-dispatcher festival festival-us
paru -S festival-freebsoft-utils
```

### Configure

```sh
# pick to configure for user (not system)
spd-conf
```

Find and uncomment (by removing the # from in front of it) the line:
`~/.config/speech-dispatcher/speechd.conf`

```conf
#AddModule "festival"
```

### Run the speech synthesis server (festival)

```sh
festival --server &
```

### Start Up in the background

```sh
festival --server &
sleep 1 # give festival some time to boot up
~/.cargo/bin/caldi assistant '<path to a ggml bin file>' 1> ~/caldi.log &
```

## Notice on supported systems

Only tested on Arch linux, but could work on other distros with a little more effort.

## References
- https://github.com/rhasspy/piper
- https://github.com/coqui-ai/TTS
- https://github.com/yl4579/StyleTTS2
