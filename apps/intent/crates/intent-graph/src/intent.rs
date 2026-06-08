use serde::{Deserialize, Serialize};

pub type IntentId = usize;

/// A collection of intent content strings indexed by IntentId.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    items: Vec<String>,
}

impl Intent {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, content: String) -> IntentId {
        let id = self.items.len();
        self.items.push(content);
        id
    }

    pub fn get(&self, id: IntentId) -> Option<&str> {
        self.items.get(id).map(|s| s.as_str())
    }

    pub fn all(&self) -> &[String] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn into_vec(self) -> Vec<String> {
        self.items
    }

    pub fn from_vec(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl Default for Intent {
    fn default() -> Self {
        Self::new()
    }
}
