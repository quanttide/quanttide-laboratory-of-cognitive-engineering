use std::collections::{HashMap, HashSet};

use quanttide_think::{
    situation::Situation,
    schema::Schema,
};

use crate::tokenizer::tokenize;

pub struct KeywordIndex {
    /// situation name → keywords
    sit_keys: HashMap<String, Vec<String>>,
}

impl KeywordIndex {
    pub fn new(situations: &[Situation], schemas: &[Schema]) -> Self {
        let mut sit_keys: HashMap<String, Vec<String>> = HashMap::new();
        for sit in situations {
            let mut all = tokenize(&sit.content.agenda);
            all.extend(tokenize(&sit.content.ecology));
            all.extend(tokenize(&sit.content.frame));
            all.extend(tokenize(&sit.content.dynamics));
            all.sort();
            all.dedup();
            sit_keys.insert(sit.name.clone(), all);
        }
        // Also index schema text
        for sch in schemas {
            let entry = sit_keys.entry(sch.name.clone()).or_default();
            let mut tokens = tokenize(&sch.content.usage);
            for c in &sch.content.causals {
                tokens.extend(tokenize(&c.condition));
                tokens.extend(tokenize(&c.outcome));
            }
            tokens.sort();
            tokens.dedup();
            // Only add tokens not already present
            for t in tokens {
                if !entry.contains(&t) {
                    entry.push(t);
                }
            }
        }
        Self { sit_keys }
    }

    /// Return sorted (situation_name, score) for the top-N matches
    pub fn search(&self, query: &str, top_n: usize) -> Vec<(String, usize)> {
        let q_tokens: HashSet<String> = tokenize(query).into_iter().collect();
        if q_tokens.is_empty() {
            return Vec::new();
        }
        let mut scores: Vec<(String, usize)> = self.sit_keys
            .iter()
            .filter_map(|(name, keywords)| {
                let matched = keywords.iter().filter(|k| q_tokens.contains(k.as_str())).count();
                if matched > 0 {
                    Some((name.clone(), matched))
                } else {
                    None
                }
            })
            .collect();
        scores.sort_by(|a, b| b.1.cmp(&a.1));
        scores.truncate(top_n);
        scores
    }
}
