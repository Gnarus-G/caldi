# Caldi

Voice commanded calculator in the terminal.

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
