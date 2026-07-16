//! 文档质量检测器 — 支持文本（Markdown 散文）和代码（Rust/通用）的质量评估。
//! 文本指标：标题层级、过渡词、文本相似度、表格合理性、概念密度、逻辑跳跃。
//! 代码指标：函数长度、注释密度、结构复杂度。

use clap::{Args, Subcommand};
use std::collections::HashSet;

const TRANSITION_WORDS: &[&str] = &[
    "但是",
    "因此",
    "例如",
    "然而",
    "所以",
    "不过",
    "而且",
    "此外",
    "总之",
    "也就是说",
    "换句话说",
    "具体来说",
    "另一方面",
    "与此同时",
    "尽管如此",
];

/// 文档类型：文本、代码、混合
#[derive(Debug, Clone, Copy, PartialEq)]
enum DocType {
    Text,
    Code,
    Mixed,
}

/// 根据文件扩展名判断文档类型；无法识别时按内容第一行关键词判断
fn detect_type(path: &str) -> DocType {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "hpp" | "rb" | "kt" | "scala" => DocType::Code,
        "md" | "txt" | "rst" | "markdown" | "adoc" => DocType::Text,
        _ => DocType::Text, // 默认按文本处理
    }
}

/// 根据内容第一行关键词判断文档类型
fn detect_type_from_content(text: &str) -> DocType {
    let first_line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let trimmed = first_line.trim();
    let code_keywords = ["fn ", "def ", "function ", "import ", "pub ", "use ", "#include", "package ", "module "];
    if code_keywords.iter().any(|kw| trimmed.starts_with(kw)) {
        DocType::Code
    } else {
        DocType::Text
    }
}

#[derive(Clone, Args)]
pub struct CheckArgs {
    #[arg(long, default_value = "-")]
    pub input: String,
    #[arg(long, default_value = "normal")]
    pub mode: Mode,
}

#[derive(Clone, Copy, clap::ValueEnum, Default)]
pub enum Mode {
    Summary,
    #[default]
    Normal,
    Verbose,
}

#[derive(clap::Parser)]
#[command(name = "detect")]
pub struct DetectCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Check(CheckArgs),
}

/// 分发检测命令
pub fn dispatch(cli: DetectCli) {
    match cli.command {
        Commands::Check(args) => cmd_check(&args),
    }
}

/// 读取输入（文件或 stdin）
fn read_input(path: &str) -> String {
    if path == "-" {
        std::io::read_to_string(std::io::stdin()).unwrap_or_default()
    } else {
        std::fs::read_to_string(path).unwrap_or_default()
    }
}

/// 根据输入类型路由到不同的检测规则集
fn cmd_check(args: &CheckArgs) {
    let text = read_input(&args.input);
    let dtype = if args.input == "-" {
        detect_type_from_content(&text)
    } else {
        detect_type(&args.input)
    };
    match dtype {
        DocType::Code => {
            let results = run_code_rules(&text);
            print_report(&results, args.mode);
        }
        DocType::Text => {
            let doc = parse_document(&text);
            let results = run_rules(&doc);
            print_report(&results, args.mode);
        }
        DocType::Mixed => {
            let doc = parse_document(&text);
            let mut results = run_rules(&doc);
            results.extend(run_code_rules(&text));
            print_report(&results, args.mode);
        }
    }
}

struct Document {
    paragraphs: Vec<Paragraph>,
    tables: Vec<Table>,
}

struct Paragraph {
    line_start: usize,
    text: String,
    is_heading: bool,
    heading_level: usize,
}

struct Table {
    line_start: usize,
    rows: Vec<Vec<String>>,
}

fn parse_document(text: &str) -> Document {
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
            tables.push(Table {
                line_start: i - rows.len(),
                rows,
            });
            continue;
        } else if !line.trim().is_empty() {
            paragraphs.push(Paragraph {
                line_start: i,
                text: line.to_string(),
                is_heading: false,
                heading_level: 0,
            });
        }
        i += 1;
    }
    Document { paragraphs, tables }
}

fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        let count = trimmed.chars().take_while(|c| *c == '#').count();
        if trimmed.len() > count && trimmed.as_bytes()[count] == b' ' {
            Some(count)
        } else {
            None
        }
    } else {
        None
    }
}

struct RuleResult {
    name: &'static str,
    score: f64,
    max_score: f64,
    details: Vec<String>,
}

fn run_rules(doc: &Document) -> Vec<RuleResult> {
    let mut results = vec![
        title_depth(&doc),
        transition_words(&doc),
        text_similarity(&doc),
        table_check(&doc),
        concept_density(&doc),
    ];
    let avg: f64 =
        results.iter().map(|r| r.score / r.max_score).sum::<f64>() / results.len() as f64;
    if avg > 0.3 && avg < 0.8 {
        results.push(logic_jump(&doc));
    }
    results
}

/// 构建 LLM 请求载荷
fn build_jump_payload(texts: &[(usize, &str)]) -> serde_json::Value {
    serde_json::json!({
        "model": std::env::var("LLM_MODEL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "deepseek-v4-flash".to_string()),
        "messages": [
            {"role": "system", "content": "你是一个文档逻辑检测助手。找出文档中相邻段落之间的逻辑跳跃（突然转换话题、缺少过渡、因果关系断裂）。输出JSON数组，每个元素：{\"index\":段落序号,\"jump\":true/false,\"reason\":\"\"}"},
            {"role": "user", "content": texts.iter().enumerate().map(|(i, (_, t))| format!("{}: {}", i, t)).collect::<Vec<_>>().join("\n\n")}
        ],
        "max_tokens": 300
    })
}

/// 调用 LLM API
fn call_llm(body: &serde_json::Value) -> Result<ureq::Response, ureq::Error> {
    let api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
    let base_url = std::env::var("LLM_BASE_URL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "https://api.deepseek.com".to_string());
    let client = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(5))
        .timeout_read(std::time::Duration::from_secs(10))
        .build();
    client.post(&format!("{}/chat/completions", base_url))
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(body)
}

/// 解析 LLM 响应，提取逻辑跳跃
fn parse_jump_response(resp: ureq::Response, texts: &[(usize, &str)], details: &mut Vec<String>) -> usize {
    let data: serde_json::Value = resp.into_json().unwrap_or_default();
    let content = data["choices"][0]["message"]["content"].as_str().unwrap_or("[]");
    let items: Vec<serde_json::Value> = serde_json::from_str(content).unwrap_or_default();
    for item in &items {
        if item["jump"].as_bool().unwrap_or(false) {
            if let Some(idx) = item["index"].as_u64() {
                let idx = idx as usize;
                if idx > 0 && idx < texts.len() {
                    details.push(format!(
                        "第 {} 行与第 {} 行之间：{}",
                        texts[idx - 1].0 + 1,
                        texts[idx].0 + 1,
                        item["reason"].as_str().unwrap_or("逻辑跳跃")
                    ));
                }
            }
        }
    }
    items.iter().filter(|i| i["jump"].as_bool().unwrap_or(false)).count()
}

fn logic_jump(doc: &Document) -> RuleResult {
    let api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return RuleResult { name: "逻辑跳跃", score: 100.0, max_score: 100.0, details: vec!["未配置 LLM，跳过".to_string()] };
    }
    let texts: Vec<(usize, &str)> = doc.paragraphs.iter().filter(|p| !p.is_heading).map(|p| (p.line_start, p.text.as_str())).collect();
    if texts.len() < 2 {
        return RuleResult { name: "逻辑跳跃", score: 100.0, max_score: 100.0, details: vec!["段落不足".to_string()] };
    }
    let mut details = Vec::new();
    let body = build_jump_payload(&texts);
    let jumps = match call_llm(&body) {
        Ok(resp) => parse_jump_response(resp, &texts, &mut details),
        Err(e) => { details.push(format!("LLM 调用失败: {}", e)); 0 }
    };
    let score = if details.is_empty() { 100.0 } else { f64::max(0.0, 100.0 - jumps as f64 * 30.0) };
    if details.is_empty() { details.push("未检测到逻辑跳跃".to_string()); }
    RuleResult { name: "逻辑跳跃", score, max_score: 100.0, details }
}

