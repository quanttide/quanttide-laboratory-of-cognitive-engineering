use serde::Deserialize;

use crate::situation::Situation;

/// Full definition of the situation graph (nodes) in YAML form.
#[derive(Debug, Deserialize)]
pub struct GraphDefinition {
    pub situations: Vec<Situation>,
}

#[derive(Debug, Deserialize)]
pub struct RelationDefinition {
    pub stable_relations: Vec<RelationEntry>,
    pub periodic_tensions: Vec<RelationEntry>,
    pub situational_relations: Vec<SituationalRelationEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RelationEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub relation_type: String,
    pub weeks: Vec<String>,
    pub logic: String,
}

#[derive(Debug, Deserialize)]
pub struct SituationalRelationEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub relation_type: String,
    pub weeks: Vec<String>,
    pub trigger: Option<String>,
}
