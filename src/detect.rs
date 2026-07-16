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

pub fn dispatch(cli: DetectCli) {
    match cli.command {
        Commands::Check(args) => cmd_check(&args),
    }
}

fn cmd_check(args: &CheckArgs) {
    let text = if args.input == "-" {
        std::io::read_to_string(std::io::stdin()).unwrap_or_default()
    } else {
        std::fs::read_to_string(&args.input).unwrap_or_default()
    };
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

fn logic_jump(doc: &Document) -> RuleResult {
    let api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return RuleResult {
            name: "逻辑跳跃",
            score: 100.0,
            max_score: 100.0,
            details: vec!["未配置 LLM，跳过".to_string()],
        };
    }
    let mut details = Vec::new();
    let texts: Vec<(usize, &str)> = doc
        .paragraphs
        .iter()
        .filter(|p| !p.is_heading)
        .map(|p| (p.line_start, p.text.as_str()))
        .collect();
    if texts.len() < 2 {
        return RuleResult {
            name: "逻辑跳跃",
            score: 100.0,
            max_score: 100.0,
            details: vec!["段落不足".to_string()],
        };
    }
    let body = serde_json::json!({
        "model": std::env::var("LLM_MODEL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "deepseek-v4-flash".to_string()),
        "messages": [
            {"role": "system", "content": "你是一个文档逻辑检测助手。找出文档中相邻段落之间的逻辑跳跃（突然转换话题、缺少过渡、因果关系断裂）。输出JSON数组，每个元素：{\"index\":段落序号,\"jump\":true/false,\"reason\":\"\"}"},
            {"role": "user", "content": texts.iter().enumerate().map(|(i, (_, t))| format!("{}: {}", i, t)).collect::<Vec<_>>().join("\n\n")}
        ],
        "max_tokens": 300
    });
    let client = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(5))
        .timeout_read(std::time::Duration::from_secs(10))
        .build();
    let base_url = std::env::var("LLM_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "https://api.deepseek.com".to_string());
    let resp = client
        .post(&format!("{}/chat/completions", base_url))
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(&body);
    let jumps = match resp {
        Ok(r) => {
            let data: serde_json::Value = r.into_json().unwrap_or_default();
            let content = data["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("[]");
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
            items
                .iter()
                .filter(|i| i["jump"].as_bool().unwrap_or(false))
                .count()
        }
        Err(e) => {
            details.push(format!("LLM 调用失败: {}", e));
            0
        }
    };
    let score = if details.is_empty() {
        100.0
    } else {
        f64::max(0.0, 100.0 - jumps as f64 * 30.0)
    };
    if details.is_empty() {
        details.push("未检测到逻辑跳跃".to_string());
    }
    RuleResult {
        name: "逻辑跳跃",
        score,
        max_score: 100.0,
        details,
    }
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

/// 指标一：函数长度 — 统计每个函数体的行数
fn check_function_length(text: &str) -> RuleResult {
    let lines: Vec<&str> = text.lines().collect();
    let mut fn_starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            let t = l.trim();
            (t.starts_with("fn ") || t.starts_with("pub fn ") || t.starts_with("pub(crate) fn "))
                && !t.starts_with("//")
                && !t.starts_with("///")
        })
        .map(|(i, _)| i)
        .collect();

    if fn_starts.is_empty() {
        // 尝试其他语言的关键字
        fn_starts = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| {
                let t = l.trim();
                (t.starts_with("def ")
                    || t.starts_with("function ")
                    || t.starts_with("func ")
                    || t.starts_with("pub fn "))
                    && !t.starts_with("//")
                    && !t.starts_with("#")
                    && !t.starts_with("--")
            })
            .map(|(i, _)| i)
            .collect();
    }

    if fn_starts.is_empty() {
        return RuleResult {
            name: "函数长度",
            score: 100.0,
            max_score: 100.0,
            details: vec!["未检测到函数定义".to_string()],
        };
    }

    let mut details = Vec::new();
    let mut worst_ratio = 1.0;

    for (idx, &start) in fn_starts.iter().enumerate() {
        let end = if idx + 1 < fn_starts.len() {
            fn_starts[idx + 1]
        } else {
            lines.len()
        };
        let func_lines = end - start;
        let func_name = lines[start]
            .trim()
            .split_whitespace()
            .nth(1)
            .unwrap_or("?");

        let ratio = if func_lines > 80 {
            0.0
        } else if func_lines > 40 {
            1.0 - (func_lines - 40) as f64 / 40.0
        } else {
            1.0
        };

        if ratio < worst_ratio {
            worst_ratio = ratio;
        }

        if func_lines > 40 {
            let level = if func_lines > 80 {
                "大幅"
            } else {
                ""
            };
            details.push(format!(
                "函数 `{}` 共 {} 行（{}超过建议上限 40 行）",
                func_name, func_lines, level
            ));
        }
    }

    let score = worst_ratio * 100.0;
    if details.is_empty() {
        details.push("所有函数长度合理".to_string());
    }
    RuleResult {
        name: "函数长度",
        score,
        max_score: 100.0,
        details,
    }
}

