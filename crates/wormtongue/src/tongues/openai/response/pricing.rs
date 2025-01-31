use crate::tongues::openai::response::TokenCount;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum OpenAIApiPricing {
    Gpt4o {
        input: f64,
        cached_input: f64,
        output: f64,
    },
    Gpt4oAudioPreview {
        input: f64,
        cached_input: f64,
        output: f64,
    },
    Gpt4oRealtimePreview {
        input: f64,
        cached_input: f64,
        output: f64,
    },
    Gpt4oMini {
        input: f64,
        cached_input: f64,
        output: f64,
    },
    Gpt4oMiniRealtimePreview {
        input: f64,
        cached_input: f64,
        output: f64,
    },
    Gpt4oMiniAudioPreview {
        input: f64,
        cached_input: f64,
        output: f64,
    },
    o1 {
        input: f64,
        cached_input: f64,
        output: f64,
    },
    o1Mini {
        input: f64,
        cached_input: f64,
        output: f64,
    },
    Other {
        input: f64,
        output: f64,
    },
}

impl OpenAIApiPricing {
    // price per million tokens
    pub fn from_model(model: &str) -> Self {
        match model {
            m if m.contains("gpt-4o-audio") => OpenAIApiPricing::Gpt4oAudioPreview {
                input: 2.50,
                cached_input: 0.0,
                output: 10.00,
            },
            m if m.contains("gpt-4o-realtime") => OpenAIApiPricing::Gpt4oRealtimePreview {
                input: 5.00,
                cached_input: 2.50,
                output: 20.00,
            },
            m if m.contains("gpt-4o-mini-audio") => OpenAIApiPricing::Gpt4oMiniAudioPreview {
                input: 0.15,
                cached_input: 0.0,
                output: 0.60,
            },
            m if m.contains("gpt-4o-mini-realtime") => OpenAIApiPricing::Gpt4oMiniRealtimePreview {
                input: 0.60,
                cached_input: 0.30,
                output: 2.40,
            },

            m if m.contains("gpt-4o-mini") => OpenAIApiPricing::Gpt4oMini {
                input: 0.15,
                cached_input: 0.075,
                output: 0.60,
            },
            m if m.contains("gpt-4o") => OpenAIApiPricing::Gpt4o {
                input: 2.50,
                cached_input: 1.25,
                output: 10.00,
            },
            m if m.contains("o1-mini") => OpenAIApiPricing::o1Mini {
                input: 3.00,
                cached_input: 1.50,
                output: 12.00,
            },
            m if m.contains("o1") => OpenAIApiPricing::o1 {
                input: 15.00,
                cached_input: 7.50,
                output: 60.00,
            },
            _ => OpenAIApiPricing::Other {
                input: 10.00,
                output: 30.00,
            },
        }
    }

    pub fn calculate_cost(&self, tokens: &TokenCount) -> f64 {
        match self {
            OpenAIApiPricing::Other { input, output } => {
                let input_cost = (tokens.input_tokens as f64 / 1_000_000.0) * input;
                let output_cost = (tokens.output_tokens as f64 / 1_000_000.0) * output;
                input_cost + output_cost
            }
            OpenAIApiPricing::Gpt4o {
                input,
                cached_input,
                output,
            }
            | OpenAIApiPricing::Gpt4oAudioPreview {
                input,
                cached_input,
                output,
            }
            | OpenAIApiPricing::Gpt4oRealtimePreview {
                input,
                cached_input,
                output,
            }
            | OpenAIApiPricing::Gpt4oMini {
                input,
                cached_input,
                output,
            }
            | OpenAIApiPricing::Gpt4oMiniRealtimePreview {
                input,
                cached_input,
                output,
            }
            | OpenAIApiPricing::Gpt4oMiniAudioPreview {
                input,
                cached_input,
                output,
            }
            | OpenAIApiPricing::o1 {
                input,
                cached_input,
                output,
            }
            | OpenAIApiPricing::o1Mini {
                input,
                cached_input,
                output,
            } => {
                let regular_input_cost =
                    ((tokens.input_tokens - tokens.cached_tokens) as f64 / 1_000_000.0) * input;
                let cached_input_cost = (tokens.cached_tokens as f64 / 1_000_000.0) * cached_input;
                let output_cost = (tokens.output_tokens as f64 / 1_000_000.0) * output;
                regular_input_cost + cached_input_cost + output_cost
            }
        }
    }
}
