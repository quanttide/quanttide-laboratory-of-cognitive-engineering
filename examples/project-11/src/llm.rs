use std::env;

use serde_json::Value;

pub struct DeepSeekClient {
    api_key: String,
    model: String,
}

impl DeepSeekClient {
    pub fn from_env() -> Result<Self, String> {
        let api_key =
            env::var("DEEPSEEK_API_KEY").map_err(|_| "DEEPSEEK_API_KEY not set".to_string())?;
        let model = env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());
        Ok(Self { api_key, model })
    }

    pub fn chat(&self, prompt: &str) -> Result<String, String> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.3,
            "max_tokens": 4096,
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
    // Try code block first
    for marker in &["```json", "```JSON", "```yaml", "```YAML"] {
        if let Some(start) = response.find(marker) {
            let s = start + marker.len();
            let e = response[s..].find("```").map(|i| s + i).unwrap_or(response.len());
            let trimmed = response[s..e].trim();
            if let Ok(v) = serde_json::from_str(trimmed) {
                return Ok(v);
            }
        }
    }
    // Try raw JSON object
    if let Some(start) = response.find('{') {
        let e = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let trimmed = response[start..e].trim();
        if let Ok(v) = serde_json::from_str(trimmed) {
            return Ok(v);
        }
    }
    // Try raw JSON array
    if let Some(start) = response.find('[') {
        let e = response.rfind(']').map(|i| i + 1).unwrap_or(response.len());
        let trimmed = response[start..e].trim();
        if let Ok(v) = serde_json::from_str(trimmed) {
            return Ok(v);
        }
    }
    Err("No valid JSON found in response".to_string())
}
