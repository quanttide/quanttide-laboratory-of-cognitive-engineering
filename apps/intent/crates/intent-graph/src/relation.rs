use serde::{Deserialize, Serialize};

/// Weight stored on each graph edge.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdgeWeight {
    pub relation_type: String,
    pub logic: String,
    pub weeks: Vec<String>,
    pub period_type: String,
}

pub(crate) fn parse_name(name: &str) -> (String, String, bool) {
    for sep in &[" ⇄ ", " ↔ ", " → "] {
        if let Some(pos) = name.find(sep) {
            let source = name[..pos].trim().to_string();
            let target = name[pos + sep.len()..].trim().to_string();
            let bidirectional = *sep == " ⇄ " || *sep == " ↔ ";
            return (source, target, bidirectional);
        }
    }
    (String::new(), name.to_string(), false)
}

pub(crate) fn parse_type(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(paren) = raw.find('（') {
        raw[..paren].trim().to_string()
    } else if let Some(paren) = raw.find('(') {
        raw[..paren].trim().to_string()
    } else if raw.contains("双向") {
        "支持".to_string()
    } else if raw.contains(" + ") {
        raw.split(" + ").next().unwrap_or(raw).trim().to_string()
    } else {
        raw.to_string()
    }
}
