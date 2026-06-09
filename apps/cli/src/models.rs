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
