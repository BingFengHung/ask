use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;
use crate::env_detector::EnvironmentContext;

#[async_trait::async_trait]
pub trait LlmProvider {
    fn provider_name(&self) -> &'static str;
    async fn is_available(&self) -> bool;
    async fn generate(&self, prompt: &str, ctx: &EnvironmentContext) -> Result<String>;
}

// -----------------------------------------------------------------------------
// 1. Antigravity CLI (agy) Provider
// -----------------------------------------------------------------------------
pub struct AgyProvider;

#[async_trait::async_trait]
impl LlmProvider for AgyProvider {
    fn provider_name(&self) -> &'static str {
        "Antigravity CLI (agy)"
    }

    async fn is_available(&self) -> bool {
        Command::new("agy")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    async fn generate(&self, prompt: &str, ctx: &EnvironmentContext) -> Result<String> {
        let full_prompt = build_system_prompt(prompt, ctx);
        let output = Command::new("agy")
            .arg("-p")
            .arg(&full_prompt)
            .output()?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("agy command failed: {}", err_msg));
        }

        let raw_response = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(clean_llm_response(&raw_response))
    }
}

// -----------------------------------------------------------------------------
// 2. Local Ollama Provider
// -----------------------------------------------------------------------------
pub struct OllamaProvider {
    pub host: String,
    pub model: String,
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self {
            host: env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:latest".to_string()),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    fn provider_name(&self) -> &'static str {
        "Local Ollama"
    }

    async fn is_available(&self) -> bool {
        let client = reqwest::Client::new();
        let url = format!("{}/api/tags", self.host);
        client
            .get(&url)
            .timeout(std::time::Duration::from_millis(800))
            .send()
            .await
            .map(|res| res.status().is_success())
            .unwrap_or(false)
    }

    async fn generate(&self, prompt: &str, ctx: &EnvironmentContext) -> Result<String> {
        let client = reqwest::Client::new();
        let full_prompt = build_system_prompt(prompt, ctx);
        let payload = OllamaRequest {
            model: self.model.clone(),
            prompt: full_prompt,
            stream: false,
        };

        let url = format!("{}/api/generate", self.host);
        let res = client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Ollama API returned HTTP {}", res.status()));
        }

        let body: OllamaResponse = res.json().await?;
        Ok(clean_llm_response(&body.response))
    }
}

// -----------------------------------------------------------------------------
// 3. Cloud API Provider (OpenAI / DeepSeek)
// -----------------------------------------------------------------------------
pub struct CloudApiProvider;

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Deserialize)]
struct ChatMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[async_trait::async_trait]
impl LlmProvider for CloudApiProvider {
    fn provider_name(&self) -> &'static str {
        "Cloud API (OpenAI / DeepSeek)"
    }

    async fn is_available(&self) -> bool {
        env::var("OPENAI_API_KEY").is_ok() || env::var("DEEPSEEK_API_KEY").is_ok()
    }

    async fn generate(&self, prompt: &str, ctx: &EnvironmentContext) -> Result<String> {
        let (api_key, api_url, model) = if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
            (key, "https://api.deepseek.com/chat/completions", "deepseek-chat")
        } else if let Ok(key) = env::var("OPENAI_API_KEY") {
            (key, "https://api.openai.com/v1/chat/completions", "gpt-4o-mini")
        } else {
            return Err(anyhow!("No Cloud API Key found in environment variables"));
        };

        let client = reqwest::Client::new();
        let payload = ChatCompletionRequest {
            model: model.to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "You are a command-line assistant. Convert user instructions into exact shell commands for OS: {}, Shell: {}. Return ONLY the command.",
                        ctx.os.name(),
                        ctx.shell.name()
                    ),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        };

        let res = client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await?;

        let body: ChatCompletionResponse = res.json().await?;
        if let Some(choice) = body.choices.first() {
            Ok(clean_llm_response(&choice.message.content))
        } else {
            Err(anyhow!("Empty completion response from Cloud API"))
        }
    }
}

// -----------------------------------------------------------------------------
// Helper functions
// -----------------------------------------------------------------------------
fn build_system_prompt(user_query: &str, ctx: &EnvironmentContext) -> String {
    format!(
        "Target OS: {}\nTarget Shell: {}\nCurrent Working Directory: {}\nUser Request: {}\n\nInstruction: Provide ONLY the exact single-line or multi-line shell command that satisfies the request. Do NOT explain. Do NOT use markdown code block formatting ```. Output raw command text only.",
        ctx.os.name(),
        ctx.shell.name(),
        ctx.cwd,
        user_query
    )
}

pub fn clean_llm_response(raw: &str) -> String {
    let trimmed = raw.trim();
    let lines: Vec<&str> = trimmed.lines().collect();

    let mut result_lines = Vec::new();
    for line in lines {
        let l = line.trim();
        if l.starts_with("```") || l.ends_with("```") {
            continue;
        }
        result_lines.push(l);
    }

    result_lines.join("\n").trim().to_string()
}

pub async fn query_best_available_provider(
    prompt: &str,
    ctx: &EnvironmentContext,
) -> Result<(String, &'static str)> {
    let agy = AgyProvider;
    if agy.is_available().await {
        if let Ok(cmd) = agy.generate(prompt, ctx).await {
            return Ok((cmd, agy.provider_name()));
        }
    }

    let ollama = OllamaProvider::default();
    if ollama.is_available().await {
        if let Ok(cmd) = ollama.generate(prompt, ctx).await {
            return Ok((cmd, ollama.provider_name()));
        }
    }

    let cloud = CloudApiProvider;
    if cloud.is_available().await {
        if let Ok(cmd) = cloud.generate(prompt, ctx).await {
            return Ok((cmd, cloud.provider_name()));
        }
    }

    Err(anyhow!("No LLM provider available! Please ensure `agy` is installed, or `ollama` is running, or set `DEEPSEEK_API_KEY`/`OPENAI_API_KEY`."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_llm_response() {
        let raw = "```bash\nkill -9 8080\n```";
        assert_eq!(clean_llm_response(raw), "kill -9 8080");

        let plain = "git status";
        assert_eq!(clean_llm_response(plain), "git status");
    }
}
