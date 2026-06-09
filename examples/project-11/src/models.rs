use serde::{Deserialize, Serialize};

/// A single week's situation file (e.g. 2026-W23/org.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Situation {
    pub id: String,
    pub name: String,
    pub label: String,
    pub content: SituationContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SituationContent {
    pub agenda: String,
    pub ecology: String,
    pub frame: String,
    pub dynamics: String,
}

/// A single intention entry (list root in intention YAML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intention {
    pub id: String,
    pub title: String,
    pub description: String,
    pub motivation: String,
    pub agent: Labelled,
    pub level: LabelledDescription,
    pub priority: LabelledDescription,
    pub trigger: LabelledDescription,
    pub risk: LabelledDescription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Labelled {
    pub name: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelledDescription {
    pub name: String,
    pub label: String,
    pub description: String,
}

/// Registry entry mapping name to label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub label: String,
}

/// A week bundle: all situations + intentions for a given week
#[derive(Debug, Clone)]
pub struct WeekData {
    pub week: String,
    pub situations: Vec<Situation>,
    pub intentions: Vec<Intention>,
    /// intentions keyed by situation name
    pub intention_map: std::collections::HashMap<String, Vec<Intention>>,
}

/// Relationship between two situations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub source: String,
    pub target: String,
    pub relation_type: String,
    pub strength: String,
    pub logic: String,
}

/// Schema: fine-grained mental model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub id: String,
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub content: SchemaContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaContent {
    #[serde(default)]
    pub usage: String,
    #[serde(default)]
    pub entities: Vec<serde_yaml::Value>,
    #[serde(default)]
    pub causals: Vec<Causal>,
    #[serde(default)]
    pub boundaries: Vec<String>,
    #[serde(default)]
    pub properties: Vec<KeyValue>,
    #[serde(default)]
    pub mappings: Vec<IntentMapping>,
    #[serde(default)]
    pub biases: Vec<Bias>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Causal {
    pub condition: String,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentMapping {
    pub intent: String,
    pub action: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bias {
    pub id: String,
    pub belief: String,
    pub fact: String,
}

/// Mental model identified across situations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalModel {
    pub name: String,
    pub definition: String,
    pub situations: Vec<String>,
    pub correct_pattern: String,
    pub error_pattern: String,
    pub correct_summary: String,
    pub error_summary: String,
    pub prediction: String,
}

/// Full weekly report
#[derive(Debug, Clone)]
pub struct WeeklyReport {
    pub week: String,
    pub core_judgment: String,
    pub actions: Vec<Action>,
    pub situations: Vec<SituationReport>,
    pub relations: Vec<Relation>,
    pub mental_models: Vec<MentalModel>,
    pub comparisons: Vec<Comparison>,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub priority: String,
    pub action: String,
    pub owner: String,
    pub deadline: String,
    pub expected_outcome: String,
    pub risk: String,
}

#[derive(Debug, Clone)]
pub struct SituationReport {
    pub label: String,
    pub name: String,
    pub phenomenon: String,
    pub reason: String,
    pub implication: String,
    pub key_intentions: Vec<Intention>,
}

#[derive(Debug, Clone)]
pub struct Comparison {
    pub label: String,
    pub change: String,
    pub implication: String,
}
