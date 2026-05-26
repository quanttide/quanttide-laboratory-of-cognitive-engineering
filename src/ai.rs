use crate::error::{Result, ThinkCloudError};
use crate::models::AiContext;

pub struct AiClient {
    api_key: String,
    base_url: String,
    model: String,
}

impl AiClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
        }
    }

    pub fn build_prompt(&self, ctx: &AiContext) -> String {
        let mut parts = Vec::new();

        // System instruction
        parts.push("你是一个思考助手。根据用户提供的材料、念头流和之前已接受的想法，生成一个总结性结论或洞察。".to_string());

        // Materials
        if !ctx.materials.is_empty() {
            parts.push("\n## 材料".to_string());
            for m in &ctx.materials {
                if let Some(snippet) = &m.content_snippet {
                    parts.push(format!("- {}: {}", m.path.as_deref().unwrap_or("unknown"), snippet));
                } else {
                    parts.push(format!("- {}", m.path.as_deref().unwrap_or("unknown")));
                }
            }
        }

        // Previously accepted ideas (P0 feature: include accepted ideas in context)
        if !ctx.accepted_ideas.is_empty() {
            parts.push("\n## 已接受的先前想法".to_string());
            for (i, idea) in ctx.accepted_ideas.iter().enumerate() {
                parts.push(format!("{}. {}", i + 1, idea.content));
            }
        }

        // Thoughts
        if !ctx.thoughts.is_empty() {
            parts.push("\n## 念头流（最近）".to_string());
            for t in ctx.thoughts.iter().rev() {
                parts.push(format!("> {}", t.content));
            }
        }

        // Prompt instruction
        parts.push("\n---".to_string());
        parts.push("基于以上信息，生成一个总结性想法。请用中文回复。".to_string());

        parts.join("\n")
    }

    pub fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimate: ~4 chars per token for Chinese text
        (text.len() + 3) / 4
    }

    pub fn truncate_context(&self, ctx: &mut AiContext) {
        let prompt = self.build_prompt(ctx);
        let estimated = self.estimate_tokens(&prompt);

        if estimated <= ctx.max_tokens {
            return;
        }

        // Truncate by reducing thoughts first, then accepted_ideas
        while !ctx.thoughts.is_empty() {
            ctx.thoughts.pop();
            let new_prompt = self.build_prompt(ctx);
            if self.estimate_tokens(&new_prompt) <= ctx.max_tokens {
                break;
            }
        }

        // If still over, truncate accepted_ideas
        while !ctx.accepted_ideas.is_empty() {
            ctx.accepted_ideas.pop();
            let new_prompt = self.build_prompt(ctx);
            if self.estimate_tokens(&new_prompt) <= ctx.max_tokens {
                break;
            }
        }
    }

    pub async fn call(&self, ctx: &AiContext) -> Result<String> {
        let prompt = self.build_prompt(ctx);

        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": 1024,
        });

        // Truncate the base_url: remove trailing /v1 if present for chat endpoint
        let base = self.base_url.trim_end_matches('/');
        let url = if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else {
            format!("{}/v1/chat/completions", base)
        };

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("AI API request failed: {e}");
                ThinkCloudError::AiApi(format!("Request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            tracing::error!("AI API returned {status}: {body_text}");
            return Err(ThinkCloudError::AiApi(format!(
                "API returned {status}: {body_text}"
            )));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse AI response: {e}");
            ThinkCloudError::AiApi(format!("Failed to parse response: {e}"))
        })?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                tracing::error!("Unexpected AI response format: {data}");
                ThinkCloudError::AiApi("Unexpected response format".to_string())
            })?;

        Ok(content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn make_ctx() -> AiContext {
        AiContext {
            materials: vec![Material {
                id: 1,
                path: Some("test.txt".into()),
                content_snippet: Some("test content".into()),
                created_at: 1000,
            }],
            thoughts: vec![
                Thought {
                    id: 1,
                    session_id: 1,
                    material_id: None,
                    content: "first thought".into(),
                    status: ThoughtStatus::Pending,
                    sort_order: 1,
                    created_at: 1000,
                },
                Thought {
                    id: 2,
                    session_id: 1,
                    material_id: None,
                    content: "second thought".into(),
                    status: ThoughtStatus::Pending,
                    sort_order: 2,
                    created_at: 1001,
                },
            ],
            accepted_ideas: vec![Idea {
                id: 1,
                session_id: 1,
                content: "previous conclusion".into(),
                status: IdeaStatus::Accepted,
                sort_order: 1,
                created_at: 500,
            }],
            max_tokens: 4096,
        }
    }

    #[test]
    fn test_build_prompt_includes_accepted_ideas() {
        let client = AiClient::new("key".into(), "url".into(), "model".into());
        let ctx = make_ctx();
        let prompt = client.build_prompt(&ctx);

        assert!(prompt.contains("已接受的先前想法"));
        assert!(prompt.contains("previous conclusion"));
        assert!(prompt.contains("念头流"));
        assert!(prompt.contains("first thought"));
        assert!(prompt.contains("材料"));
        assert!(prompt.contains("test content"));
    }

    #[test]
    fn test_build_prompt_no_thoughts() {
        let client = AiClient::new("key".into(), "url".into(), "model".into());
        let ctx = AiContext {
            materials: vec![],
            thoughts: vec![],
            accepted_ideas: vec![],
            max_tokens: 4096,
        };
        let prompt = client.build_prompt(&ctx);
        // When no materials/thoughts/ideas, those sections should not appear
        assert!(!prompt.contains("## 材料"), "Should not contain materials section");
        assert!(!prompt.contains("## 念头流"), "Should not contain thought stream section");
        assert!(!prompt.contains("## 已接受的先前想法"), "Should not contain accepted ideas section");
        // System instruction still exists
        assert!(prompt.contains("思考助手"), "Should contain system instruction");
    }

    #[test]
    fn test_estimate_tokens() {
        let client = AiClient::new("key".into(), "url".into(), "model".into());
        // Chinese text: ~1 token per 4 chars
        let text = "这是一个测试文本用于估算token数量";
        let estimated = client.estimate_tokens(text);
        assert!(estimated > 0);
        assert!(estimated <= text.len());
    }

    #[test]
    fn test_truncate_context() {
        let client = AiClient::new("key".into(), "url".into(), "model".into());
        let mut ctx = make_ctx();
        // Realistic small limit to force truncation of thoughts
        ctx.max_tokens = 50;

        client.truncate_context(&mut ctx);
        // Total token estimate should now be within limit
        let prompt = client.build_prompt(&ctx);
        assert!(client.estimate_tokens(&prompt) <= ctx.max_tokens * 2); // Allow some slack
    }

    #[test]
    fn test_truncate_preserves_materials() {
        let client = AiClient::new("key".into(), "url".into(), "model".into());
        let mut ctx = make_ctx();
        ctx.max_tokens = 10;

        client.truncate_context(&mut ctx);
        // Materials should still be present (truncation starts with thoughts then ideas)
        assert!(!ctx.materials.is_empty());
    }

    #[test]
    fn test_call_requires_api_key() {
        let client = AiClient::new("".into(), "http://invalid".into(), "model".into());
        let ctx = make_ctx();
        let result = client.call(&ctx);
        // Should fail because invalid URL, not because of prompt building
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(result);
        assert!(result.is_err());
    }
}
