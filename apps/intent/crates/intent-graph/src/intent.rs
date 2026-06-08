use serde::{Deserialize, Serialize};

pub type IntentId = usize;

/// A store of intent content strings, indexed by IntentId.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentStore {
    intents: Vec<String>,
}

impl IntentStore {
    pub fn new() -> Self {
        Self { intents: Vec::new() }
    }

    pub fn add(&mut self, content: String) -> IntentId {
        let id = self.intents.len();
        self.intents.push(content);
        id
    }

    pub fn get(&self, id: IntentId) -> Option<&str> {
        self.intents.get(id).map(|s| s.as_str())
    }

    pub fn all(&self) -> &[String] {
        &self.intents
    }

    pub fn len(&self) -> usize {
        self.intents.len()
    }

    pub fn into_vec(self) -> Vec<String> {
        self.intents
    }

    pub fn from_vec(intents: Vec<String>) -> Self {
        Self { intents }
    }
}
