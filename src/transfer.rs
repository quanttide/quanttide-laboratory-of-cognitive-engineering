use std::collections::{BTreeMap, BTreeSet};
use qtcloud_think_cli::repo::DomainFile;
use crate::data::{Annotations, AnnotatedCausal, FlexibleBias, OutputSchema};

fn normalize(text: &str) -> String {
    text.trim().trim_end_matches(['。', '，', '.', '!', '?']).replace(' ', "")
}

/// Build a consolidated OutputSchema for a domain across multiple weeks.
pub fn fill_schema(
    weeks_data: &[&DomainFile],
    annotations: Option<&Annotations>,
) -> OutputSchema {
    // Usage: collect agenda from situations
    let mut usage_parts: Vec<String> = Vec::new();
    for wf in weeks_data {
        for js in &wf.situations {
            usage_parts.push(format!("[{}] {}", js.situation.name, js.situation.content.agenda));
        }
    }
    let usage = if usage_parts.is_empty() {
        "暂无描述".into()
    } else {
        format!("结合{}周数据：{}", usage_parts.len(), usage_parts.join("；"))
    };

    // Build annotation lookup
    let ann_map: BTreeMap<String, &crate::data::AnnotationEntry> = annotations
        .map(|a| a.causals.iter().map(|e| (normalize(&e.condition), e)).collect())
        .unwrap_or_default();

    // SchemaContent → AnnotatedCausal converter with annotation overlay
    let annotate_causal = |c: &quanttide_think::Causal| -> AnnotatedCausal {
        let norm = normalize(&c.condition);
        let entry = ann_map.get(&norm);
        AnnotatedCausal {
            condition: c.condition.clone(),
            outcome: c.outcome.clone(),
            causal_type: entry.map(|e| e.causal_type.clone()).or(Some("保留".into())),
            verify: entry.and_then(|e| e.verify.clone()),
            note: None,
        }
    };

    // Collect SchemaContent from all weeks
    let all_schemas: Vec<&quanttide_think::SchemaContent> = weeks_data.iter()
        .filter_map(|wf| wf.schemas.as_ref())
        .flat_map(|v| v.iter())
        .collect();

    if all_schemas.is_empty() {
        return OutputSchema {
            usage: Some(usage), entities: None, causals: None, boundaries: None,
            properties: None, dynamics: None, mappings: None, biases: None,
        };
    }

    // Entities: merge by name
    let mut entity_map: BTreeMap<String, quanttide_think::Entity> = BTreeMap::new();
    for sc in &all_schemas {
        for e in &sc.entities {
            let entry = entity_map.entry(e.name.clone()).or_insert_with(||
                quanttide_think::Entity { name: e.name.clone(), attributes: vec![] });
            for a in &e.attributes {
                if !entry.attributes.contains(a) { entry.attributes.push(a.clone()); }
            }
        }
    }

    // Causals: annotate + dedup
    let mut causal_map: BTreeMap<String, AnnotatedCausal> = BTreeMap::new();
    for sc in &all_schemas {
        for c in &sc.causals {
            let key = normalize(&c.condition);
            causal_map.entry(key).or_insert_with(|| annotate_causal(c));
        }
    }

    // Boundaries: set
    let mut boundary_set: BTreeSet<String> = BTreeSet::new();
    for sc in &all_schemas { for b in &sc.boundaries { boundary_set.insert(b.clone()); } }

    // Properties & dynamics: later weeks override
    let mut prop_map: BTreeMap<String, String> = BTreeMap::new();
    let mut dyn_map: BTreeMap<String, String> = BTreeMap::new();
    for sc in &all_schemas {
        for kv in &sc.properties { prop_map.insert(kv.key.clone(), kv.value.clone()); }
        for kv in &sc.dynamics { dyn_map.insert(kv.key.clone(), kv.value.clone()); }
    }

    // Mappings: dedup by intent
    let mut mapping_map: BTreeMap<String, &quanttide_think::Mapping> = BTreeMap::new();
    for sc in &all_schemas { for m in &sc.mappings { mapping_map.entry(m.intent.clone()).or_insert(m); } }

    // Biases: dedup by UUID
    let mut bias_map: BTreeMap<String, &quanttide_think::Bias> = BTreeMap::new();
    for sc in &all_schemas { for b in &sc.biases { bias_map.entry(b.id.to_string()).or_insert(b); } }

    OutputSchema {
        usage: Some(usage),
        entities: Some(entity_map.into_values().collect()),
        causals: Some(causal_map.into_values().collect()),
        boundaries: Some(boundary_set.into_iter().collect()),
        properties: Some(prop_map.into_iter().map(|(k, v)| quanttide_think::KeyValue { key: k, value: v }).collect()),
        dynamics: Some(dyn_map.into_iter().map(|(k, v)| quanttide_think::KeyValue { key: k, value: v }).collect()),
        mappings: Some(mapping_map.into_values().cloned().collect()),
        biases: Some(bias_map.into_values().map(|b| FlexibleBias {
            id: b.id, belief: b.belief.clone(), fact: b.fact.clone(), causal_type: None,
        }).collect()),
    }
}
