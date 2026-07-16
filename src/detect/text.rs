//! 文本（Markdown 散文）检测指标
//! 结构：Document → Paragraph / Table → 各规则消费

use crate::detect::{RuleResult, TextRule};
use std::collections::HashSet;

// ── 文档模型 ──────────────────────────────────────────

#[derive(Debug)]
pub struct Document {
    pub paragraphs: Vec<Paragraph>,
    pub tables: Vec<Table>,
}

#[derive(Debug)]
pub struct Paragraph {
    pub line_start: usize,
    pub text: String,
    pub is_heading: bool,
    pub heading_level: usize,
}

#[derive(Debug)]
pub struct Table {
    pub line_start: usize,
    pub rows: Vec<Vec<String>>,
}

/// 从 Markdown 文本解析文档
pub fn parse(text: &str) -> Document {
    let lines: Vec<&str> = text.lines().collect();
    let mut paragraphs = Vec::new();
    let mut tables = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(level) = heading_level(line) {
            paragraphs.push(Paragraph {
                line_start: i,
                text: line.to_string(),
                is_heading: true,
                heading_level: level,
            });
        } else if line.trim_start().starts_with('|') && line.trim_end().ends_with('|') {
            let mut rows = Vec::new();
            while i < lines.len() && lines[i].trim_start().starts_with('|') {
                let cells: Vec<String> = lines[i]
                    .split('|')
                    .filter(|c| !c.is_empty())
                    .map(|c| c.trim().to_string())
                    .collect();
                rows.push(cells);
                i += 1;
            }
            tables.push(Table { line_start: i - rows.len(), rows });
            continue;
        } else if !line.trim().is_empty() {
            paragraphs.push(Paragraph { line_start: i, text: line.to_string(), is_heading: false, heading_level: 0 });
        }
        i += 1;
    }
    Document { paragraphs, tables }
}

fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        let count = trimmed.chars().take_while(|c| *c == '#').count();
        if trimmed.len() > count && trimmed.as_bytes()[count] == b' ' { Some(count) } else { None }
    } else { None }
}

// ── 规则实现 ──────────────────────────────────────────

const TRANSITION_WORDS: &[&str] = &[
    "但是", "因此", "例如", "然而", "所以", "不过",
    "而且", "此外", "总之", "也就是说", "换句话说",
    "具体来说", "另一方面", "与此同时", "尽管如此",
];

pub struct TitleDepth;
impl TextRule for TitleDepth {
    fn check(&self, doc: &Document) -> RuleResult {
        let headings: Vec<_> = doc.paragraphs.iter().filter(|p| p.is_heading).collect();
        let (details, score) = if headings.is_empty() {
            (vec!["文档缺少标题结构".to_string()], 40.0)
        } else {
            let max_level = headings.iter().map(|p| p.heading_level).max().unwrap();
            if max_level > 3 {
                (vec![format!("标题层级最深为 {} 层（建议不超过 3 层）", max_level)],
                 f64::max(0.0, 100.0 - (max_level as f64 - 3.0) * 20.0))
            } else { (vec![], 100.0) }
        };
        RuleResult { name: "标题层级", score, max_score: 100.0, details: if details.is_empty() { vec!["标题层级合理".to_string()] } else { details } }
    }
}

pub struct TransitionWords;
impl TextRule for TransitionWords {
    fn check(&self, doc: &Document) -> RuleResult {
        let body: Vec<&Paragraph> = doc.paragraphs.iter().filter(|p| !p.is_heading).collect();
        let count = body.iter().filter(|p| TRANSITION_WORDS.iter().any(|w| p.text.contains(w))).count();
        let ratio = if body.is_empty() { 1.0 } else { count as f64 / body.len() as f64 };
        let score = f64::min(100.0, ratio * 100.0);
        let mut details = Vec::new();
        if count == 0 { details.push("未检测到过渡词".to_string()); }
        RuleResult { name: "过渡词使用", score, max_score: 100.0, details }
    }
}

pub struct TextSimilarity;
impl TextRule for TextSimilarity {
    fn check(&self, doc: &Document) -> RuleResult {
        let texts: Vec<&str> = doc.paragraphs.iter().filter(|p| !p.is_heading).map(|p| p.text.as_str()).collect();
        let mut details = Vec::new();
        for i in 1..texts.len() {
            let sim = text_similarity_pair(texts[i - 1], texts[i]);
            if sim > 0.4 {
                details.push(format!("第 {} 行与第 {} 行内容相似度 {:.0}%，可能重复", doc.paragraphs[i - 1].line_start + 1, doc.paragraphs[i].line_start + 1, sim * 100.0));
            }
        }
        let score = if details.is_empty() { 100.0 } else { f64::max(0.0, 100.0 - details.len() as f64 * 30.0) };
        if details.is_empty() { details.push("相邻段落无显著重复".to_string()); }
        RuleResult { name: "文本相似度", score, max_score: 100.0, details }
    }
}

fn text_similarity_pair(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() { return 1.0; }
    if a.is_empty() || b.is_empty() { return 0.0; }
    let wa: HashSet<&str> = a.split_whitespace().collect();
    let wb: HashSet<&str> = b.split_whitespace().collect();
    let inter = wa.intersection(&wb).count();
    let union = wa.len() + wb.len() - inter;
    if union == 0 { 0.0 } else { inter as f64 / union as f64 }
}

pub struct TableCheck;
impl TextRule for TableCheck {
    fn check(&self, doc: &Document) -> RuleResult {
        let mut invalid = 0;
        let mut details = Vec::new();
        for t in &doc.tables {
            if t.rows.len() < 3 {
                let has_summary = doc.paragraphs.iter().any(|p| !p.is_heading && p.line_start > t.line_start + t.rows.len() && p.text.contains("总结"));
                if !has_summary {
                    invalid += 1;
                    details.push(format!("第 {} 行的表格行数 < 3 且无总结", t.line_start + 1));
                }
            }
        }
        let score = if doc.tables.is_empty() { 100.0 } else { f64::max(0.0, 100.0 - (invalid as f64 / doc.tables.len() as f64) * 100.0) };
        if details.is_empty() { details.push(if doc.tables.is_empty() { "未检测到表格".to_string() } else { "所有表格合理".to_string() }); }
        RuleResult { name: "表格合理性", score, max_score: 100.0, details }
    }
}

pub struct ConceptDensity;
impl TextRule for ConceptDensity {
    fn check(&self, doc: &Document) -> RuleResult {
        let mut seen = HashSet::new();
        let mut details = Vec::new();
        for p in &doc.paragraphs {
            if p.is_heading { continue; }
            let terms: Vec<&str> = p.text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                .filter(|w| w.chars().all(|c| c > '\u{4e00}' && c <= '\u{9fff}') && w.len() >= 2).collect();
            let new: Vec<&str> = terms.into_iter().filter(|t| seen.insert(t.to_string())).collect();
            if new.len() >= 5 {
                details.push(format!("第 {} 行：首次出现 {} 个新术语（{}）", p.line_start + 1, new.len(), new.join(", ")));
            }
        }
        let score = if details.is_empty() { 100.0 } else { f64::max(0.0, 100.0 - details.len() as f64 * 15.0) };
        if details.is_empty() { details.push("概念密度合理".to_string()); }
        RuleResult { name: "概念密度", score, max_score: 100.0, details }
    }
}
