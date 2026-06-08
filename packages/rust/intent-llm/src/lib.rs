use std::env;

use serde_json::Value;

pub struct DeepSeekClient {
    api_key: String,
    model: String,
}

pub struct ChatConfig {
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            temperature: 0.3,
            max_tokens: 8192,
        }
    }
}

impl DeepSeekClient {
    pub fn from_env() -> Result<Self, String> {
        let api_key = env::var("DEEPSEEK_API_KEY")
            .map_err(|_| "DEEPSEEK_API_KEY not set in environment")?;
        Ok(Self {
            api_key,
            model: "deepseek-chat".to_string(),
        })
    }

    pub fn chat(&self, prompt: &str) -> Result<String, String> {
        self.chat_with_config(prompt, &ChatConfig::default())
    }

    pub fn chat_with_config(&self, prompt: &str, config: &ChatConfig) -> Result<String, String> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
        });

        let response = ureq::post("https://api.deepseek.com/v1/chat/completions")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| format!("API request failed: {}", e))?;

        let resp_json: Value = response
            .into_json()
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        resp_json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in API response".to_string())
    }
}

pub fn extract_json(response: &str) -> Result<Value, String> {
    let json_str = if let Some(start) = response.find("```json") {
        let content_start = start + 7;
        let end = response[content_start..]
            .find("```")
            .map(|i| content_start + i)
            .unwrap_or(response.len());
        response[content_start..end].trim()
    } else if let Some(start) = response.find('{') {
        let end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        response[start..end].trim()
    } else {
        response.trim()
    };

    serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))
}
