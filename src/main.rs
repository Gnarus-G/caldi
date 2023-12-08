use std::sync::{Arc, Condvar, Mutex};

mod stt;

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tts::Tts;

#[derive(Parser)]
struct CLi {
    #[clap(default_value = "Caldi")]
    assistant_name: String,
}

impl CLi {
    fn waiting_mode_transcription_prompt(&self) -> String {
        format!(
            r#"[system] The user will probably say "Hey, {}", and if they don't then just repeat what they said. [user]"#,
            self.assistant_name
        )
    }

    fn is_signal_to_start_command(&self, text: &str) -> bool {
        let text = text.trim().to_lowercase();
        return text.starts_with("hey") && text.contains(&self.assistant_name.to_lowercase());
    }
}

const WHISPER_SAMPLE_RATE: u32 = 16000;
const WHISPER_CHANNEL_COUNT: u16 = 1; // mono because whisper wants it

#[derive(Debug, PartialEq)]
enum ListenState {
    Waiting,
    Listening,
    Transcribing,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = CLi::parse();
    let tts = Arc::new(Mutex::new(Tts::default()?));
    let _tts = Arc::clone(&tts);

    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("failed to get input device");

    // We'll try and use the same configuration between streams to keep it simple.
    let config: cpal::StreamConfig = cpal::StreamConfig {
        channels: WHISPER_CHANNEL_COUNT,
        sample_rate: cpal::SampleRate(WHISPER_SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Fixed(WHISPER_SAMPLE_RATE * 4), // going for a buffer spanning 3
                                                                       // seconds
    };

    let speech_audio = Arc::new(Mutex::new(Vec::<f32>::new()));
    let _speech_audio = Arc::clone(&speech_audio);

    let signal = Arc::new((Mutex::new(ListenState::Waiting), Condvar::new()));
    let _signal = Arc::clone(&signal);

    let tr = Arc::new(stt::Transcribe::new());
    let _tr = Arc::clone(&tr);

    let input_stream = device.build_input_stream(
        &config,
        move |data: &[f32], _info| {
            let mut state = signal.0.lock().unwrap();

            match *state {
                ListenState::Waiting => {
                    if is_silence(data) {
                        eprintln!("[INFO] silence detected, still waiting");
                        return;
                    }

                    let text = _tr.transcribe(data, &cli.waiting_mode_transcription_prompt());

                    eprintln!("[DEBUG] heard and transcribed: {}", text);
                    if cli.is_signal_to_start_command(&text) {
                        eprintln!(
                            "[DEBUG] received signal to start recording command: {}",
                            &text
                        );

                        eprintln!("[INFO] recording...");

                        _tts.lock()
                            .unwrap()
                            .speak("Ready!", false)
                            .expect("failed to speak");

                        *state = ListenState::Listening;
                    }
                }
                ListenState::Listening => {
                    let mut s = _speech_audio.lock().unwrap();
                    for &sample in data {
                        s.push(sample);
                    }

                    if is_silence(data) && !is_silence(&s) {
                        eprintln!("[INFO] silence detected after having spoken something");
                        *state = ListenState::Transcribing;
                        let (_, cvar) = &*signal;
                        cvar.notify_one();
                    }
                }
                ListenState::Transcribing => {
                    eprintln!("[DEBUG] noop in input_stream, currently transcribing");
                }
            }
        },
        err_fn,
        None,
    )?;

    input_stream.play()?;

    loop {
        let (_state, cvar) = &*_signal;
        let mut state = _state.lock().unwrap();

        while *state != ListenState::Transcribing {
            state = cvar.wait(state).unwrap();
        }

        input_stream.pause()?;

        let mut data = speech_audio.lock().unwrap();

        let prompt = r#"[system] Get ready. The user will pose some math problems. [user]"#;
        let text = tr.transcribe(&data, prompt);

        println!("[echo]: {text}");
        tts.lock()
            .unwrap()
            .speak(&format!("I heard you say, {text}"), false)?;

        *state = ListenState::Waiting;
        data.clear();
        input_stream.play()?;
    }
}

fn is_silence(samples: &[f32]) -> bool {
    samples.iter().all(|sample| sample.abs() < 0.0005)
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("[ERROR] an error occurred on stream: {}", err);
}
