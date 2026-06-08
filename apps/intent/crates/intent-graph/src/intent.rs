pub type IntentId = u64;

/// An atomic intent extracted from raw material.
#[derive(Debug, Clone)]
pub struct Intent {
    pub id: IntentId,
    pub content: String,
}
