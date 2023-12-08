# Caldi

Voice commanded calculator in the terminal.

## Setup Whisper

[OpenCL](https://github.com/ggerganov/whisper.cpp/#opencl-gpu-support-via-clblast)

```sh
sudo pacman -S opencl clinfo clblast
```

[OpenBlas](https://github.com/ggerganov/whisper.cpp/#blas-cpu-support-via-openblas)

```sh
sudo pacman -S openblas
```

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
