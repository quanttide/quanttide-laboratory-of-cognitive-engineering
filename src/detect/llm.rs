//! LLM 调用封装 — 用于逻辑跳跃检测

use crate::detect::{RuleResult, TextRule};
use crate::detect::text::Document;

pub struct LogicJump;
impl TextRule for LogicJump {
    fn check(&self, doc: &Document) -> RuleResult {
        let api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            return RuleResult { name: "逻辑跳跃", score: 100.0, max_score: 100.0, details: vec!["未配置 LLM，跳过".to_string()] };
        }
        let texts: Vec<(usize, &str)> = doc.paragraphs.iter().filter(|p| !p.is_heading).map(|p| (p.line_start, p.text.as_str())).collect();
        if texts.len() < 2 {
            return RuleResult { name: "逻辑跳跃", score: 100.0, max_score: 100.0, details: vec!["段落不足".to_string()] };
        }
        let mut details = Vec::new();
        let body = build_payload(&texts);
        let jumps = match call_llm(&body) {
            Ok(resp) => parse_response(resp, &texts, &mut details),
            Err(e) => { details.push(format!("LLM 调用失败: {}", e)); 0 }
        };
        let score = if details.is_empty() { 100.0 } else { f64::max(0.0, 100.0 - jumps as f64 * 30.0) };
        if details.is_empty() { details.push("未检测到逻辑跳跃".to_string()); }
        RuleResult { name: "逻辑跳跃", score, max_score: 100.0, details }
    }
}

fn build_payload(texts: &[(usize, &str)]) -> serde_json::Value {
    serde_json::json!({
        "model": std::env::var("LLM_MODEL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "deepseek-v4-flash".to_string()),
        "messages": [
            {"role": "system", "content": "你是一个文档逻辑检测助手。找出文档中相邻段落之间的逻辑跳跃（突然转换话题、缺少过渡、因果关系断裂）。输出JSON数组，每个元素：{\"index\":段落序号,\"jump\":true/false,\"reason\":\"\"}"},
            {"role": "user", "content": texts.iter().enumerate().map(|(i, (_, t))| format!("{}: {}", i, t)).collect::<Vec<_>>().join("\n\n")}
        ],
        "max_tokens": 300
    })
}

fn call_llm(body: &serde_json::Value) -> Result<ureq::Response, ureq::Error> {
    let api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
    let base_url = std::env::var("LLM_BASE_URL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "https://api.deepseek.com".to_string());
    let client = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(5))
        .timeout_read(std::time::Duration::from_secs(10))
        .build();
    client.post(&format!("{}/chat/completions", base_url))
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(body)
}

fn parse_response(resp: ureq::Response, texts: &[(usize, &str)], details: &mut Vec<String>) -> usize {
    let data: serde_json::Value = resp.into_json().unwrap_or_default();
    let content = data["choices"][0]["message"]["content"].as_str().unwrap_or("[]");
    let items: Vec<serde_json::Value> = serde_json::from_str(content).unwrap_or_default();
    for item in &items {
        if item["jump"].as_bool().unwrap_or(false) {
            if let Some(idx) = item["index"].as_u64() {
                let idx = idx as usize;
                if idx > 0 && idx < texts.len() {
                    details.push(format!("第 {} 行与第 {} 行之间：{}", texts[idx - 1].0 + 1, texts[idx].0 + 1, item["reason"].as_str().unwrap_or("逻辑跳跃")));
                }
            }
        }
    }
    items.iter().filter(|i| i["jump"].as_bool().unwrap_or(false)).count()
}
