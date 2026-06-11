use std::collections::BTreeMap;
use crate::data::{Annotations, AnnotatedCausal, FlexibleBias, JournalDomain, JournalSchema};

/// Normalize text for fuzzy matching: strip whitespace/punctuation.
fn normalize(text: &str) -> String {
    text.trim()
        .trim_end_matches('。')
        .trim_end_matches('，')
        .trim_end_matches('.')
        .trim_end_matches('!')
        .trim_end_matches('?')
        .replace(' ', "")
}

/// Merge causals across weeks, applying annotations.
pub fn merge_causals(
    weeks_data: &[&JournalDomain],
    annotations: Option<&Annotations>,
) -> Vec<AnnotatedCausal> {
    // Build annotation lookup keyed by normalized condition
    let mut ann_map: BTreeMap<String, &crate::data::AnnotationEntry> = BTreeMap::new();
    if let Some(ann) = annotations {
        for entry in &ann.causals {
            ann_map.insert(normalize(&entry.condition), entry);
        }
    }

    // Collect all causals and annotate
    let mut all: Vec<AnnotatedCausal> = Vec::new();
    for wd in weeks_data {
        if let Some(ref schemas) = wd.schemas {
            for s in schemas {
                if let Some(ref causals) = s.causals {
                    for c in causals {
                        let mut cloned = c.clone();
                        let norm_cond = normalize(&c.condition);
                        if let Some(entry) = ann_map.get(&norm_cond) {
                            cloned.causal_type = Some(entry.causal_type.clone());
                            if let Some(ref v) = entry.verify {
                                cloned.verify = Some(v.clone());
                            }
                        } else if cloned.causal_type.is_none() {
                            cloned.causal_type = Some("保留".to_string());
                        }
                        all.push(cloned);
                    }
                }
            }
        }
    }

    // Deduplicate by normalized condition
    let mut seen: BTreeMap<String, AnnotatedCausal> = BTreeMap::new();
    for c in all {
        let key = normalize(&c.condition);
        seen.entry(key).or_insert(c);
    }

    seen.into_values().collect()
}

/// Merge entities across weeks (dedup by name, union attributes).
pub fn merge_entities(weeks_data: &[&JournalDomain]) -> Vec<quanttide_think::Entity> {
    let mut seen: BTreeMap<String, quanttide_think::Entity> = BTreeMap::new();
    for wd in weeks_data {
        if let Some(ref schemas) = wd.schemas {
            for s in schemas {
                if let Some(ref entities) = s.entities {
                    for e in entities {
                        let entry = seen.entry(e.name.clone()).or_insert_with(|| {
                            quanttide_think::Entity {
                                name: e.name.clone(),
                                attributes: Vec::new(),
                            }
                        });
                        for attr in &e.attributes {
                            if !entry.attributes.contains(attr) {
                                entry.attributes.push(attr.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    seen.into_values().collect()
}

/// Merge boundaries across weeks (dedup set).
pub fn merge_boundaries(weeks_data: &[&JournalDomain]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    for wd in weeks_data {
        if let Some(ref schemas) = wd.schemas {
            for s in schemas {
                if let Some(ref bounds) = s.boundaries {
                    for b in bounds {
                        seen.insert(b.clone());
                    }
                }
            }
        }
    }
    seen.into_iter().collect()
}

/// Merge key-value pairs (later weeks override earlier ones).
pub fn merge_keyvalues(
    weeks_data: &[&JournalDomain],
    extract: fn(&JournalSchema) -> Option<&Vec<quanttide_think::KeyValue>>,
) -> Vec<quanttide_think::KeyValue> {
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for wd in weeks_data {
        if let Some(ref schemas) = wd.schemas {
            for s in schemas {
                if let Some(kvs) = extract(s) {
                    for kv in kvs {
                        seen.insert(kv.key.clone(), kv.value.clone());
                    }
                }
            }
        }
    }
    seen.into_iter().map(|(k, v)| quanttide_think::KeyValue { key: k, value: v }).collect()
}

/// Merge mappings (union by intent).
pub fn merge_mappings(weeks_data: &[&JournalDomain]) -> Vec<quanttide_think::Mapping> {
    let mut seen: BTreeMap<String, quanttide_think::Mapping> = BTreeMap::new();
    for wd in weeks_data {
        if let Some(ref schemas) = wd.schemas {
            for s in schemas {
                if let Some(ref mappings) = s.mappings {
                    for m in mappings {
                        seen.entry(m.intent.clone()).or_insert_with(|| m.clone());
                    }
                }
            }
        }
    }
    seen.into_values().collect()
}

/// Merge biases (dedup by uuid or belief text).
pub fn merge_biases(weeks_data: &[&JournalDomain]) -> Vec<FlexibleBias> {
    let mut seen: BTreeMap<String, FlexibleBias> = BTreeMap::new();
    for wd in weeks_data {
        if let Some(ref schemas) = wd.schemas {
            for s in schemas {
                if let Some(ref biases) = s.biases {
                    for b in biases {
                        let key = b.id.to_string();
                        if !seen.contains_key(&key) {
                            seen.insert(key, b.clone());
                        }
                    }
                }
            }
        }
    }
    seen.into_values().collect()
}

/// Build a consolidated JournalSchema for a domain across multiple weeks.
pub fn fill_schema(
    weeks_data: &[&JournalDomain],
    annotations: Option<&Annotations>,
) -> JournalSchema {
    let mut usage_parts: Vec<String> = Vec::new();
    for wd in weeks_data {
        if let Some(ref situations) = wd.situations {
            for s in situations {
                usage_parts.push(format!("[{}] {}", s.name, s.content.agenda));
            }
        }
    }
    let usage = if usage_parts.is_empty() {
        "暂无描述".to_string()
    } else {
        format!("结合{}周数据：{}", usage_parts.len(), usage_parts.join("；"))
    };

    JournalSchema {
        usage: Some(usage),
        entities: Some(merge_entities(weeks_data)),
        causals: Some(merge_causals(weeks_data, annotations)),
        boundaries: Some(merge_boundaries(weeks_data)),
        properties: Some(merge_keyvalues(weeks_data, |s| s.properties.as_ref())),
        dynamics: Some(merge_keyvalues(weeks_data, |s| s.dynamics.as_ref())),
        mappings: Some(merge_mappings(weeks_data)),
        biases: Some(merge_biases(weeks_data)),
    }
}
