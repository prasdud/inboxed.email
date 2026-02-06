use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use super::engine::{GenerationParams, LlmEngine};

/// AI-powered email summarizer using local LLM
pub struct Summarizer {
    engine: Option<Arc<LlmEngine>>,
    model_type: ModelType,
}

/// Different model types require different prompt formats
#[derive(Debug, Clone, Copy, Default)]
pub enum ModelType {
    #[default]
    LFM25,      // LiquidAI LFM2.5 series
    Qwen25,     // Qwen 2.5 series
    Unknown,    // Generic ChatML
}

impl Summarizer {
    /// Create a new Summarizer without a loaded model
    /// Call `load_model` to initialize the LLM
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: None,
            model_type: ModelType::default(),
        })
    }

    /// Load an LLM model from the given path
    pub fn load_model(&mut self, model_path: &Path) -> Result<()> {
        let engine = LlmEngine::new(model_path)?;
        self.engine = Some(Arc::new(engine));

        // Detect model type from filename
        let filename = model_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        self.model_type = if filename.contains("lfm") {
            ModelType::LFM25
        } else if filename.contains("qwen") {
            ModelType::Qwen25
        } else {
            ModelType::Unknown
        };

        Ok(())
    }

    /// Check if a model is loaded
    pub fn is_model_loaded(&self) -> bool {
        self.engine.is_some()
    }

    /// Get the LLM engine (for streaming operations)
    pub fn engine(&self) -> Option<Arc<LlmEngine>> {
        self.engine.clone()
    }

    /// Format a prompt for the loaded model type
    fn format_prompt(&self, system: &str, user: &str) -> String {
        match self.model_type {
            ModelType::LFM25 => {
                // LFM2.5 uses <|startoftext|> and ChatML-like format
                format!(
                    "<|startoftext|><|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n"
                )
            }
            ModelType::Qwen25 => {
                // Qwen 2.5 uses standard ChatML
                format!(
                    "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n"
                )
            }
            ModelType::Unknown => {
                // Generic ChatML format
                format!(
                    "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n"
                )
            }
        }
    }

    /// Get stop sequences for the model
    fn get_stop_sequences(&self) -> Vec<String> {
        vec![
            "<|im_end|>".to_string(),
            "<|endoftext|>".to_string(),
            "\n\n\n".to_string(),
        ]
    }

    /// Determine summary parameters based on email length
    fn get_summary_params(word_count: usize) -> (u32, &'static str) {
        // Returns (max_tokens, instruction)
        match word_count {
            0..=50 => (50, "Summarize this short email in 1 sentence, capturing the key point."),
            51..=150 => (80, "Summarize this email in 1-2 sentences, capturing the main point and any action needed."),
            151..=400 => (120, "Summarize this email in 2-3 sentences, covering the main points and any required actions."),
            401..=800 => (180, "Summarize this email in 3-4 sentences, ensuring all important points and action items are captured."),
            _ => (250, "Provide a comprehensive summary of this long email in 4-5 sentences. Capture all key points, decisions, action items, and important details without losing critical information."),
        }
    }

    /// Summarize email content using LLM
    pub fn summarize_email(
        &self,
        subject: &str,
        from: &str,
        body: &str,
    ) -> Result<String> {
        let body_text = Self::strip_html(body);
        let word_count = body_text.split_whitespace().count();

        // Adjust context size based on email length
        let max_body_chars = if word_count > 800 { 4000 } else { 2000 };
        let body_preview = Self::truncate_text(&body_text, max_body_chars);

        if let Some(engine) = &self.engine {
            let (max_tokens, instruction) = Self::get_summary_params(word_count);

            let system = format!(
                "You are a helpful email assistant. {} Do not miss any important information.",
                instruction
            );
            let user = format!(
                "Summarize this email:\n\nFrom: {from}\nSubject: {subject}\n\n{body_preview}"
            );

            let prompt = self.format_prompt(&system, &user);

            let params = GenerationParams {
                max_tokens,
                temperature: 0.3,
                stop_sequences: self.get_stop_sequences(),
                ..Default::default()
            };

            engine.generate(&prompt, &params)
        } else {
            // Fallback to simple extraction if no model loaded
            Self::simple_summary(subject, from, &body_text, word_count)
        }
    }

    /// Summarize with streaming callback
    pub fn summarize_email_stream<F>(
        &self,
        subject: &str,
        from: &str,
        body: &str,
        on_token: F,
    ) -> Result<String>
    where
        F: FnMut(&str),
    {
        let body_text = Self::strip_html(body);
        let word_count = body_text.split_whitespace().count();

        // Adjust context size based on email length
        let max_body_chars = if word_count > 800 { 4000 } else { 2000 };
        let body_preview = Self::truncate_text(&body_text, max_body_chars);

        if let Some(engine) = &self.engine {
            let (max_tokens, instruction) = Self::get_summary_params(word_count);

            let system = format!(
                "You are a helpful email assistant. {} Do not miss any important information.",
                instruction
            );
            let user = format!(
                "Summarize this email:\n\nFrom: {from}\nSubject: {subject}\n\n{body_preview}"
            );

            let prompt = self.format_prompt(&system, &user);

            let params = GenerationParams {
                max_tokens,
                temperature: 0.3,
                stop_sequences: self.get_stop_sequences(),
                ..Default::default()
            };

            engine.generate_stream(&prompt, &params, on_token)
        } else {
            // Fallback
            let summary = Self::simple_summary(subject, from, &body_text, word_count)?;
            Ok(summary)
        }
    }

    /// Generate AI insights about the email
    pub fn generate_insights(&self, subject: &str, body: &str) -> Result<Vec<String>> {
        let body_text = Self::strip_html(body);
        let body_preview = Self::truncate_text(&body_text, 1500);

        if let Some(engine) = &self.engine {
            let system = "You are an email analysis assistant. List 1-3 key insights about emails. Each insight should be one short sentence. Format: one insight per line starting with an emoji.";
            let user = format!("Analyze this email:\n\nSubject: {subject}\n\n{body_preview}");

            let prompt = self.format_prompt(system, &user);

            let params = GenerationParams {
                max_tokens: 150,
                temperature: 0.3,
                stop_sequences: self.get_stop_sequences(),
                ..Default::default()
            };

            let response = engine.generate(&prompt, &params)?;

            // Parse insights from response (one per line)
            let insights: Vec<String> = response
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .take(5)
                .collect();

            if insights.is_empty() {
                Ok(vec!["ℹ️ Email analyzed successfully".to_string()])
            } else {
                Ok(insights)
            }
        } else {
            // Fallback to keyword-based insights
            Self::simple_insights(subject, &body_text)
        }
    }

    /// Classify email priority using LLM
    pub fn classify_priority(&self, subject: &str, body: &str) -> Result<String> {
        let body_text = Self::strip_html(body);
        let body_preview = Self::truncate_text(&body_text, 1000);

        if let Some(engine) = &self.engine {
            let system = "You are an email priority classifier. Respond with exactly one word: HIGH, MEDIUM, or LOW.\nHIGH = urgent, time-sensitive, requires immediate action\nMEDIUM = important but not urgent, needs attention soon\nLOW = informational, no action required";
            let user = format!("Classify this email's priority:\n\nSubject: {subject}\n\n{body_preview}");

            let prompt = self.format_prompt(system, &user);

            let params = GenerationParams {
                max_tokens: 10,
                temperature: 0.1,
                stop_sequences: self.get_stop_sequences(),
                ..Default::default()
            };

            let response = engine.generate(&prompt, &params)?;
            let priority = response.trim().to_uppercase();

            // Validate response
            match priority.as_str() {
                "HIGH" | "MEDIUM" | "LOW" => Ok(priority),
                _ => {
                    // Try to extract valid priority from response
                    if priority.contains("HIGH") {
                        Ok("HIGH".to_string())
                    } else if priority.contains("MEDIUM") {
                        Ok("MEDIUM".to_string())
                    } else {
                        Ok("LOW".to_string())
                    }
                }
            }
        } else {
            // Fallback to simple classification
            Self::simple_priority(subject, &body_text)
        }
    }

    /// Strip HTML tags from content
    fn strip_html(html: &str) -> String {
        let result = html
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("</p>", "\n\n")
            .replace("</div>", "\n");

        // Remove all HTML tags
        let mut in_tag = false;
        let mut cleaned = String::new();

        for ch in result.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
            } else if !in_tag {
                cleaned.push(ch);
            }
        }

        // Clean up whitespace
        cleaned
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Truncate text to a maximum number of characters
    fn truncate_text(text: &str, max_chars: usize) -> String {
        if text.len() <= max_chars {
            text.to_string()
        } else {
            let truncated: String = text.chars().take(max_chars).collect();
            format!("{}...", truncated)
        }
    }

    /// Simple fallback summary (used when no LLM is loaded)
    fn simple_summary(subject: &str, from: &str, body_text: &str, word_count: usize) -> Result<String> {
        let words: Vec<&str> = body_text.split_whitespace().collect();
        let sender = from.split('<').next().unwrap_or(from).trim();

        // Adjust preview length based on email size
        let preview_words = match word_count {
            0..=50 => word_count,      // Show all for very short
            51..=150 => 40,            // Short preview
            151..=400 => 60,           // Medium preview
            401..=800 => 80,           // Longer preview
            _ => 100,                  // Comprehensive preview for long emails
        };

        let summary = if words.len() > preview_words {
            let preview: Vec<&str> = words.iter().take(preview_words).copied().collect();
            format!(
                "Email from {} regarding \"{}\": {}...",
                sender,
                subject,
                preview.join(" ")
            )
        } else {
            format!(
                "Email from {} regarding \"{}\": {}",
                sender,
                subject,
                body_text
            )
        };

        Ok(summary)
    }

    /// Simple fallback insights (keyword-based)
    fn simple_insights(subject: &str, body_text: &str) -> Result<Vec<String>> {
        let mut insights = Vec::new();
        let combined = format!("{} {}", subject, body_text).to_lowercase();

        if combined.contains("urgent") || combined.contains("asap") {
            insights.push("⚡ Urgent: Requires immediate attention".to_string());
        }

        if combined.contains("meeting")
            || combined.contains("call")
            || combined.contains("schedule")
        {
            insights.push("📅 Action: Schedule or attend meeting".to_string());
        }

        if combined.contains("deadline") || combined.contains("due date") {
            insights.push("⏰ Deadline: Time-sensitive task".to_string());
        }

        if combined.contains('?') {
            insights.push("❓ Requires response: Questions asked".to_string());
        }

        if combined.contains("invoice")
            || combined.contains("payment")
            || combined.contains('$')
        {
            insights.push("💰 Financial: Payment or invoice related".to_string());
        }

        if insights.is_empty() {
            insights.push("ℹ️ Informational: No immediate action required".to_string());
        }

        Ok(insights)
    }

    /// Simple fallback priority classification
    fn simple_priority(subject: &str, body_text: &str) -> Result<String> {
        let combined = format!("{} {}", subject, body_text).to_lowercase();

        if combined.contains("urgent")
            || combined.contains("asap")
            || combined.contains("critical")
            || combined.contains("emergency")
        {
            return Ok("HIGH".to_string());
        }

        if combined.contains("important")
            || combined.contains("deadline")
            || combined.contains("meeting")
            || combined.contains("action required")
        {
            return Ok("MEDIUM".to_string());
        }

        Ok("LOW".to_string())
    }

    /// Generate a conversational chat response
    pub fn chat(
        &self,
        user_message: &str,
        email_context: Option<&str>,
    ) -> Result<String> {
        if let Some(engine) = &self.engine {
            let system = if email_context.is_some() {
                "You are an intelligent email assistant for Inboxed. Help users understand their emails. Be concise and conversational. Only reference information from the provided context."
            } else {
                "You are an intelligent email assistant for Inboxed. Be helpful and concise."
            };

            let user = match email_context {
                Some(ctx) => format!("Email context:\n{}\n\nUser: {}", ctx, user_message),
                None => user_message.to_string(),
            };

            let prompt = self.format_prompt(system, &user);
            let params = GenerationParams {
                max_tokens: 300,
                temperature: 0.7,
                stop_sequences: self.get_stop_sequences(),
                ..Default::default()
            };

            engine.generate(&prompt, &params)
        } else {
            // Fallback when no model loaded
            Ok(Self::fallback_chat_response(email_context))
        }
    }

    /// Fallback response when LLM is not available
    fn fallback_chat_response(email_context: Option<&str>) -> String {
        if email_context.is_some() {
            "I found some relevant emails for you. Please note that the AI model isn't loaded, so I can't provide a detailed analysis. Check the email list for details.".to_string()
        } else {
            "I'm your email assistant! I can help you find and understand your emails. Try asking about today's emails, important messages, or search for specific topics.".to_string()
        }
    }
}

impl Default for Summarizer {
    fn default() -> Self {
        Self::new().expect("Failed to create Summarizer")
    }
}
