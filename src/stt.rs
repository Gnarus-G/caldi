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

    pub fn transcribe(&self, audio_data: &[f32], prompt: &str) -> String {
        let ctx = &self.ctx;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        let tokens = &ctx.tokenize(prompt, prompt.len()).unwrap();
        params.set_tokens(tokens);

        params.set_n_threads(1);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_no_context(true);
        params.set_suppress_non_speech_tokens(true);

        // now we can run the model
        let mut state = ctx.create_state().expect("failed to create state");
        state.full(params, audio_data).expect("failed to run model");

        // fetch the results
        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");

        // average english word length is 5.1 characters which we round up to 6
        let mut text = String::with_capacity(6 * num_segments as usize);

        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .expect("failed to get segment");

            text.push_str(&segment);
        }

        text
    }
}
