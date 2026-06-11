use std::collections::BTreeMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Journal domain file: the raw YAML structure per week per domain.
#[derive(Debug, Deserialize)]
pub struct JournalDomain {
    pub schemas: Option<Vec<JournalSchema>>,
    pub situations: Option<Vec<quanttide_think::Situation>>,
    pub intentions: Option<Vec<quanttide_think::Intention>>,
    pub thoughts: Option<Vec<String>>,
}

/// Schema in journal format (no id/name/label wrapper).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JournalSchema {
    pub usage: Option<String>,
    pub entities: Option<Vec<quanttide_think::Entity>>,
    pub causals: Option<Vec<AnnotatedCausal>>,
    pub boundaries: Option<Vec<String>>,
    pub properties: Option<Vec<quanttide_think::KeyValue>>,
    pub dynamics: Option<Vec<quanttide_think::KeyValue>>,
    pub mappings: Option<Vec<quanttide_think::Mapping>>,
    pub biases: Option<Vec<FlexibleBias>>,
}

/// Flexible bias that accepts optional UUID.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FlexibleBias {
    #[serde(default = "default_uuid")]
    pub id: uuid::Uuid,
    pub belief: String,
    pub fact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causal_type: Option<String>,
}

fn default_uuid() -> uuid::Uuid {
    uuid::Uuid::nil()
}
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

/// Annotations file format for causal type overrides.
#[derive(Debug, Deserialize)]
pub struct Annotations {
    pub causals: Vec<AnnotationEntry>,
}

#[derive(Debug, Deserialize)]
pub struct AnnotationEntry {
    pub condition: String,
    pub outcome: Option<String>,
    #[serde(rename = "type")]
    pub causal_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<String>,
}

/// Ingested journal data across all weeks.
#[derive(Debug)]
pub struct JournalData {
    pub weeks: BTreeMap<String, BTreeMap<String, JournalDomain>>,
}

/// Load journal data from the ingest JSON file.
pub fn load_journal(path: &PathBuf) -> Result<JournalData, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let raw: BTreeMap<String, BTreeMap<String, serde_json::Value>> = serde_json::from_reader(reader)?;

    let mut weeks: BTreeMap<String, BTreeMap<String, JournalDomain>> = BTreeMap::new();
    for (week_name, domains) in raw {
        let mut domain_map: BTreeMap<String, JournalDomain> = BTreeMap::new();
        for (domain_name, val) in domains {
            let jd: JournalDomain = serde_json::from_value(val)?;
            domain_map.insert(domain_name, jd);
        }
        weeks.insert(week_name, domain_map);
    }
    Ok(JournalData { weeks })
}

/// Load annotations from YAML file.
pub fn load_annotations(path: &PathBuf) -> Result<Annotations, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    Ok(serde_yaml::from_reader(file)?)
}
