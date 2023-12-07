use std::sync::{Arc, Condvar, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const WHISPER_SAMPLE_RATE: u32 = 16000;
const WHISPER_CHANNEL_COUNT: u16 = 1; // mono because whisper wants it

#[derive(Debug, PartialEq)]
enum ListenState {
    Waiting,
    Listening,
    Transcribing,
}

fn main() -> Result<(), anyhow::Error> {
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

                    let text = _tr.transcribe(data);
                    eprintln!("[DEBUG] heard and transcribed: {}", text);
                    if is_signal_to_start_command(&text) {
                        eprintln!(
                            "[DEBUG] received signal to start recording command: {}",
                            &text
                        );

                        eprintln!("[INFO] recording...");

                        *state = ListenState::Listening;
                    }
                }
                ListenState::Listening => {
                    let mut s = _speech_audio.lock().unwrap();
                    for &sample in data {
                        s.push(sample);
                    }

                    if is_silence(data) && !is_silence(&s) {
                        eprintln!("[INFO] silence detected");
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

        let text = tr.transcribe(&data);

        println!("[echo]: {text}");

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

fn is_signal_to_start_command(text: &str) -> bool {
    let candidates = ["hey", "hey,caldi"];
    candidates.iter().any(|c| text.to_lowercase().contains(c))
}

mod stt {
    use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

    pub struct Transcribe {
        ctx: WhisperContext,
    }

    impl Transcribe {
        pub fn new() -> Self {
            let path_to_model = "./models/ggml-base.en.bin";

            let ctx =
                WhisperContext::new_with_params(path_to_model, WhisperContextParameters::default())
                    .expect("failed to load model");

            Self { ctx }
        }

        pub fn transcribe(&self, audio_data: &[f32]) -> String {
            let ctx = &self.ctx;

            // create a params object
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 2 });
            let prompt = r#"[system] Get ready. The user will say "Hey, Caldi", and then they will pose some math problems. [user]"#;
            let tokens = &ctx.tokenize(prompt, prompt.len()).unwrap();

            params.set_tokens(tokens);
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);

            // now we can run the model
            let mut state = ctx.create_state().expect("failed to create state");
            state.full(params, audio_data).expect("failed to run model");

            // fetch the results
            let num_segments = state
                .full_n_segments()
                .expect("failed to get number of segments");

            let mut text = String::new();

            for i in 0..num_segments {
                let segment = state
                    .full_get_segment_text(i)
                    .expect("failed to get segment");

                text.push_str(&segment);
            }

            text
        }
    }
}