fn title_depth(doc: &Document) -> RuleResult {
    let mut max_level = 0;
    for p in &doc.paragraphs {
        if p.is_heading && p.heading_level > max_level {
            max_level = p.heading_level;
        }
    }
    let details = if max_level > 3 {
        vec![format!(
            "标题层级最深为 {} 层（建议不超过 3 层）",
            max_level
        )]
    } else {
        vec![]
    };
    let score = if max_level <= 3 {
        100.0
    } else {
        f64::max(0.0, 100.0 - (max_level as f64 - 3.0) * 20.0)
    };
    RuleResult {
        name: "标题层级",
        score,
        max_score: 100.0,
        details,
    }
}

fn transition_words(doc: &Document) -> RuleResult {
    let mut count = 0;
    let mut details = Vec::new();
    for p in &doc.paragraphs {
        if p.is_heading {
            continue;
        }
        for w in TRANSITION_WORDS {
            if p.text.find(w).is_some() {
                let line_no = p.line_start + 1;
                details.push(format!("第 {} 行：出现过渡词「{}」", line_no, w));
                count += 1;
                break;
            }
        }
    }
    let total = doc.paragraphs.iter().filter(|p| !p.is_heading).count();
    let ratio = if total == 0 {
        1.0
    } else {
        count as f64 / total as f64
    };
    let score = f64::min(100.0, ratio * 200.0);
    if details.is_empty() {
        details.push("未检测到过渡词".to_string());
    }
    RuleResult {
        name: "过渡词使用",
        score,
        max_score: 100.0,
        details,
    }
}

fn text_similarity(doc: &Document) -> RuleResult {
    let texts: Vec<&str> = doc
        .paragraphs
        .iter()
        .filter(|p| !p.is_heading)
        .map(|p| p.text.as_str())
        .collect();
    let mut details = Vec::new();
    for i in 1..texts.len() {
        let sim = text_similarity_pair(texts[i - 1], texts[i]);
        if sim > 0.6 {
            details.push(format!(
                "第 {} 行与第 {} 行内容相似度 {:.0}%, 可能重复",
                doc.paragraphs[i - 1].line_start + 1,
                doc.paragraphs[i].line_start + 1,
                sim * 100.0
            ));
        }
    }
    let score = if details.is_empty() {
        100.0
    } else {
        f64::max(0.0, 100.0 - details.len() as f64 * 25.0)
    };
    if details.is_empty() {
        details.push("相邻段落无显著重复".to_string());
    }
    RuleResult {
        name: "文本相似度",
        score,
        max_score: 100.0,
        details,
    }
}

