use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;

const WHISPER_SAMPLE_RATE: u32 = 16000;
const WHISPER_CHANNEL_COUNT: u16 = 1; // mono because whisper wants it

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("failed to get input device");

    // We'll try and use the same configuration between streams to keep it simple.
    let config: cpal::StreamConfig = cpal::StreamConfig {
        channels: WHISPER_CHANNEL_COUNT,
        sample_rate: cpal::SampleRate(WHISPER_SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Fixed(WHISPER_SAMPLE_RATE * 2), // going for a buffer spanning 2
                                                                       // seconds
    };

    let latency = 1000;

    // Create a delay in case the input and output devices aren't synced.
    let latency_samples = samples_over_a_period(&config, latency);

    // The buffer to share samples
    let ring = HeapRb::<f32>::new(latency_samples * 2);
    let ring2 = HeapRb::<f32>::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();
    let (mut producer2, mut consumer2) = ring2.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0).unwrap();
    }

    let input_stream = device.build_input_stream(
        &config,
        move |data: &[f32], _info| {
            let mut output_fell_behind = false;

            for &sample in data {
                if producer.push(sample).is_err() {
                    output_fell_behind = true;
                };
                let _ = producer2.push(sample);
            }

            if output_fell_behind {
                eprintln!("[WARN] output stream fell behind: try increasing latency");
            }
        },
        err_fn,
        None,
    )?;

    let out_device = host
        .default_output_device()
        .expect("failed to get speakers");

    let output_stream = out_device.build_output_stream(
        &config,
        move |data: &mut [f32], _| {
            let mut input_fell_behind = false;
            for sample in data {
                *sample = match consumer.pop() {
                    Some(s) => s,
                    None => {
                        input_fell_behind = true;
                        0.0
                    }
                };
            }
            if input_fell_behind {
                eprintln!("[WARN] input stream fell behind: try increasing latency");
            }
        },
        err_fn,
        None,
    )?;

    input_stream.play()?;

    output_stream.play()?;

    let tr = stt::Transcribe::new();

    loop {
        if consumer2.is_full() {
            let data: Vec<_> = consumer2.pop_iter().collect();
            let text = tr.transcribe(&data);

            eprintln!("[DEBUG] heard and transcribed: {}", text);
            if is_signal_to_start_command(&text) {
                eprintln!(
                    "[DEBUG] received signal to start recording command: {}",
                    &text
                );

                let speech_ref = Arc::new(Mutex::new(Vec::<f32>::new()));
                let speech_ref_here = Arc::clone(&speech_ref);

                let signal = Arc::new((Mutex::new(false), Condvar::new()));
                let signal_rec = Arc::clone(&signal);

                let input_stream = device.build_input_stream(
                    &config,
                    move |data: &[f32], _info| {
                        let mut s = speech_ref.lock().unwrap();
                        for &sample in data {
                            s.push(sample);
                        }

                        let is_silence = data.iter().all(|sample| sample.abs() < 0.0005);

                        if is_silence {
                            eprintln!("[INFO] silence for over a second");
                            let (lock, cvar) = &*signal;
                            let mut start = lock.lock().unwrap();
                            *start = true;
                            cvar.notify_one();
                        }
                    },
                    err_fn,
                    None,
                )?;

                input_stream.play()?;

                eprintln!("[INFO] recording...");
                std::thread::sleep(Duration::from_secs(3)); // wait at least 3 seconds?
                let (lock, cvar) = &*signal_rec;
                let mut start_guard = lock.lock().unwrap();

                while !*start_guard {
                    start_guard = cvar.wait(start_guard).unwrap();
                }

                let data = speech_ref_here.lock().unwrap();

                let text = tr.transcribe(&data);

                println!("[echo]: {text}");
            }
        }
    }
}

fn samples_over_a_period(config: &cpal::StreamConfig, period_ms: usize) -> usize {
    let latency_frames = (period_ms as f32 / 1_000.0) * config.sample_rate.0 as f32;
    latency_frames as usize * config.channels as usize
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
