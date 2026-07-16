//! 代码与模块级检测指标
//! 函数级：函数长度、API 文档覆盖率、结构复杂度（嵌套+圈复杂度+命名熵）
//! 模块级：文件长度、模块文档存在性、模块耦合度（use 数量）

use crate::detect::{CodeRule, RuleResult};

// ── 函数级指标 ──────────────────────────────────────

// 指标一：函数长度
pub struct FunctionLength;
impl CodeRule for FunctionLength {
    fn check(&self, source: &str) -> RuleResult {
        let lines: Vec<&str> = source.lines().collect();
        let fn_starts = find_fn_positions(&lines);
        if fn_starts.is_empty() {
            return RuleResult { name: "函数长度", score: 100.0, max_score: 100.0, details: vec!["未检测到函数定义".to_string()] };
        }
        let (mut details, worst) = score_fn_body_lengths(&fn_starts, &lines);
        let score = worst * 100.0;
        if details.is_empty() { details.push("所有函数长度合理".to_string()); }
        RuleResult { name: "函数长度", score, max_score: 100.0, details }
    }
}

// 指标二：API 文档覆盖率
pub struct ApiDocCoverage;
impl CodeRule for ApiDocCoverage {
    fn check(&self, source: &str) -> RuleResult {
        let lines: Vec<&str> = source.lines().collect();
        let missing = find_missing_doc_comments(&lines);
        let mut details = Vec::new();
        let mut score = 100.0;
        if !missing.is_empty() {
            details.push(format!("{} 个公开项缺少文档注释：{}", missing.len(), missing.join(", ")));
            score -= missing.len() as f64 * 15.0;
        }
        let score = f64::max(0.0, score);
        if details.is_empty() { details.push("API 文档注释完整".to_string()); }
        RuleResult { name: "API 文档覆盖率", score, max_score: 100.0, details }
    }
}

// 指标三：结构复杂度（嵌套深度 + 圈复杂度 + 命名熵）
pub struct StructuralComplexity;
impl CodeRule for StructuralComplexity {
    fn check(&self, source: &str) -> RuleResult {
        let lines: Vec<&str> = source.lines().collect();
        let mut details = Vec::new();
        let mut score = 100.0;

        let max_indent = nesting_depth(&lines);
        if max_indent > 4 {
            details.push(format!("最大嵌套深度 {} 层（建议不超过 4 层）", max_indent));
            score -= f64::min(30.0, (max_indent as f64 - 4.0) * 10.0);
        }

        for (line_no, comp) in &high_complexity_fns(&lines) {
            let name = lines[*line_no].trim().split_whitespace().nth(1).unwrap_or("?");
            details.push(format!("函数 `{}`（第 {} 行）圈复杂度 {}（建议不超过 10）", name, line_no + 1, comp));
            score -= f64::min(30.0, (*comp as f64 - 10.0) * 5.0);
        }

        let (avg, min, max) = naming_entropy(source);
        if avg > 0.0 {
            if avg < 4.0 {
                details.push(format!("命名平均长度 {:.1} 字符（偏短），可能存在含义不明的缩写", avg));
                score -= 15.0;
            } else if avg > 20.0 {
                details.push(format!("命名平均长度 {:.1} 字符（偏长），可能存在冗余命名", avg));
                score -= 15.0;
            }
            if min >= 2 && max - min > 25 {
                details.push(format!("命名长度跨度大（{}–{}），风格不一致", min, max));
                score -= 10.0;
            }
        }

        let score = f64::max(0.0, score);
        if details.is_empty() { details.push("结构复杂度合理".to_string()); }
        RuleResult { name: "结构复杂度", score, max_score: 100.0, details }
    }
}

// ── 模块级指标 ──────────────────────────────────────

// 指标四：文件长度
pub struct FileLength;
impl CodeRule for FileLength {
    fn check(&self, source: &str) -> RuleResult {
        let total = source.lines().count();
        let (details, score) = if total > 300 {
            (vec![format!("文件共 {} 行（建议不超过 300 行）", total)],
             f64::max(0.0, 100.0 - (total as f64 - 300.0) * 0.5))
        } else { (vec![], 100.0) };
        let mut d = details;
        if d.is_empty() { d.push(format!("文件长度合理（{} 行）", total)); }
        RuleResult { name: "文件长度", score, max_score: 100.0, details: d }
    }
}

// 指标五：模块文档存在性
pub struct ModDocPresence;
impl CodeRule for ModDocPresence {
    fn check(&self, source: &str) -> RuleResult {
        let has_mod_doc = source.lines().take(10).any(|l| l.trim().starts_with("//!"));
        if has_mod_doc {
            RuleResult { name: "模块文档", score: 100.0, max_score: 100.0, details: vec!["文件头包含模块文档（//!）".to_string()] }
        } else {
            RuleResult { name: "模块文档", score: 0.0, max_score: 100.0, details: vec!["文件头缺少模块文档（//!）".to_string()] }
        }
    }
}

