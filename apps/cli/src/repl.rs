use std::io::{self, BufRead};

use quanttide_agent::llm::LLM;
use quanttide_agent::message::Message;

use crate::discover::KeywordIndex;
use crate::graph::RelationGraph;
use crate::intent::{classify, Intent};
use crate::report::ReportGenerator;
use crate::query::QueryEngine;

pub struct Repl {
    engine: QueryEngine,
    gallery_path: String,
}

impl Repl {
    pub fn new(engine: QueryEngine, gallery_path: String) -> Self {
        Self { engine, gallery_path }
    }

    pub fn run(&self) -> Result<(), String> {
        let reporter = ReportGenerator::new(QueryEngine::new(&self.gallery_path));

        println!("=== Project 11: Situation Engine ===");
        println!("Gallery: {}", self.gallery_path);
        println!("Commands:");
        println!("  weeks                    - list available weeks");
        println!("  show <week>              - show week summary");
        println!("  landscape <week>         - show week landscape (compact)");
        println!("  explore <name>           - track situation evolution across weeks");
        println!("  registry                 - show situation registry");
        println!("  report <week>            - generate structured weekly report");
        println!("  diff <weekA> <weekB>      - compare two weeks");
        println!("  relate <week>            - LLM infer situation relations");
        println!("  intentions [week] [name]  - list intentions");
        println!("  intention <id>           - show intention detail");
        println!("  filter <options>         - filter intentions");
        println!("  trace <title>            - find intention across weeks");
        println!("  drift <weekA> <weekB> <name> - compare priority/risk shift");
        println!("  evolve <name>            - intention evolution table");
        println!("  exit                     - quit");
        println!("\n输入任何字符串自动关联情境；输入 exit 退出。\n");

        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts[0] {
                "exit" | "quit" => break,
                "help" => {
                    println!("输入任何文字自动理解意图并执行。");
                    println!("你也可以精确指定命令：");
                    println!("  情境查询: 输入关键词（如「组织管理」「基础设施」）");
                    println!("  演化分析: 输入含「演化」「趋势」「变化」的描述");
                    println!("  对比分析: 输入含「对比」「diff」的描述");
                    println!("  关系分析: 输入含「关系」「关联」「冲突」的描述");
                    println!("  精确命令: weeks / show / report / diff / intentions / drift");
                    println!("  exit                     - 退出");
                }
                "weeks" => match self.engine.list_weeks() {
                    Ok(weeks) => {
                        println!("Available weeks:");
                        for w in weeks {
                            println!("  {}", w);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                },
                "registry" => match self.engine.registry() {
                    Ok(reg) => {
                        println!("Situation Registry:");
                        for r in reg {
                            println!("  {}: {}", r.name, r.label);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                },
                "show" => {
                    if parts.len() < 2 {
                        println!("Usage: show <week>");
                        continue;
                    }
                    match reporter.summary(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "landscape" => {
                    if parts.len() < 2 {
                        println!("Usage: landscape <week>");
                        continue;
                    }
                    match reporter.landscape(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "explore" => {
                    if parts.len() < 2 {
                        println!("Usage: explore <name>");
                        continue;
                    }
                    match reporter.evolution(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "report" => {
                    if parts.len() < 2 {
                        println!("Usage: report <week>");
                        continue;
                    }
                    match reporter.report(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "diff" => {
                    if parts.len() < 3 {
                        println!("Usage: diff <weekA> <weekB>");
                        continue;
                    }
                    match reporter.diff(parts[1], parts[2]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "relate" => {
                    if parts.len() < 2 {
                        println!("Usage: relate <week>");
                        continue;
                    }
                    match reporter.relate_llm(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "intentions" => {
                    let first = parts.get(1).map(|s| *s);
                    // if 1st arg is a week pattern, use as week; otherwise as situation name
                    let (w, n) = if let Some(v) = first {
                        if v.starts_with("2026-") {
                            (Some(v), parts.get(2).map(|s| *s))
                        } else {
                            (None, Some(v))
                        }
                    } else {
                        (None, None)
                    };
                    if let Some(w) = w {
                        match reporter.list_intentions(w, n) {
                            Ok(s) => println!("{}", s),
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        // no args: list all intentions across all weeks
                        match reporter.filter_intentions(None, None, None, None, None, None) {
                            Ok(s) => println!("{}", s),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                }
                "intention" => {
                    if parts.len() < 2 {
                        println!("Usage: intention <id>");
                        continue;
                    }
                    match reporter.show_intention(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "trace" => {
                    if parts.len() < 2 {
                        println!("Usage: trace <title>");
                        continue;
                    }
                    let title = parts[1..].join(" ");
                    match reporter.trace(&title) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "drift" => {
                    if parts.len() < 3 {
                        println!("Usage: drift <weekA> <weekB> <sit_name>");
                        continue;
                    }
                    let sit_name = parts.get(3).map(|s| *s).unwrap_or("");
                    match reporter.drift(parts[1], parts[2], sit_name) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "evolve" => {
                    if parts.len() < 2 {
                        println!("Usage: evolve <sit_name>");
                        continue;
                    }
                    match reporter.evolution_table(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "relate-intentions" | "ri" => {
                    if parts.len() < 2 {
                        println!("Usage: relate-intentions <week>");
                        continue;
                    }
                    match reporter.relate_intentions_llm(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "coverage" => {
                    if parts.len() < 2 {
                        println!("Usage: coverage <week>");
                        continue;
                    }
                    match reporter.coverage(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "schemas" => {
                    if parts.len() < 2 {
                        println!("Usage: schemas <week>");
                        continue;
                    }
                    match reporter.list_schemas(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "tension" => {
                    if parts.len() < 2 {
                        println!("Usage: tension <week>");
                        continue;
                    }
                    match reporter.tension(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "filter" => {
                    let mut week = None;
                    let mut sit_name = None;
                    let mut priority = None;
                    let mut risk = None;
                    let mut level = None;
                    let mut agent = None;
                    let mut i = 1;
                    while i < parts.len() {
                        match parts[i] {
                            "--week" => { i += 1; week = parts.get(i).map(|s| s.to_string()); }
                            "--sit" => { i += 1; sit_name = parts.get(i).map(|s| s.to_string()); }
                            "--priority" => { i += 1; priority = parts.get(i).map(|s| s.to_string()); }
                            "--risk" => { i += 1; risk = parts.get(i).map(|s| s.to_string()); }
                            "--level" => { i += 1; level = parts.get(i).map(|s| s.to_string()); }
                            "--agent" => { i += 1; agent = parts.get(i).map(|s| s.to_string()); }
                            _ => {}
                        }
                        i += 1;
                    }
                    match reporter.filter_intentions(
                        week.as_deref(),
                        sit_name.as_deref(),
                        priority.as_deref(),
                        risk.as_deref(),
                        level.as_deref(),
                        agent.as_deref(),
                    ) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                _ => {
                    let intent = classify(line);
                    match intent {
                        Intent::Explore(q) => {
                            let weeks = match self.engine.list_weeks() {
                                Ok(w) => w,
                                Err(_) => { println!("No gallery data loaded."); continue; }
                            };
                            let latest = weeks.last().cloned().unwrap_or_default();
                            let sits = self.engine.loader.load_situations(&latest).unwrap_or_default();
                            let schemas = self.engine.loader.load_schemas(&latest).unwrap_or_default();
                            let relations = self.engine.loader.load_situation_relations(&latest).unwrap_or_default();
                            let graph = RelationGraph::new(&relations);
                            let idx = KeywordIndex::new(&sits, &schemas);
                            let results = idx.search(&q, 5);
                            if results.is_empty() {
                                println!("未找到相关情境。尝试更精确的关键词。");
                            } else {
                                println!("相关情境（最新周 {}）：", latest);
                                    for (name, score) in &results {
                                        let label = sits.iter().find(|s| s.name == *name)
                                            .map(|s| s.label.as_str()).unwrap_or(name);
                                        println!("  {}（{}） 匹配度：{}", label, name, score);
                                        let neighbors = graph.neighbors(name);
                                        if !neighbors.is_empty() {
                                            for (n, rel) in &neighbors {
                                                let nlabel = sits.iter().find(|s| s.name == *n)
                                                    .map(|s| s.label.as_str()).unwrap_or(n);
                                                println!("    └ {} → {}（{:?}）", nlabel, label, rel);
                                            }
                                        }
                                    }

                                    // LLM analysis: if query suggests deeper analysis
                                    let analysis_keywords = ["分析", "总结", "怎么回事", "什么情况", "如何"];
                                    if analysis_keywords.iter().any(|k| q.contains(k)) {
                                        let context: Vec<String> = results.iter().map(|(name, _)| {
                                            let label = sits.iter().find(|s| s.name == *name)
                                                .map(|s| format!("{}（{}）", s.label, s.name))
                                                .unwrap_or_else(|| name.clone());
                                            let neighbors: Vec<String> = graph.neighbors(name).iter().map(|(n, rel)| {
                                                let nlabel = sits.iter().find(|s| s.name == *n)
                                                    .map(|s| s.label.as_str()).unwrap_or(n);
                                                format!("  - {}（{:?}）", nlabel, rel)
                                            }).collect();
                                            format!("{}\n关联情境：{}", label, neighbors.join("\n"))
                                        }).collect();
                                        let prompt = format!(
                                            "基于以下数据回答用户问题「{}」。\n\n匹配情境：\n{}\n\n请用中文简要回答，不超过200字。",
                                            q, context.join("\n\n")
                                        );
                                        if let Ok(llm) = std::env::var("LLM_API_KEY").or_else(|_| std::env::var("DEEPSEEK_API_KEY")) {
                                            if !llm.is_empty() {
                                                let client = LLM::default();
                                                if let Ok(resp) = client.complete(
                                                    &[Message::new("user", &prompt)],
                                                    Default::default(),
                                                ) {
                                                    println!("\nLLM 分析：\n{}", resp.content);
                                                }
                                            }
                                        }
                                    }
                            }
                        }
                        Intent::Evolution(q) => {
                            let sits = match self.engine.list_weeks().ok()
                                .and_then(|w| w.last().cloned())
                                .and_then(|week| self.engine.loader.load_situations(&week).ok())
                            {
                                Some(s) => s,
                                None => { println!("无法加载情境数据。"); continue; }
                            };
                            // Try to find a situation name in the query
                            let matched = sits.iter().find(|s| q.contains(&s.name) || q.contains(&s.label));
                            match matched {
                                Some(sit) => match reporter.evolution(&sit.name) {
                                    Ok(r) => println!("{}", r),
                                    Err(e) => println!("Error: {}", e),
                                },
                                None => println!("请指定具体情境名，如「认知工程的演化」"),
                            }
                        }
                        Intent::Compare(_q) => {
                            let weeks = match self.engine.list_weeks() {
                                Ok(w) => w,
                                Err(_) => { println!("No gallery data."); continue; }
                            };
                            if weeks.len() >= 2 {
                                let last = weeks.last().unwrap();
                                let prev = weeks.get(weeks.len().saturating_sub(2)).unwrap();
                                match reporter.diff(prev, last) {
                                    Ok(r) => println!("{}", r),
                                    Err(e) => println!("Error: {}", e),
                                }
                            }
                        }
                        Intent::Relate(_q) => {
                            let weeks = match self.engine.list_weeks() {
                                Ok(w) => w,
                                Err(_) => { println!("No gallery data."); continue; }
                            };
                            let latest = weeks.last().cloned().unwrap_or_default();
                            match reporter.tension(&latest) {
                                Ok(r) => println!("{}", r),
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
