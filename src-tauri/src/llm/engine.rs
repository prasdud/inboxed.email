use anyhow::{anyhow, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex, Once};

/// Default generation parameters
const DEFAULT_MAX_TOKENS: u32 = 256;
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_TOP_P: f32 = 0.9;
const DEFAULT_CONTEXT_SIZE: u32 = 4096;

/// Global singleton for the LlamaBackend (can only be initialized once per process)
static BACKEND_INIT: Once = Once::new();
static LLAMA_BACKEND: Mutex<Option<Arc<LlamaBackend>>> = Mutex::new(None);

/// Get or initialize the global LlamaBackend
fn get_backend() -> Result<Arc<LlamaBackend>> {
    let mut init_error: Option<String> = None;

    BACKEND_INIT.call_once(|| match LlamaBackend::init() {
        Ok(backend) => {
            let mut guard = LLAMA_BACKEND.lock().unwrap();
            *guard = Some(Arc::new(backend));
        }
        Err(e) => {
            init_error = Some(format!("Failed to initialize LlamaBackend: {:?}", e));
        }
    });

    if let Some(err) = init_error {
        return Err(anyhow!(err));
    }

    let guard = LLAMA_BACKEND.lock().unwrap();
    guard
        .clone()
        .ok_or_else(|| anyhow!("LlamaBackend not initialized"))
}

/// LLM Engine for text generation with Metal acceleration
pub struct LlmEngine {
    model: Arc<LlamaModel>,
    backend: Arc<LlamaBackend>,
}

/// Generation parameters
#[derive(Debug, Clone)]
pub struct GenerationParams {
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub stop_sequences: Vec<String>,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
            top_p: DEFAULT_TOP_P,
            stop_sequences: vec![],
        }
    }
}

impl LlmEngine {
    /// Create a new LlmEngine by loading a model from the given path
    /// Automatically uses Metal acceleration on macOS
    pub fn new(model_path: &Path) -> Result<Self> {
        // Get the singleton backend (initialized once per process)
        let backend = get_backend()?;

        // Configure model parameters
        // Metal/GPU acceleration is enabled by default on macOS
        let model_params = LlamaModelParams::default();

        // Load the model
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
            .map_err(|e| anyhow!("Failed to load model: {:?}", e))?;

        Ok(Self {
            model: Arc::new(model),
            backend,
        })
    }

    /// Generate text with streaming callback
    pub fn generate_stream<F>(
        &self,
        prompt: &str,
        params: &GenerationParams,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(&str),
    {
        // Create context for this generation
        let ctx_params =
            LlamaContextParams::default().with_n_ctx(NonZeroU32::new(DEFAULT_CONTEXT_SIZE));

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| anyhow!("Failed to create context: {:?}", e))?;

        // Tokenize the prompt
        let tokens = self
            .model
            .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
            .map_err(|e| anyhow!("Failed to tokenize: {:?}", e))?;

        // Create batch and add prompt tokens
        let mut batch = LlamaBatch::new(DEFAULT_CONTEXT_SIZE as usize, 1);

        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add(*token, i as i32, &[0], is_last)
                .map_err(|e| anyhow!("Failed to add token to batch: {:?}", e))?;
        }

        // Process the prompt
        ctx.decode(&mut batch)
            .map_err(|e| anyhow!("Failed to decode prompt: {:?}", e))?;

        // Create sampler chain with temperature and top_p
        let seed = rand::random::<u32>();
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(params.temperature),
            LlamaSampler::top_p(params.top_p, 1),
            LlamaSampler::dist(seed),
        ]);

        // Generate tokens
        let mut output = String::new();
        let mut n_cur = tokens.len();

        for _ in 0..params.max_tokens {
            // Sample the next token
            let new_token = sampler.sample(&ctx, (batch.n_tokens() - 1) as i32);

            // Accept the token
            sampler.accept(new_token);

            // Check for end of sequence
            if self.model.is_eog_token(new_token) {
                break;
            }

            // Decode token to string
            let token_str = self
                .model
                .token_to_str(new_token, llama_cpp_2::model::Special::Tokenize)
                .map_err(|e| anyhow!("Failed to decode token: {:?}", e))?;

            // Check for stop sequences
            let should_stop = params
                .stop_sequences
                .iter()
                .any(|stop| output.ends_with(stop) || token_str.contains(stop.as_str()));

            if should_stop {
                break;
            }

            // Emit the token
            on_token(&token_str);
            output.push_str(&token_str);

            // Prepare next batch
            batch.clear();
            batch
                .add(new_token, n_cur as i32, &[0], true)
                .map_err(|e| anyhow!("Failed to add token: {:?}", e))?;

            n_cur += 1;

            // Decode next token
            ctx.decode(&mut batch)
                .map_err(|e| anyhow!("Failed to decode: {:?}", e))?;
        }

        Ok(output.trim().to_string())
    }

    /// Generate text without streaming
    pub fn generate(&self, prompt: &str, params: &GenerationParams) -> Result<String> {
        self.generate_stream(prompt, params, |_| {})
    }

    /// Generate with default parameters
    pub fn generate_simple(&self, prompt: &str) -> Result<String> {
        self.generate(prompt, &GenerationParams::default())
    }
}

// Ensure thread-safety
unsafe impl Send for LlmEngine {}
unsafe impl Sync for LlmEngine {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.max_tokens, DEFAULT_MAX_TOKENS);
        assert!((params.temperature - DEFAULT_TEMPERATURE).abs() < f32::EPSILON);
    }
}
