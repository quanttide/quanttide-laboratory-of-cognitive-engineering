pub use quanttide_think::{
    thought::Thought,
    intention::{Intention, Agent, Level, Priority, Trigger, Risk},
    situation::{Situation, SituationContent},
    schema::{Schema, SchemaContent, Entity, Causal, KeyValue, Mapping, Bias},
    domain::Domain,
};

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Relation {
    pub source: String,
    pub target: String,
    pub relation_type: String,
    pub strength: String,
    pub logic: String,
}

/// Mental model identified across situations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
