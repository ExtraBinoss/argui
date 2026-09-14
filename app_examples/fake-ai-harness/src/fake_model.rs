use std::time::Duration;

const BILLION: u128 = 1_000_000_000;

pub const SOURCE_TITLE: &str = "Wikipedia · Large language model";
pub const SOURCE_URL: &str = "https://en.wikipedia.org/wiki/Large_language_model";

// An original, compact adaptation of the linked article. It is bundled so the
// stress run stays deterministic and does not measure network latency.
const ARTICLE: &str = r#"Large language models are machine learning systems built to process and generate natural language. They learn statistical relationships from large text collections and can apply those patterns to writing, summarization, translation, classification, question answering, and software assistance. Modern chatbots often place such a model inside a larger product that adds instructions, search, tools, memory, safety checks, and a user interface.

Most prominent large language models use the transformer architecture. A transformer represents text as tokens, turns those tokens into vectors, and repeatedly mixes information through attention and feed-forward layers. Attention lets the network weigh relationships between positions in the current context. Training can be parallelized efficiently because the architecture does not need to process every token strictly one after another in the way older recurrent networks did.

Training usually begins with a prediction objective. The model receives part of a sequence and learns to estimate missing or subsequent tokens. Repeating this process over a broad corpus adjusts billions of numerical parameters. Scale alone does not determine quality: the selection and cleaning of data, the balance of domains and languages, the tokenizer, the optimization recipe, and the available computing budget all influence the result.

After initial training, developers often adapt a base model for conversation or a specialized domain. Supervised examples can demonstrate desired responses. Preference learning can rank alternatives and encourage answers that people judge useful. Tool use can connect the model to calculators, code execution, databases, or retrieval systems. These additions change the behavior of the complete application even when its underlying language model stays the same.

At inference time, the model assigns probabilities to possible next tokens. A decoding rule selects one token, appends it to the context, and repeats the calculation. Temperature and sampling parameters affect how concentrated or varied the choices are. Streaming interfaces display generated tokens before the whole answer is complete, reducing perceived latency and allowing a person to start reading while computation continues.

Context length limits how much text the model can inspect in one request. Longer contexts can hold documents, prior conversation, or tool results, but they also require memory and computation. Systems may cache attention state, batch requests, quantize weights, or distribute layers across processors. Production serving therefore combines model quality with throughput, time to first token, tokens per second, memory use, and operating cost.

Evaluation is difficult because language tasks are diverse. Benchmarks may test factual recall, reasoning, coding, translation, instruction following, safety, or calibration. A high score on one collection does not guarantee reliability in an unfamiliar setting. Test data can leak into training corpora, and small wording changes can affect results. Useful evaluation includes representative tasks, human review, adversarial cases, latency measurements, and monitoring after deployment.

Large language models can produce fluent statements that are inaccurate. They can inherit bias from data, follow ambiguous instructions, expose private material, or be manipulated by hostile input. Retrieval and citations may help users inspect evidence, but they do not automatically make every claim correct. Applications still need access control, clear product boundaries, validation for consequential actions, and a way for people to recover from errors.

Research continues beyond a single architecture or training method. Mixture-of-experts systems activate only part of a model for each token. Smaller models can be distilled or trained for constrained devices. Multimodal models combine language with images, audio, or video. Other sequence architectures explore different ways to retain long-range information. Open and proprietary projects make different tradeoffs in weights, data disclosure, licensing, hardware support, and deployment.

This local Argui demonstration does not run a neural network. It replays this adapted article through the same kind of incremental delivery contract used by a real provider. The clock releases exactly the number of tokens due at the configured rate. Presentation updates collect those tokens into frame-sized batches, while the virtual list keeps only visible message chunks mounted. That isolates the interface workload and makes the result repeatable across native and WebAssembly builds."#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenBatch {
    pub text: String,
    pub tokens: usize,
}

#[derive(Clone, Debug)]
pub struct FakeStream {
    rate: u32,
    total: usize,
    emitted: usize,
    pieces: Vec<String>,
}

impl FakeStream {
    #[must_use]
    pub fn new(rate: u32, total: usize, prompt: &str) -> Self {
        let mut pieces = tokenize(&format!(
            "You asked: {prompt}\n\nHere is a locally bundled, adapted overview of large language models.\n\n{ARTICLE}"
        ));
        pieces.push(format!(
            "\n\nThe deterministic source now repeats to sustain the {total}-token benchmark.\n\n"
        ));
        Self {
            rate,
            total,
            emitted: 0,
            pieces,
        }
    }

    #[must_use]
    pub const fn rate(&self) -> u32 {
        self.rate
    }

    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    #[must_use]
    pub const fn emitted(&self) -> usize {
        self.emitted
    }

    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.emitted == self.total
    }

    pub fn take_due(&mut self, elapsed: Duration) -> TokenBatch {
        let scheduled = elapsed.as_nanos().saturating_mul(u128::from(self.rate)) / BILLION;
        let due = usize::try_from(scheduled)
            .unwrap_or(usize::MAX)
            .min(self.total)
            .saturating_sub(self.emitted);
        let start = self.emitted;
        self.emitted += due;

        let mut text = String::with_capacity(due.saturating_mul(7));
        for index in start..self.emitted {
            text.push_str(&self.pieces[index % self.pieces.len()]);
        }
        TokenBatch { text, tokens: due }
    }
}

fn tokenize(value: &str) -> Vec<String> {
    value
        .split_inclusive(char::is_whitespace)
        .filter(|piece| !piece.is_empty())
        .map(str::to_owned)
        .collect()
}