fn text_similarity_pair(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();
    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.len() + words_b.len() - intersection;
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn table_check(doc: &Document) -> RuleResult {
    let mut invalid = 0;
    let mut details = Vec::new();
    for t in &doc.tables {
        if t.rows.len() < 3 {
            let has_summary = doc.paragraphs.iter().any(|p| {
                !p.is_heading
                    && p.line_start > t.line_start + t.rows.len()
                    && p.text.contains("总结")
            });
            if !has_summary {
                invalid += 1;
                details.push(format!("第 {} 行的表格行数 < 3 且无总结", t.line_start + 1));
            }
        }
    }
    let score = if doc.tables.is_empty() {
        100.0
    } else {
        f64::max(
            0.0,
            100.0 - (invalid as f64 / doc.tables.len() as f64) * 100.0,
        )
    };
    if details.is_empty() {
        if doc.tables.is_empty() {
            details.push("未检测到表格".to_string());
        } else {
            details.push("所有表格合理".to_string());
        }
    }
    RuleResult {
        name: "表格合理性",
        score,
        max_score: 100.0,
        details,
    }
}

fn concept_density(doc: &Document) -> RuleResult {
    let mut seen = HashSet::new();
    let mut details = Vec::new();
    for p in &doc.paragraphs {
        if p.is_heading {
            continue;
        }
        let terms: Vec<&str> = p
            .text
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
            .filter(|w| w.chars().all(|c| c > '\u{4e00}' && c <= '\u{9fff}') && w.len() >= 2)
            .collect();
        let mut new_terms = Vec::new();
        for t in &terms {
            if seen.insert(t.to_string()) {
                new_terms.push(*t);
            }
        }
        if new_terms.len() >= 5 {
            details.push(format!(
                "第 {} 行：首次出现 {} 个新术语（{}）",
                p.line_start + 1,
                new_terms.len(),
                new_terms.join(", ")
            ));
        }
    }
    let score = if details.is_empty() {
        100.0
    } else {
        f64::max(0.0, 100.0 - details.len() as f64 * 15.0)
    };
    if details.is_empty() {
        details.push("概念密度合理".to_string());
    }
    RuleResult {
        name: "概念密度",
        score,
        max_score: 100.0,
        details,
    }
}

// ── 代码指标 ──────────────────────────────────────────

fn run_code_rules(text: &str) -> Vec<RuleResult> {
    vec![
        check_function_length(text),
        check_comment_density(text),
        check_structural_complexity(text),
    ]
}

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

/// 指标一：函数长度
fn check_function_length(text: &str) -> RuleResult {
    let lines: Vec<&str> = text.lines().collect();
    let fn_starts = find_fn_positions(&lines);
    if fn_starts.is_empty() {
        return RuleResult { name: "函数长度", score: 100.0, max_score: 100.0, details: vec!["未检测到函数定义".to_string()] };
    }
    let (mut details, worst) = score_fn_body_lengths(&fn_starts, &lines);
    let score = worst * 100.0;
    if details.is_empty() { details.push("所有函数长度合理".to_string()); }
    RuleResult { name: "函数长度", score, max_score: 100.0, details }
}

/// 查找缺少文档注释的公开函数
fn find_undocumented_pub_fns(lines: &[&str]) -> Vec<String> {
    let mut missing = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("pub fn ") || t.starts_with("pub(crate) fn ") {
            let has_doc = (1..=3).any(|off| i >= off && lines[i - off].trim().starts_with("///"));
            if !has_doc { missing.push(t.split_whitespace().nth(2).unwrap_or("?").to_string()); }
        }
    }
    missing
}

/// 指标二：注释密度
fn check_comment_density(text: &str) -> RuleResult {
    let lines: Vec<&str> = text.lines().collect();
    let total = lines.len();
    if total == 0 {
        return RuleResult { name: "注释密度", score: 100.0, max_score: 100.0, details: vec!["空文件".to_string()] };
    }
    let comment_lines = lines.iter().filter(|l| {
        let t = l.trim();
        t.starts_with("//") || t.starts_with("///") || t.starts_with("/*") || t.starts_with("* ") || t.starts_with("#") || t.starts_with("--")
    }).count();
    let ratio = comment_lines as f64 / total as f64;
    let missing_doc = find_undocumented_pub_fns(&lines);
    let mut details = Vec::new();
    let mut score = 100.0;
    if ratio < 0.05 {
        details.push(format!("注释行占比 {:.1}%（低于建议的 5%）", ratio * 100.0));
        score -= 30.0;
    }
    if !missing_doc.is_empty() {
        details.push(format!("{} 个公开函数缺少文档注释：{}", missing_doc.len(), missing_doc.join(", ")));
        score -= missing_doc.len() as f64 * 15.0;
    }
    let score = f64::max(0.0, score);
    if details.is_empty() { details.push("注释密度合理".to_string()); }
    RuleResult { name: "注释密度", score, max_score: 100.0, details }
}

