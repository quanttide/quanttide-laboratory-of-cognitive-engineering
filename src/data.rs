use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Output schema format (with annotations on causals/biases).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OutputSchema {
    pub usage: Option<String>,
    pub entities: Option<Vec<quanttide_think::Entity>>,
    pub causals: Option<Vec<AnnotatedCausal>>,
    pub boundaries: Option<Vec<String>>,
    pub properties: Option<Vec<quanttide_think::KeyValue>>,
    pub dynamics: Option<Vec<quanttide_think::KeyValue>>,
    pub mappings: Option<Vec<quanttide_think::Mapping>>,
    pub biases: Option<Vec<FlexibleBias>>,
}

/// Causal with optional type/verify annotations.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnnotatedCausal {
    pub condition: String,
    pub outcome: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub causal_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Bias with optional UUID.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FlexibleBias {
    #[serde(default = "nil_uuid")]
    pub id: uuid::Uuid,
    pub belief: String,
    pub fact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causal_type: Option<String>,
}

fn nil_uuid() -> uuid::Uuid { uuid::Uuid::nil() }

/// Causal type annotations from human review.
#[derive(Debug, Deserialize)]
pub struct Annotations {
    pub causals: Vec<AnnotationEntry>,
}

#[derive(Debug, Deserialize)]
pub struct AnnotationEntry {
    pub condition: String,
    #[serde(rename = "type")]
    pub causal_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<String>,
}

pub fn load_annotations(path: &PathBuf) -> Result<Annotations, Box<dyn std::error::Error>> {
    Ok(serde_yaml::from_reader(std::fs::File::open(path)?)?)
}

/// Wrap OutputSchema with top-level `schemas:` key for YAML output.
#[derive(Serialize, Deserialize)]
pub struct SchemaFile {
    pub schemas: Vec<OutputSchema>,
}