// 指标六：模块耦合度
pub struct ModuleCoupling;
impl CodeRule for ModuleCoupling {
    fn check(&self, source: &str) -> RuleResult {
        let count = source.lines().filter(|l| {
            let t = l.trim();
            t.starts_with("use ") || t.starts_with("use crate::") || t.starts_with("use std::")
                || t.starts_with("extern crate ")
        }).count();
        let (details, score) = if count > 15 {
            (vec![format!("use 声明 {} 个（建议不超过 15 个）", count)],
             f64::max(0.0, 100.0 - (count as f64 - 15.0) * 10.0))
        } else { (vec![], 100.0) };
        let mut d = details;
        if d.is_empty() { d.push(format!("模块耦合度合理（{} 个 use）", count)); }
        RuleResult { name: "模块耦合度", score, max_score: 100.0, details: d }
    }
}

// ── 共享工具函数 ────────────────────────────────────

/// 查找函数定义行号（先试 Rust fn，再试其他语言）
fn find_fn_positions(lines: &[&str]) -> Vec<usize> {
    let rust: Vec<_> = lines.iter().enumerate()
        .filter(|(_, l)| { let t = l.trim(); (t.starts_with("fn ") || t.starts_with("pub fn ") || t.starts_with("pub(crate) fn ")) && !t.starts_with("//") && !t.starts_with("///") })
        .map(|(i, _)| i).collect();
    if !rust.is_empty() { return rust; }
    lines.iter().enumerate()
        .filter(|(_, l)| { let t = l.trim(); (t.starts_with("def ") || t.starts_with("function ") || t.starts_with("func ")) && !t.starts_with("//") && !t.starts_with("#") && !t.starts_with("--") })
        .map(|(i, _)| i).collect()
}

/// 评分每个函数体长度：< 40 → 1.0, 40–80 → 线性下降, > 80 → 0.0
fn score_fn_body_lengths(starts: &[usize], lines: &[&str]) -> (Vec<String>, f64) {
    let mut details = Vec::new();
    let mut worst = 1.0;
    for (idx, &start) in starts.iter().enumerate() {
        let end = if idx + 1 < starts.len() { starts[idx + 1] } else { lines.len() };
        let n = end - start;
        let name = lines[start].trim().split_whitespace().nth(1).unwrap_or("?");
        let r = if n > 80 { 0.0 } else if n > 40 { 1.0 - (n - 40) as f64 / 40.0 } else { 1.0 };
        if r < worst { worst = r; }
        if n > 40 {
            details.push(format!("函数 `{}` 共 {} 行（{}超过建议上限 40 行）", name, n, if n > 80 { "大幅" } else { "" }));
        }
    }
    (details, worst)
}

/// 收集缺少 `///` 文档注释的公开函数
fn find_missing_doc_comments(lines: &[&str]) -> Vec<String> {
    let mut missing = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("pub fn ") || t.starts_with("pub(crate) fn ") {
            let has_doc = (1..=3).any(|off| i >= off && lines[i - off].trim().starts_with("///"));
            if !has_doc {
                missing.push(t.split_whitespace().nth(2).unwrap_or("?").to_string());
            }
        }
    }
    missing
}

/// 最大缩进层级（嵌套深度）
fn nesting_depth(lines: &[&str]) -> usize {
    lines.iter().filter_map(|line| {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || t.starts_with("///") || t.starts_with("/*") { return None; }
        let indent = line.len() - line.trim_start().len();
        Some(if indent > 0 && line.starts_with('\t') { indent } else { indent / 4 })
    }).max().unwrap_or(0)
}

/// 判断行是否为函数定义
fn is_fn_def(t: &str) -> bool {
    t.starts_with("fn ") || t.starts_with("pub fn ") || t.starts_with("def ") || t.starts_with("function ") || t.starts_with("func ")
}

/// 统计行中分支关键字出现次数
fn branch_kw_count(t: &str) -> usize {
    ["if ", "else if ", "for ", "while ", "match ", "case ", "catch ", "except"]
        .iter().filter(|kw| t.contains(*kw)).count()
}

/// 查找圈复杂度超过 10 的函数
fn high_complexity_fns(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let mut in_fn = false;
    let mut start = 0;
    let mut comp = 0;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if is_fn_def(t) {
            if in_fn && comp > 10 { results.push((start, comp)); }
            start = i; comp = 0; in_fn = true;
        } else if in_fn {
            comp += branch_kw_count(t);
        }
    }
    if in_fn && comp > 10 { results.push((start, comp)); }
    results
}

/// 分析命名熵：标识符长度分布
fn naming_entropy(text: &str) -> (f64, usize, usize) {
    let ids: Vec<&str> = text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty() && w.len() >= 2 && w.len() <= 40
            && (w.starts_with(|c: char| c.is_ascii_lowercase()) || w.starts_with(|c: char| c.is_ascii_uppercase()) || w.starts_with('_')))
        .collect();
    if ids.is_empty() { return (0.0, 0, 0); }
    let avg = ids.iter().map(|w| w.len()).sum::<usize>() as f64 / ids.len() as f64;
    (avg, ids.iter().map(|w| w.len()).min().unwrap(), ids.iter().map(|w| w.len()).max().unwrap())
}