/// 检测最大嵌套深度
fn detect_nesting(lines: &[&str]) -> usize {
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

/// 检测圈复杂度超过 10 的函数
fn detect_high_complexity(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let mut in_fn = false;
    let mut start = 0;
    let mut comp = 0;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if is_fn_def(t) {
            // 上一个函数收尾
            if in_fn && comp > 10 { results.push((start, comp)); }
            start = i; comp = 0; in_fn = true;
        } else if in_fn {
            comp += branch_kw_count(t);
        }
    }
    // 最后一个函数收尾
    if in_fn && comp > 10 { results.push((start, comp)); }
    results
}

/// 分析命名熵：标识符长度分布
fn detect_naming(text: &str) -> (f64, usize, usize) {
    let ids: Vec<&str> = text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty() && w.len() >= 2 && w.len() <= 40
            && (w.starts_with(|c: char| c.is_ascii_lowercase()) || w.starts_with(|c: char| c.is_ascii_uppercase()) || w.starts_with('_')))
        .collect();
    if ids.is_empty() { return (0.0, 0, 0); }
    let avg = ids.iter().map(|w| w.len()).sum::<usize>() as f64 / ids.len() as f64;
    (avg, ids.iter().map(|w| w.len()).min().unwrap(), ids.iter().map(|w| w.len()).max().unwrap())
}

/// 指标三：结构复杂度
fn check_structural_complexity(text: &str) -> RuleResult {
    let lines: Vec<&str> = text.lines().collect();
    let mut details = Vec::new();
    let mut score = 100.0;
    let max_indent = detect_nesting(&lines);
    if max_indent > 4 {
        details.push(format!("最大嵌套深度 {} 层（建议不超过 4 层）", max_indent));
        score -= f64::min(30.0, (max_indent as f64 - 4.0) * 10.0);
    }
    for (line_no, comp) in &detect_high_complexity(&lines) {
        let name = lines[*line_no].trim().split_whitespace().nth(1).unwrap_or("?");
        details.push(format!("函数 `{}`（第 {} 行）圈复杂度 {}（建议不超过 10）", name, line_no + 1, comp));
        score -= f64::min(30.0, (*comp as f64 - 10.0) * 5.0);
    }
    let (avg_len, min_len, max_len) = detect_naming(text);
    if avg_len > 0.0 {
        if avg_len < 4.0 {
            details.push(format!("命名长度偏短（平均 {:.1} 字符），可能存在含义不明的缩写", avg_len));
            score -= 15.0;
        } else if avg_len > 20.0 {
            details.push(format!("命名长度偏长（平均 {:.1} 字符），可能存在冗余命名", avg_len));
            score -= 15.0;
        }
        if min_len >= 2 && max_len - min_len > 25 {
            details.push(format!("命名长度跨度大（最短 {} 字符，最长 {} 字符），风格不一致", min_len, max_len));
            score -= 10.0;
        }
    }
    let score = f64::max(0.0, score);
    if details.is_empty() { details.push("结构复杂度合理".to_string()); }
    RuleResult { name: "结构复杂度", score, max_score: 100.0, details }
}

// ── 报告输出 ──────────────────────────────────────────

/// 输出检测报告
fn print_report(results: &[RuleResult], mode: Mode) {
    let total: f64 = results.iter().map(|r| r.score).sum();
    let max: f64 = results.iter().map(|r| r.max_score).sum();
    let pct = total / max * 100.0;
    let level = if pct >= 80.0 {
        "低"
    } else {
        if pct >= 60.0 {
            "中"
        } else {
            "高"
        }
    };
    match mode {
        Mode::Summary => println!("{}% ({})", pct as u32, level),
        Mode::Normal => {
            println!("{}% ({})", pct as u32, level);
            for r in results {
                let p = r.score / r.max_score * 100.0;
                if p < 80.0 {
                    println!("  {}:{:.0}%", r.name, p);
                }
            }
        }
        Mode::Verbose => {
            println!("{}% ({})", pct as u32, level);
            for r in results {
                let p = r.score / r.max_score * 100.0;
                if p < 80.0 {
                    println!("  {}:{:.0}%", r.name, p);
                    for d in &r.details {
                        println!("    {}", d);
                    }
                }
            }
        }
    }
}
