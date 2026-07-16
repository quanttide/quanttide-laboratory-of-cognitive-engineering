//! 文档质量检测器 — 文本质量评估
//! 指标：标题层级、过渡词、文本相似度、表格合理性、概念密度、逻辑跳跃

pub mod llm;
pub mod report;
pub mod text;

use clap::{Args, Subcommand};

// ── 领域类型 ──────────────────────────────────────────

/// 文档类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocType {
    Text,
    Code,
}

/// 单条规则结果
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub name: &'static str,
    pub score: f64,
    pub max_score: f64,
    pub details: Vec<String>,
}

/// 规则 trait — 文本指标
pub trait TextRule: Send + Sync {
    fn check(&self, doc: &text::Document) -> RuleResult;
}

/// 注册的规则集
pub struct RuleSet {
    text_rules: Vec<Box<dyn TextRule>>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self { text_rules: Vec::new() }
    }

    pub fn add_text(&mut self, rule: Box<dyn TextRule>) -> &mut Self {
        self.text_rules.push(rule);
        self
    }

    pub fn run_text(&self, doc: &text::Document) -> Vec<RuleResult> {
        self.text_rules.iter().map(|r| r.check(doc)).collect()
    }
}

/// 构建默认规则集（仅文本）
pub fn default_rules() -> RuleSet {
    let mut rs = RuleSet::new();
    rs.add_text(Box::new(text::TitleDepth));
    rs.add_text(Box::new(text::TransitionWords));
    rs.add_text(Box::new(text::TextSimilarity));
    rs.add_text(Box::new(text::TableCheck));
    rs.add_text(Box::new(text::ConceptDensity));
    rs.add_text(Box::new(llm::LogicJump));
    rs
}

// ── CLI ──────────────────────────────────────────────

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

const CODE_EXTS: &[&str] = &[
    "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "hpp", "rb", "kt", "scala",
];
const TEXT_EXTS: &[&str] = &["md", "txt", "rst", "markdown", "adoc"];

/// 按扩展名判断文档类型
fn detect_type(path: &str) -> DocType {
    let ext = path.rsplit('.').next().unwrap_or("");
    if CODE_EXTS.contains(&ext) { DocType::Code }
    else if TEXT_EXTS.contains(&ext) { DocType::Text }
    else { DocType::Text }
}

/// 按内容第一行关键词判断文档类型
fn detect_type_from_content(text: &str) -> DocType {
    let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    let kws = ["fn ", "def ", "function ", "import ", "pub ", "use ", "#include", "package "];
    if kws.iter().any(|kw| first.starts_with(kw)) { DocType::Code } else { DocType::Text }
}

/// 根据输入类型路由到检测规则
fn cmd_check(args: &CheckArgs) {
    let text = read_input(&args.input);
    let dtype = if args.input == "-" { detect_type_from_content(&text) } else { detect_type(&args.input) };
    let rules = &mut default_rules();
    match dtype {
        DocType::Code => {
            eprintln!("⚠ 代码审计已迁移至 qtcloud-devops code audit");
            std::process::exit(1);
        }
        DocType::Text => {
            let doc = text::parse(&text);
            let mut results = rules.run_text(&doc);
            let avg = results.iter().map(|r| r.score / r.max_score).sum::<f64>()
                / results.len().max(1) as f64;
            if avg <= 0.3 || avg >= 0.8 {
                results.retain(|r| r.name != "逻辑跳跃");
            }
            report::print(&results, args.mode);
        }
    }
}
