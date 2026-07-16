//! 文档质量检测器 — 文本 & 代码质量评估
//! 文本指标：标题层级、过渡词、文本相似度、表格合理性、概念密度、逻辑跳跃
//! 代码指标：函数长度、API 文档覆盖率、结构复杂度、文件长度、模块耦合

pub mod code;
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
    Mixed,
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
/// 规则 trait — 文本指标
pub trait TextRule: Send + Sync {
    fn check(&self, doc: &text::Document) -> RuleResult;
}

/// 规则 trait — 代码指标
pub trait CodeRule: Send + Sync {
    fn check(&self, source: &str) -> RuleResult;
}

/// 注册的规则集
pub struct RuleSet {
    text_rules: Vec<Box<dyn TextRule>>,
    code_rules: Vec<Box<dyn CodeRule>>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self { text_rules: Vec::new(), code_rules: Vec::new() }
    }

    pub fn add_text(&mut self, rule: Box<dyn TextRule>) -> &mut Self {
        self.text_rules.push(rule);
        self
    }

    pub fn add_code(&mut self, rule: Box<dyn CodeRule>) -> &mut Self {
        self.code_rules.push(rule);
        self
    }

    pub fn run_text(&self, doc: &text::Document) -> Vec<RuleResult> {
        self.text_rules.iter().map(|r| r.check(doc)).collect()
    }

    pub fn run_code(&self, source: &str) -> Vec<RuleResult> {
        self.code_rules.iter().map(|r| r.check(source)).collect()
    }
}

/// 构建默认规则集
pub fn default_rules() -> RuleSet {
    let mut rs = RuleSet::new();
    // 文本
    rs.add_text(Box::new(text::TitleDepth));
    rs.add_text(Box::new(text::TransitionWords));
    rs.add_text(Box::new(text::TextSimilarity));
    rs.add_text(Box::new(text::TableCheck));
    rs.add_text(Box::new(text::ConceptDensity));
    rs.add_text(Box::new(llm::LogicJump));
    // 代码
    rs.add_code(Box::new(code::FunctionLength));
    rs.add_code(Box::new(code::ApiDocCoverage));
    rs.add_code(Box::new(code::StructuralComplexity));
    rs.add_code(Box::new(code::FileLength));
    rs.add_code(Box::new(code::ModDocPresence));
    rs.add_code(Box::new(code::ModuleCoupling));
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
    let mut results = Vec::new();
    match dtype {
        DocType::Code => results.extend(rules.run_code(&text)),
        DocType::Text => {
            let doc = text::parse(&text);
            results.extend(rules.run_text(&doc));
            // logic_jump 仅在基础指标接近阈值时触发
            let avg = results.iter().map(|r| r.score / r.max_score).sum::<f64>()
                / results.len().max(1) as f64;
            if avg <= 0.3 || avg >= 0.8 {
                results.retain(|r| r.name != "逻辑跳跃");
            }
        }
        DocType::Mixed => {
            let doc = text::parse(&text);
            results.extend(rules.run_text(&doc));
            results.extend(rules.run_code(&text));
        }
    }
    report::print(&results, args.mode);
}