/// 指标二：注释密度 — 注释行占比 + 公开 API 文档注释检查
fn check_comment_density(text: &str) -> RuleResult {
    let lines: Vec<&str> = text.lines().collect();
    let total = lines.len();
    if total == 0 {
        return RuleResult {
            name: "注释密度",
            score: 100.0,
            max_score: 100.0,
            details: vec!["空文件".to_string()],
        };
    }

    // 统计注释行
    let comment_lines = lines
        .iter()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("//") || t.starts_with("///") || t.starts_with("/*") || t.starts_with("* ")
                || t.starts_with("#") || t.starts_with("--")
        })
        .count();

    let ratio = comment_lines as f64 / total as f64;
    let mut details = Vec::new();

    // 检查公开函数是否有文档注释
    let mut missing_doc = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("pub fn ") || t.starts_with("pub(crate) fn ") {
            // 向前看是否有 `///` 注释块
            let has_doc = (1..=3).any(|offset| {
                if i >= offset {
                    let prev = lines[i - offset].trim();
                    prev.starts_with("///")
                } else {
                    false
                }
            });
            if !has_doc {
                let name = t.split_whitespace().nth(2).unwrap_or("?");
                missing_doc.push(name.to_string());
            }
        }
    }

    let mut score = 100.0;

    if ratio < 0.05 {
        details.push(format!(
            "注释行占比 {:.1}%（低于建议的 5%）",
            ratio * 100.0
        ));
        score -= 30.0;
    }

    if !missing_doc.is_empty() {
        details.push(format!(
            "{} 个公开函数缺少文档注释：{}",
            missing_doc.len(),
            missing_doc.join(", ")
        ));
        score -= missing_doc.len() as f64 * 15.0;
    }

    let score = f64::max(0.0, score);
    if details.is_empty() {
        details.push("注释密度合理".to_string());
    }
    RuleResult {
        name: "注释密度",
        score,
        max_score: 100.0,
        details,
    }
}

/// 指标三：结构复杂度 — 嵌套深度 + 圈复杂度 + 命名熵
fn check_structural_complexity(text: &str) -> RuleResult {
    let lines: Vec<&str> = text.lines().collect();
    let mut details = Vec::new();
    let mut score = 100.0;

    // 1. 嵌套深度检测（按缩进层级）
    let mut max_indent = 0;
    for line in &lines {
        let indent = line.len() - line.trim_start().len();
        // 跳过纯注释和空行
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || t.starts_with("///") || t.starts_with("/*") {
            continue;
        }
        // 缩进以 4 空格或 1 tab 为一级
        let level = if indent > 0 && line.starts_with('\t') {
            indent // tab = 1 级
        } else {
            indent / 4
        };
        if level > max_indent {
            max_indent = level;
        }
    }
    if max_indent > 4 {
        details.push(format!(
            "最大嵌套深度 {} 层（建议不超过 4 层）",
            max_indent
        ));
        score -= f64::min(30.0, (max_indent as f64 - 4.0) * 10.0);
    }

    // 2. 圈复杂度（按分支关键字计数）
    let branch_kws = ["if ", "else if ", "for ", "while ", "match ", "case ", "catch ", "except"];
    let mut _total_complexity = 0;
    let mut fn_complexities: Vec<(usize, usize)> = Vec::new();
    let mut fn_start = 0;
    let mut fn_complexity = 0;
    let mut in_fn = false;

    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("fn ")
            || t.starts_with("pub fn ")
            || t.starts_with("def ")
            || t.starts_with("function ")
            || t.starts_with("func ")
        {
            if in_fn && fn_complexity > 10 {
                fn_complexities.push((fn_start, fn_complexity));
            }
            fn_start = i;
            fn_complexity = 0;
            in_fn = true;
        }
        if in_fn {
            for kw in &branch_kws {
                if t.contains(kw) {
                    fn_complexity += 1;
                    _total_complexity += 1;
                }
            }
        }
    }
    if in_fn && fn_complexity > 10 {
        fn_complexities.push((fn_start, fn_complexity));
    }

    for (line_no, comp) in &fn_complexities {
        let func_name = lines[*line_no].trim().split_whitespace().nth(1).unwrap_or("?");
        details.push(format!(
            "函数 `{}`（第 {} 行）圈复杂度 {}（建议不超过 10）",
            func_name,
            line_no + 1,
            comp
        ));
        score -= f64::min(30.0, (*comp as f64 - 10.0) * 5.0);
    }

    // 3. 命名熵检测（标识符长度分布）
    let identifiers: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| {
            !w.is_empty()
                && (w.starts_with(|c: char| c.is_ascii_lowercase())
                    || w.starts_with(|c: char| c.is_ascii_uppercase())
                    || w.starts_with('_'))
                && w.len() >= 2
                && w.len() <= 40
        })
        .collect();

    if !identifiers.is_empty() {
        let avg_len = identifiers.iter().map(|w| w.len()).sum::<usize>() as f64 / identifiers.len() as f64;
        let min_len = identifiers.iter().map(|w| w.len()).min().unwrap_or(0);
        let max_len = identifiers.iter().map(|w| w.len()).max().unwrap_or(0);

        // 命名熵异常：全短名（avg < 4）或全长名（avg > 20）
        if avg_len < 4.0 {
            details.push(format!(
                "命名长度偏短（平均 {:.1} 字符），可能存在含义不明的缩写",
                avg_len
            ));
            score -= 15.0;
        } else if avg_len > 20.0 {
            details.push(format!(
                "命名长度偏长（平均 {:.1} 字符），可能存在冗余命名",
                avg_len
            ));
            score -= 15.0;
        }

        // 命名跨度异常
        if min_len >= 2 && max_len - min_len > 25 {
            details.push(format!(
                "命名长度跨度大（最短 {} 字符，最长 {} 字符），风格不一致",
                min_len, max_len
            ));
            score -= 10.0;
        }
    }

    let score = f64::max(0.0, score);
    if details.is_empty() {
        details.push("结构复杂度合理".to_string());
    }
    RuleResult {
        name: "结构复杂度",
        score,
        max_score: 100.0,
        details,
    }
}

// ── 报告输出 ──────────────────────────────────────────

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
