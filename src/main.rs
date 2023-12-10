use std::{
    io::{stdin, stdout, Write},
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
};

mod calc;
mod stt;

use calc::eval;
use clap::{Args, Parser, Subcommand};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{LocalRb, Rb};
use tts::Tts;

#[derive(Parser)]
struct CLi {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Assistant(AssistantInterface),
}

#[derive(Args)]
struct AssistantInterface {
    /// Path to a ggml bin model file
    language_model: PathBuf,

    /// What the assistant responds to
    #[clap(long = "name", default_value = "Caldi")]
    assistant_name: String,
}

impl AssistantInterface {
    const WHISPER_SAMPLE_RATE: u32 = 16000;
    const WHISPER_CHANNEL_COUNT: u16 = 1; // mono because whisper wants it

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

    fn handle(self) -> anyhow::Result<()> {
        let mut tts = Tts::default()?;
        tts.speak("Welcome back!", false)?;

        let tts = Arc::new(Mutex::new(tts));
        let _tts = Arc::clone(&tts);

        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .expect("failed to get input device");

        let audio_input_buffer_size = Self::WHISPER_SAMPLE_RATE * 2; // going for a buffer spanning 2 seconds

        // We'll try and use the same configuration between streams to keep it simple.
        let config: cpal::StreamConfig = cpal::StreamConfig {
            channels: Self::WHISPER_CHANNEL_COUNT,
            sample_rate: cpal::SampleRate(Self::WHISPER_SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Fixed(audio_input_buffer_size),
        };

        let mut waiting_audio = LocalRb::new(audio_input_buffer_size as usize * 2);

        let speech_audio = Arc::new(Mutex::new(Vec::<f32>::new()));
        let _speech_audio = Arc::clone(&speech_audio);

        let signal = Arc::new((Mutex::new(ListenState::Waiting), Condvar::new()));
        let _signal = Arc::clone(&signal);

        let tr = Arc::new(stt::Transcribe::new(
            self.language_model
                .to_str()
                .expect("received an invalid path for the language_model file"),
        ));
        let _tr = Arc::clone(&tr);

        let input_stream = device.build_input_stream(
            &config,
            move |data: &[f32], _info| {
                let mut state = signal.0.lock().unwrap();

                match *state {
                    ListenState::Waiting => {
                        waiting_audio.push_slice_overwrite(data);

                        let (first, second) = waiting_audio.as_slices();
                        let data = &[first, second].concat();

                        if is_silence(data) {
                            eprintln!("[INFO] silence detected, still waiting");
                            return;
                        }

                        let text = _tr.transcribe(data, &self.waiting_mode_transcription_prompt());

                        eprintln!("[DEBUG] heard and transcribed: {}", text);
                        if self.is_signal_to_start_command(&text) {
                            eprintln!(
                                "[DEBUG] received signal to start recording command: {}",
                                &text
                            );

                            eprintln!("[INFO] recording...");

                            _tts.lock()
                                .unwrap()
                                .speak("Ready!", false)
                                .expect("failed to speak");

                            std::thread::sleep(std::time::Duration::from_secs(1));
                            *state = ListenState::PreListening;
                            waiting_audio.clear();
                        } else if !text.to_lowercase().trim_start().starts_with("hey") {
                            waiting_audio.clear();
                        }
                    }
                    ListenState::PreListening => {
                        // We just want to prevent any overlap from the tail of the
                        // waiting phase that would pollute the start the Listening
                        // causing the Listening phase to end early with nonsense in it
                        *state = ListenState::Listening;
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
            let answer = eval(&text);

            println!("[problem]: {text}");

            println!("[answer]: {answer:?}");

            tts.lock().unwrap().speak(
                answer
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "Sorry I couldn't evaluate that problem".to_string()),
                false,
            )?;

            *state = ListenState::Waiting;
            data.clear();
            input_stream.play()?;
        }
    }
}

#[derive(Debug, PartialEq)]
enum ListenState {
    Waiting,
    PreListening,
    Listening,
    Transcribing,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = CLi::parse();

    match cli.command {
        Some(Command::Assistant(a)) => a.handle()?,
        None => {
            print!(":> ");
            stdout().flush()?;
            for _line in stdin().lines() {
                let line = _line?;

                let answer = eval(&line);

                answer
                    .map(|ans| {
                        println!("{ans}");
                    })
                    .unwrap_or_else(|| {
                        println!("Something went wrong");
                    });

                print!(":> ");
                stdout().flush()?;
            }
        }
    }

    return Ok(());
}

fn is_silence(samples: &[f32]) -> bool {
    !samples.is_empty() && samples.iter().all(|sample| sample.abs() < 0.0005)
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("[ERROR] an error occurred on stream: {}", err);
}
