use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::context::ProjectContext;

const SYSTEM_PROMPT: &str = r#"You are a senior debugging expert. Your job is to analyze errors, log outputs, and failed commands to help developers quickly understand and fix issues.

When analyzing an error, you must:
1. Identify the root cause of the error precisely
2. Explain it in plain English that any developer can understand
3. Provide ranked fixes with the most likely solution first
4. Detect the programming language, framework, or tool involved
5. Suggest preventive measures to avoid this error in the future

You MUST respond with valid JSON in this exact format:
{
  "root_cause": "Brief description of the root cause",
  "explanation": "Detailed explanation in plain English",
  "language_or_tool": "The language/framework/tool involved (e.g., Rust, Node.js, Docker, Terraform)",
  "fixes": [
    {
      "description": "Description of the fix",
      "command": "Shell command to execute (empty string if not applicable)",
      "confidence": 95
    }
  ],
  "prevention_tips": [
    "Tip to prevent this error in the future"
  ]
}

Rules:
- Always provide at least one fix
- Confidence should be 0-100
- Order fixes from highest to lowest confidence
- Be specific with commands — use actual package names, file paths, etc.
- If you detect project context, tailor your advice accordingly
- Keep explanations concise but thorough
- Do NOT wrap your response in markdown code fences. Return raw JSON only."#;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiResponse {
    pub root_cause: String,
    pub explanation: String,
    pub language_or_tool: String,
    pub fixes: Vec<Fix>,
    pub prevention_tips: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fix {
    pub description: String,
    #[serde(default)]
    pub command: String,
    pub confidence: u8,
}

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(tag = "type")]
enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart {},
    #[serde(rename = "content_block_start")]
    ContentBlockStart {},
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { delta: Delta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {},
    #[serde(rename = "message_delta")]
    MessageDelta {},
    #[serde(rename = "message_stop")]
    MessageStop {},
    #[serde(rename = "ping")]
    Ping {},
    #[serde(other)]
    Unknown,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Delta {
    #[serde(rename = "type")]
    _type: Option<String>,
    text: Option<String>,
}

pub struct AiEngine {
    config: Config,
    client: Client,
}

impl AiEngine {
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::new();
        Ok(Self { config, client })
    }

    pub fn build_prompt(error_text: &str, context: Option<&ProjectContext>) -> String {
        let mut prompt = String::new();

        if let Some(ctx) = context {
            let summary = ctx.summary();
            if !summary.is_empty() {
                prompt.push_str("Project context:\n");
                prompt.push_str(&summary);
                prompt.push_str("\n\n");
            }
        }

        prompt.push_str("Analyze this error and provide fixes:\n\n");
        prompt.push_str(error_text);
        prompt
    }

    pub async fn analyze(
        &self,
        error_text: &str,
        context: Option<&ProjectContext>,
    ) -> Result<AiResponse> {
        let api_key = self
            .config
            .api_key()
            .context("API key not configured. Set it with:\n  sherlog config set api_key <your-key>\n  or set the SHERLOG_API_KEY environment variable")?;

        let prompt = Self::build_prompt(error_text, context);

        let request = ApiRequest {
            model: self.config.model.clone(),
            max_tokens: 4096,
            system: SYSTEM_PROMPT.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            stream: false,
        };

        let url = format!("{}/v1/messages", self.config.api_base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Anthropic API. Check your internet connection.")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            match status.as_u16() {
                401 => bail!("Authentication failed. Check your API key.\n  Run: sherlog config set api_key <your-key>"),
                429 => bail!("Rate limit exceeded. Please wait a moment and try again.\nDetails: {body}"),
                500..=599 => bail!("Anthropic API server error ({status}). Try again later.\nDetails: {body}"),
                _ => bail!("API request failed ({status}): {body}"),
            }
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        let text = api_response
            .content
            .first()
            .and_then(|b| b.text.as_ref())
            .context("Empty response from API")?;

        parse_ai_response(text)
    }

    #[allow(dead_code)]
    pub async fn analyze_streaming(
        &self,
        error_text: &str,
        context: Option<&ProjectContext>,
    ) -> Result<AiResponse> {
        let api_key = self
            .config
            .api_key()
            .context("API key not configured. Set it with:\n  sherlog config set api_key <your-key>\n  or set the SHERLOG_API_KEY environment variable")?;

        let prompt = Self::build_prompt(error_text, context);

        let request = ApiRequest {
            model: self.config.model.clone(),
            max_tokens: 4096,
            system: SYSTEM_PROMPT.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            stream: true,
        };

        let url = format!("{}/v1/messages", self.config.api_base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Anthropic API")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            match status.as_u16() {
                401 => bail!("Authentication failed. Check your API key."),
                429 => bail!("Rate limit exceeded. Please wait and try again.\n{body}"),
                _ => bail!("API request failed ({status}): {body}"),
            }
        }

        let mut full_text = String::new();
        let mut stream = response.bytes_stream();

        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Stream interrupted")?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            // Process complete SSE lines
            while let Some(pos) = buffer.find("\n\n") {
                let event_block = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                for line in event_block.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }
                        if let Ok(StreamEvent::ContentBlockDelta { delta }) =
                            serde_json::from_str::<StreamEvent>(data)
                        {
                            if let Some(text) = delta.text {
                                eprint!("{text}");
                                full_text.push_str(&text);
                            }
                        }
                    }
                }
            }
        }

        eprintln!(); // newline after streaming
        parse_ai_response(&full_text)
    }
}

fn parse_ai_response(text: &str) -> Result<AiResponse> {
    // Try to extract JSON from the response (handle markdown fences)
    let json_str = if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            &text[start..=end]
        } else {
            text
        }
    } else {
        text
    };

    serde_json::from_str::<AiResponse>(json_str).with_context(|| {
        format!(
            "Failed to parse AI response as structured JSON. Raw response:\n{}",
            &text[..text.len().min(500)]
        )
    })
}
