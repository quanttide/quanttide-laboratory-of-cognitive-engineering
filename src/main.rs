use std::fs;

use qtcloud_think_cli::analyze;
use qtcloud_think_cli::repo::Repo;

const WORLD: &str = "quanttide-founder";
const PERIOD: &str = "2026-W23";
const JOURNAL_PATH: &str = "../../data/journal";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let group = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    match group {
        "a" => println!("{}", experiment_a()),
        "b" => println!("{}", experiment_b()),
        "c" => match experiment_c() {
            Ok(output) => println!("{}", output),
            Err(e) => eprintln!("实验 C 失败: {}", e),
        },
        _ => {
            let a = experiment_a();
            let b = experiment_b();
            let c = experiment_c().unwrap_or_else(|e| format!("## 失败: {}", e));
            let report = format!(
                "# 实验对比总结\n\n\
                 ## A — 纯数据\n\n{}\n\n\
                 ## B — 规则引擎\n\n{}\n\n\
                 ## C — LLM 综合\n\n{}\n",
                a, b, c
            );
            fs::write("data/report.md", &report).expect("写 report.md 失败");
            println!("{}", report);
            println!("\n报告已写入 data/report.md");
        }
    }
}

fn experiment_a() -> String {
    let dir = format!("{}/{}/{}", JOURNAL_PATH, WORLD, PERIOD);
    let mut out = String::new();
    out.push_str(&format!("# 实验 A：纯数据访问\n\n路径: `{}`\n\n", dir));

    let entries = fs::read_dir(&dir).expect("无法读取日记目录");
    let mut domains: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().map_or(false, |x| x == "yaml") {
                p.file_stem().and_then(|s| s.to_str()).map(String::from)
            } else {
                None
            }
        })
        .filter(|n| n != "thoughts")
        .collect();
    domains.sort();

    for name in &domains {
        let path = format!("{}/{}.yaml", dir, name);
        let content = fs::read_to_string(&path).expect("读文件失败");
        let raw: serde_yaml::Value = serde_yaml::from_str(&content).expect("解析 YAML 失败");

        out.push_str(&format!("## {}\n\n", name));

        if let Some(schemas) = raw.get("schemas").and_then(|v| v.as_sequence()) {
            out.push_str(&format!("### Schema（{}个）\n\n", schemas.len()));
            for s in schemas {
                if let Some(usage) = s.get("usage").and_then(|v| v.as_str()) {
                    out.push_str(&format!("- 用途: {}\n", usage));
                }
                if let Some(entities) = s.get("entities").and_then(|v| v.as_sequence()) {
                    for e in entities {
                        if let Some(en) = e.get("name").and_then(|v| v.as_str()) {
                            out.push_str(&format!("  - 实体: {}\n", en));
                        }
                    }
                }
            }
        }

        if let Some(situations) = raw.get("situations").and_then(|v| v.as_sequence()) {
            out.push_str(&format!("\n### Situation（{}个）\n\n", situations.len()));
            for s in situations {
                if let Some(c) = s.get("content") {
                    if let Some(a) = c.get("agenda").and_then(|v| v.as_str()) {
                        out.push_str(&format!("- Agenda: {}\n", a));
                    }
                    if let Some(f) = c.get("frame").and_then(|v| v.as_str()) {
                        out.push_str(&format!("- Frame: {}\n", f));
                    }
                }
                if let Some(rels) = s.get("relations").and_then(|v| v.as_sequence()) {
                    for r in rels {
                        let src = r.get("source").and_then(|v| v.as_str()).unwrap_or("");
                        let tgt = r.get("target").and_then(|v| v.as_str()).unwrap_or("");
                        let rt = r.get("relation_type").and_then(|v| v.as_str()).unwrap_or("");
                        let desc = r.get("description").and_then(|v| v.as_str()).unwrap_or("");
                        out.push_str(&format!("  - {} ──[{}]──▶ {} : {}\n", src, rt, tgt, desc));
                    }
                }
            }
        }

        if let Some(intents) = raw.get("intentions").and_then(|v| v.as_sequence()) {
            out.push_str(&format!("\n### Intention（{}个）\n\n", intents.len()));
            for i in intents {
                let title = i.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let pri = i
                    .get("priority")
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let lvl = i
                    .get("level")
                    .and_then(|l| l.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let risk = i
                    .get("risk")
                    .and_then(|r| r.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                out.push_str(&format!(
                    "- [{}][{}][{}] {}\n",
                    pri, lvl, risk, title
                ));
            }
        }
        out.push('\n');
    }

    out.push_str("### 评估\n\n");
    out.push_str("**能回答**: 每个 domain 的 schemas/situations/intentions 及其属性。\n\n");
    out.push_str("**不能回答**: domain 间关系、跨周演化、数据一致性、方案生成。\n");
    out
}

fn experiment_b() -> String {
    let repo = Repo::open(JOURNAL_PATH);
    let mut out = String::new();
    out.push_str(&format!(
        "# 实验 B：规则引擎（CLI 接口）\n\nWorld: {}, Period: {}\n\n",
        WORLD, PERIOD
    ));

    out.push_str("## 数据一致性报告\n\n");
    if let Ok(coherence) = repo.describe(WORLD, PERIOD) {
        out.push_str("| Domain | Intentions | Schemas | Relations |\n");
        out.push_str("|--------|-----------|---------|----------|\n");
        for c in &coherence {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                c.domain, c.intentions, c.schemas, c.relations
            ));
        }
    }

    out.push_str("\n## 领域间关系网络\n\n");
    if let Ok(domains) = repo.domains(WORLD, PERIOD) {
        for d in &domains {
            if let Ok(file) = repo.load(WORLD, PERIOD, &d.name) {
                let rels = file.relations();
                if !rels.is_empty() {
                    out.push_str(&format!("### {} 的关系\n\n", d.name));
                    for r in &rels {
                        out.push_str(&format!(
                            "- {} ──[{:?}]──▶ {} (conf: {:?}) — {}\n",
                            r.source, r.relation_type, r.target, r.confidence, r.description
                        ));
                    }
                    out.push('\n');
                }
            }
        }
    }

    out.push_str("## 演化追踪\n\n");
    if let Ok(domains) = repo.domains(WORLD, PERIOD) {
        for d in &domains {
            if let Ok(snapshots) = analyze::track_evolution(&repo, WORLD, &d.name) {
                if !snapshots.is_empty() {
                    out.push_str(&format!("### {} 的演化\n\n", d.name));
                    for s in &snapshots {
                        out.push_str(&format!(
                            "- {}: {} 意图, entities: {:?}\n",
                            s.period,
                            s.intentions.len(),
                            s.entities
                        ));
                        for i in &s.intentions {
                            out.push_str(&format!(
                                "  - [{}][{}][{}] {}\n",
                                i.priority, i.level, i.risk, i.title
                            ));
                        }
                    }
                    out.push('\n');
                }
            }
        }
    }

    out.push_str("### 评估\n\n");
    out.push_str("**规则引擎增加的价值**:\n");
    out.push_str("- 自动汇总数据一致性（describe()）\n");
    out.push_str("- 关系网络自动提取（relations()）——无需人工跨文件寻找\n");
    out.push_str("- 跨周演化追踪（track_evolution()）——自动时间序列对齐\n");
    out.push_str("- 一致性检测——一眼看出哪个 domain 缺少 schemas 或 relations\n");
    out.push_str("- 冲突检测——relation_type=conflict 自动标识领域间的对立关系\n");
    out.push_str("- 优先级漂移——跨期对比发现意图优先级变化\n");
    out
}

