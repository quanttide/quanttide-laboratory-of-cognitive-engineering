use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThoughtStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl ThoughtStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThoughtStatus::Pending => "pending",
            ThoughtStatus::Processing => "processing",
            ThoughtStatus::Completed => "completed",
            ThoughtStatus::Failed => "failed",
        }
    }
}

impl fmt::Display for ThoughtStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for ThoughtStatus {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "pending" => Ok(ThoughtStatus::Pending),
            "processing" => Ok(ThoughtStatus::Processing),
            "completed" => Ok(ThoughtStatus::Completed),
            "failed" => Ok(ThoughtStatus::Failed),
            _ => Err(format!("Invalid thought status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdeaStatus {
    Pending,
    Accepted,
    Rejected,
    Failed,
}

impl IdeaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IdeaStatus::Pending => "pending",
            IdeaStatus::Accepted => "accepted",
            IdeaStatus::Rejected => "rejected",
            IdeaStatus::Failed => "failed",
        }
    }
}

impl fmt::Display for IdeaStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for IdeaStatus {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "pending" => Ok(IdeaStatus::Pending),
            "accepted" => Ok(IdeaStatus::Accepted),
            "rejected" => Ok(IdeaStatus::Rejected),
            "failed" => Ok(IdeaStatus::Failed),
            _ => Err(format!("Invalid idea status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub title: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub ai_pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: i64,
    pub path: Option<String>,
    pub content_snippet: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub id: i64,
    pub session_id: i64,
    pub material_id: Option<i64>,
    pub content: String,
    pub status: ThoughtStatus,
    pub sort_order: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idea {
    pub id: i64,
    pub session_id: i64,
    pub content: String,
    pub status: IdeaStatus,
    pub sort_order: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMaterial {
    pub session_id: i64,
    pub material_id: i64,
}

#[derive(Debug, Clone)]
pub struct AiContext {
    pub materials: Vec<Material>,
    pub thoughts: Vec<Thought>,
    pub accepted_ideas: Vec<Idea>,
    pub max_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thought_status_roundtrip() {
        for status in &[ThoughtStatus::Pending, ThoughtStatus::Processing, ThoughtStatus::Completed, ThoughtStatus::Failed] {
            let s = status.as_str();
            let parsed = ThoughtStatus::try_from(s).unwrap();
            assert_eq!(*status, parsed);
        }
    }

    #[test]
    fn test_thought_status_invalid() {
        assert!(ThoughtStatus::try_from("unknown").is_err());
    }

    #[test]
    fn test_idea_status_roundtrip() {
        for status in &[IdeaStatus::Pending, IdeaStatus::Accepted, IdeaStatus::Rejected, IdeaStatus::Failed] {
            let s = status.as_str();
            let parsed = IdeaStatus::try_from(s).unwrap();
            assert_eq!(*status, parsed);
        }
    }

    #[test]
    fn test_idea_status_display() {
        assert_eq!(IdeaStatus::Failed.to_string(), "failed");
        assert_eq!(IdeaStatus::Accepted.to_string(), "accepted");
    }
}
