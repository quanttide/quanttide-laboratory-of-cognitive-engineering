/// 关系类型分类法
///
/// 用于 LLM 推断意图簇之间关系的扩展类型体系。
/// 从最具体到最通用排列：
///
/// | 类型 | 含义 | 证据标准 |
/// |------|------|---------|
/// | 支持 | A 的存在/演进促进了 B | 同一段落明确提及 A→B 的因果 |
/// | 冲突 | A 与 B 存在内在矛盾 | 同一段落中 A/B 互斥或对立 |
/// | 触发 | A 的事件激活了 B | A 的变化在时序上紧接 B 的出现 |
/// | 演化 | A 的演进推动 B 的演进 | A 和 B 的演化轨迹在时间上平行 |
/// | 情感补给 | A 为 B 提供情绪能量 | 情绪语言连接两簇（如"释压""充电"） |
/// | 同框 | 同一时空出现但无直接互动 | 同一日/周日记中先后提及 |
/// | 时序 | A 先于 B 出现且逻辑相关 | 跨周轨迹中 A 的消失预示 B 的出现 |
/// | 类比 | A 和 B 用同一隐喻框架 | 共享核心术语（如"涌现"出现在两簇） |
/// | 组件 | A 是 B 的组成部分 | 一个簇的目的被另一个簇承载 |

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