fn experiment_c() -> Result<String, String> {
    use quanttide_agent::{Message, Settings, LLM};

    let b_output = experiment_b();

    let settings = Settings::from_env();
    if settings.llm_api_key.is_empty() {
        return Err("LLM_API_KEY 未设置。设置环境变量 LLM_API_KEY 后重试。".to_string());
    }

    let llm = LLM::new(&settings.llm_model, &settings.llm_base_url, &settings.llm_api_key);

    let prompt = format!(
        "你是一个技术咨询专家。以下是一组认知工程数据，描述了一个创始人在一周内的工作语境。\n\n\
        背景：该创始人运营的 SaaS 平台当前是单租户架构，需要升级为多租户架构，\
        同时需要预留支持 SaaS 模式和聚合平台模式两种多租户方式。\n\n\
        任务：请基于以下数据，生成一份完整的技术升级方案与报价。\n\
        方案应包括：\n\
        1. 现状分析（从数据中提取的关键洞察）\n\
        2. 技术升级方案（多租户架构设计）\n\
        3. 实施路线图\n\
        4. 报价（按阶段或工作量）\n\n\
        以下是为期一周的认知工程数据：\n\n{}",
        b_output
    );

    let messages = vec![Message::new("user", &prompt)];
    let response = llm
        .complete(&messages, Default::default())
        .map_err(|e| format!("LLM 调用失败: {}", e))?;

    let mut out = String::new();
    out.push_str("# 实验 C：CLI + LLM 综合\n\n");
    out.push_str("> 注意：本实验需要有效 LLM_API_KEY。\n\n");
    out.push_str("## Prompt\n\n```\n");
    out.push_str(&prompt);
    out.push_str("\n```\n\n");
    out.push_str("## LLM 响应\n\n");
    out.push_str(&response.content);
    out.push_str("\n\n### 使用统计\n\n");
    if let Some(usage) = response.usage {
        out.push_str(&format!(
            "- 输入 tokens: {}\n- 输出 tokens: {}\n- 总计: {}\n",
            usage.input_tokens, usage.output_tokens, usage.total_tokens
        ));
    }
    out.push_str("\n### 评估\n\n");
    out.push_str("**LLM 增量价值**:\n");
    out.push_str("- 将结构化数据转化为可交付的技术方案\n");
    out.push_str("- 生成报价——B 无法做到的推理任务\n");
    out.push_str("- 风险：可能信息失真或产生幻觉\n");
    out.push_str("- 依赖：需要 LLM API Key 和网络连接\n");

    Ok(out)
}
