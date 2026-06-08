use serde::{Deserialize, Serialize};

/// Weight stored on each graph edge.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdgeWeight {
    pub relation_type: String,
    pub logic: String,
    pub weeks: Vec<String>,
    pub period_type: String,
}
