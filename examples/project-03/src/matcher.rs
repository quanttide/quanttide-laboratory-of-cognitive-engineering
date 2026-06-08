use std::collections::HashSet;

use crate::models::{KeywordTable, MatchResult, OutputResponse};
use crate::tokenizer;

pub fn match_text(text: &str, keywords: &KeywordTable, threshold: f64) -> OutputResponse {
    let text_words = tokenizer::tokenize(text);
    let text_word_set: HashSet<&str> = text_words.iter().map(|s| s.as_str()).collect();

    let mut matched = Vec::new();

    for entry in keywords {
        let cluster_keywords: Vec<&str> = entry.keywords.iter().map(|s| s.as_str()).collect();
        let cluster_count = cluster_keywords.len();

        if cluster_count == 0 {
            continue;
        }

        let intersection_count = cluster_keywords
            .iter()
            .filter(|kw| text_word_set.contains(**kw))
            .count();

        let score = intersection_count as f64 / cluster_count as f64;

        if score > threshold {
            let evidence: Vec<String> = cluster_keywords
                .iter()
                .filter(|kw| text_word_set.contains(**kw))
                .map(|s| s.to_string())
                .collect();

            matched.push(MatchResult {
                id: entry.id,
                name: entry.name.clone(),
                score: (score * 100.0).round() / 100.0,
                evidence,
            });
        }
    }

    matched.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    OutputResponse { matched }
}
