use std::fs;
use std::path::PathBuf;

use crate::models::Situation;
use crate::query::QueryEngine;

pub struct ReportGenerator {
    pub engine: QueryEngine,
}

impl ReportGenerator {
    pub fn new(engine: QueryEngine) -> Self {
        Self { engine }
    }

    /// Generate a text summary of a week
    pub fn summary(&self, week: &str) -> Result<String, String> {
        let data = self.engine.week(week)?;
        let registry = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = registry
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let mut out = String::new();
        out.push_str(&format!("# Week {} Summary\n\n", week));
        out.push_str(&format!(
            "Situations: {} | Intentions: {}\n\n",
            data.situations.len(),
            data.intentions.len(),
        ));

        for sit in &data.situations {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            out.push_str(&format!("## {} ({})\n\n", label, sit.name));
            out.push_str(&format!("**Agenda**: {}\n", sit.content.agenda));
            out.push_str(&format!("**Ecology**: {}\n", sit.content.ecology));
            out.push_str(&format!("**Frame**: {}\n", sit.content.frame));
            out.push_str(&format!("**Dynamics**: {}\n\n", sit.content.dynamics));

            if let Some(intents) = data.intention_map.get(&sit.name) {
                out.push_str("### Intentions\n\n");
                for i in intents {
                    out.push_str(&format!(
                        "- **{}** [{}] trigger={}, risk={}\n",
                        i.title, i.priority.label, i.trigger.label, i.risk.label
                    ));
                    out.push_str(&format!("  {}\n", i.description));
                }
            }
            out.push('\n');
        }

        Ok(out)
    }

    /// Show a situation's evolution across weeks
    pub fn evolution(&self, name: &str) -> Result<String, String> {
        let results = self.engine.situation(name)?;
        let registry = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = registry
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let label = label_map.get(name).cloned().unwrap_or_else(|| name.to_string());
        let mut out = String::new();
        out.push_str(&format!("# {} ({}) Evolution\n\n", label, name));

        for (week, sit) in &results {
            out.push_str(&format!("## {}\n\n", week));
            out.push_str(&format!("**Dynamics**: {}\n\n", sit.content.dynamics));
            out.push_str(&format!("**Agenda**: {}\n", sit.content.agenda));
            if let Ok(intents) = self.engine.intentions(week, name) {
                if !intents.is_empty() {
                    out.push_str("\n**Intentions**:\n");
                    for i in &intents {
                        out.push_str(&format!("- {} [{}]\n", i.title, i.priority.label));
                    }
                }
            }
            out.push('\n');
        }
        Ok(out)
    }

    /// Show the landscape of all situations for a week (compact table)
    pub fn landscape(&self, week: &str) -> Result<String, String> {
        let data = self.engine.week(week)?;
        let registry = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = registry
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let mut out = String::new();
        out.push_str(&format!("# Landscape: {}\n\n", week));
        out.push_str(&format!(
            "| Situation | Intentions | Priorities |\n|---|---|---|\n"
        ));

        for sit in &data.situations {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            let intents = data.intention_map.get(&sit.name);
            let count = intents.map(|v| v.len()).unwrap_or(0);
            let prios: String = intents
                .map(|v| {
                    v.iter()
                        .map(|i| i.priority.label.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            out.push_str(&format!("| {} | {} | {} |\n", label, count, prios));
        }

        out.push('\n');
        Ok(out)
    }

    fn reports_dir(&self, week: &str) -> PathBuf {
        let dir = PathBuf::from("reports").join(week);
        fs::create_dir_all(&dir).ok();
        dir
    }

    /// Generate a structured weekly report (six-feature template)
    pub fn report(&self, week: &str) -> Result<String, String> {
        let data = self.engine.week(week)?;
        let registry = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = registry
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let mut out = String::new();
        out.push_str(&format!("# 情境周报：{}\n\n", week));

        // — Core Judgment —
        out.push_str("## 核心判断\n\n");
        out.push_str("（待 LLM 填充）\n\n");

        // — Actions —
        out.push_str("## 行动建议\n\n");
        out.push_str("| 优先级 | 行动项 | 负责人 | 时限 | 预期效果 | 风险 |\n");
        out.push_str("|--------|--------|-------|------|---------|------|\n");
        out.push_str("| （待生成） | | | | | |\n\n");

        // — Panorama —
        out.push_str("## 全景概览\n\n");
        out.push_str("| 情境 | 意向数 | 高优先级 | 高风险 |\n");
        out.push_str("|------|--------|---------|-------|\n");
        for sit in &data.situations {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            let intents = data.intention_map.get(&sit.name);
            let count = intents.map(|v| v.len()).unwrap_or(0);
            let high_p = intents.map(|v| v.iter().filter(|i| i.priority.name == "high").count()).unwrap_or(0);
            let high_r = intents.map(|v| v.iter().filter(|i| i.risk.name == "high").count()).unwrap_or(0);
            out.push_str(&format!("| {} | {} | {} | {} |\n", label, count, high_p, high_r));
        }
        out.push('\n');

        // — Per-situation analysis —
        out.push_str("## 逐情境分析\n\n");
        for sit in &data.situations {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            out.push_str(&format!("### {}（{}）\n\n", label, sit.name));
            out.push_str(&format!("**演化**：{}\n\n", sit.content.dynamics));
            out.push_str(&format!("**现象**：{}\n\n", sit.content.ecology));
            out.push_str(&format!("**判断**：{}\n\n", sit.content.frame));

            if let Some(intents) = data.intention_map.get(&sit.name) {
                out.push_str("| 关键意向 | 优先级 | 风险 |\n");
                out.push_str("|---------|--------|------|\n");
                for i in intents {
                    out.push_str(&format!("| {} | {} | {} |\n", i.title, i.priority.label, i.risk.label));
                }
            }
            out.push_str("\n---\n\n");
        }

        // — Try loading cached relations —
        let rel_path = self.reports_dir(week).join("relations.json");
        if let Ok(content) = fs::read_to_string(&rel_path) {
            out.push_str("## 关键关系\n\n");
            if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                for rel in &arr {
                    let s = rel["source"].as_str().unwrap_or("?");
                    let t = rel["target"].as_str().unwrap_or("?");
                    let r = rel["type"].as_str().unwrap_or("?");
                    let st = rel["strength"].as_str().unwrap_or("?");
                    let l = rel["logic"].as_str().unwrap_or("");
                    out.push_str(&format!("- **{}** ↔ **{}**：{}（{}）\n  {}\n", s, t, r, st, l));
                }
            }
            out.push('\n');
        } else {
            out.push_str("## 关键关系\n\n");
            out.push_str("（运行 `relate` 生成）\n\n");
        }

        // — Mental Models placeholder —
        out.push_str("## 跨情境心智模型\n\n");
        out.push_str("（待推理）\n\n");

        // — Comparison —
        out.push_str("## 与前周对比\n\n");
        out.push_str("（待实现跨周差异分析）\n");

        // Save report
        let report_path = self.reports_dir(week).join("report.md");
        fs::write(&report_path, &out).ok();
        println!("Report saved to: {:?}", report_path);

        Ok(out)
    }

    /// Diff two weeks: show which situations appear/disappear/change
    pub fn diff(&self, week_a: &str, week_b: &str) -> Result<String, String> {
        let data_a = self.engine.week(week_a)?;
        let data_b = self.engine.week(week_b)?;
        let registry = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = registry
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let mut out = String::new();
        out.push_str(&format!("# Diff: {} → {}\n\n", week_a, week_b));

        let mut map_a: std::collections::HashMap<&str, &Situation> = std::collections::HashMap::new();
        for s in &data_a.situations {
            map_a.insert(s.name.as_str(), s);
        }

        let mut appeared = Vec::new();
        let mut disappeared = Vec::new();
        let mut changed = Vec::new();

        for s in &data_b.situations {
            if !map_a.contains_key(s.name.as_str()) {
                appeared.push(s);
            } else if let Some(old) = map_a.get(s.name.as_str()) {
                if old.content.dynamics != s.content.dynamics {
                    changed.push((old, s));
                }
            }
        }
        for s in &data_a.situations {
            if data_b.situations.iter().all(|x| x.name != s.name) {
                disappeared.push(s);
            }
        }

        if !disappeared.is_empty() {
            out.push_str("## 消失的情境\n\n");
            for s in &disappeared {
                let label = label_map.get(&s.name).cloned().unwrap_or_else(|| s.name.clone());
                out.push_str(&format!("- {}（{}）\n", label, s.name));
            }
            out.push('\n');
        }

        if !appeared.is_empty() {
            out.push_str("## 新增的情境\n\n");
            for s in &appeared {
                let label = label_map.get(&s.name).cloned().unwrap_or_else(|| s.name.clone());
                out.push_str(&format!("- {}（{}）\n", label, s.name));
            }
            out.push('\n');
        }

        if !changed.is_empty() {
            out.push_str("## 演化变化\n\n");
            for (old, new) in &changed {
                let label = label_map.get(&new.name).cloned().unwrap_or_else(|| new.name.clone());
                out.push_str(&format!("### {}（{}）\n\n", label, new.name));
                out.push_str(&format!("| | {} | {} |\n", week_a, week_b));
                out.push_str("|---|------|------|\n");
                out.push_str(&format!("| 演化 | {} | {} |\n", old.content.dynamics, new.content.dynamics));
                out.push_str(&format!("| 意图数 | {} | {} |\n",
                    self.engine.intentions(week_a, &new.name).map(|v| v.len()).unwrap_or(0),
                    self.engine.intentions(week_b, &new.name).map(|v| v.len()).unwrap_or(0)
                ));
                out.push('\n');
            }
        }

        if disappeared.is_empty() && appeared.is_empty() && changed.is_empty() {
            out.push_str("两周之间无显著变化。\n");
        }

        Ok(out)
    }

    /// Infer relations between situations using LLM, with caching
    pub fn relate_llm(&self, week: &str) -> Result<String, String> {
        let cache_path = self.reports_dir(week).join("relations.json");

        // Check cache
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                eprintln!("(loaded cached relations from {:?})", cache_path);
                return Self::format_relations(week, &json);
            }
        }

        let client = crate::llm::DeepSeekClient::from_env()?;
        let data = self.engine.week(week)?;
        let registry = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = registry
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let mut sit_descs = String::new();
        for sit in &data.situations {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            sit_descs.push_str(&format!("情境「{}」({}):\n", label, sit.name));
            sit_descs.push_str(&format!("- agenda: {}\n", sit.content.agenda));
            sit_descs.push_str(&format!("- dynamics: {}\n\n", sit.content.dynamics));
        }

        let total = data.situations.len();
        let prompt = format!(
            r#"你是一个情境关系分析引擎。输入一个周期的 {} 个情境，请分析两两之间的关系。

关系类型：支持、冲突、触发、演化、情感补给、组件、同框

每个关系输出格式：
{{
  "source": "情境A的name",
  "target": "情境B的name",
  "type": "关系类型",
  "strength": "强/中/弱",
  "logic": "为什么存在这个关系"
}}

请以JSON数组返回，数组每个元素是一个关系对象。
只输出JSON，不要额外文字。

## 情境列表

{}

"#,
            total, sit_descs
        );

        let raw = client.chat(&prompt)?;
        let json = crate::llm::extract_json(&raw)?;

        // Save cache
        if let Ok(content) = serde_json::to_string_pretty(&json) {
            fs::write(&cache_path, &content).ok();
            eprintln!("(cached relations to {:?})", cache_path);
        }

        Self::format_relations(week, &json)
    }

    fn format_relations(week: &str, json: &serde_json::Value) -> Result<String, String> {
        let mut out = String::new();
        out.push_str(&format!("# Relations: {}\n\n", week));

        if let Some(arr) = json.as_array() {
            for rel in arr {
                let source = rel["source"].as_str().unwrap_or("?");
                let target = rel["target"].as_str().unwrap_or("?");
                let rtype = rel["type"].as_str().unwrap_or("?");
                let strength = rel["strength"].as_str().unwrap_or("?");
                let logic = rel["logic"].as_str().unwrap_or("");
                out.push_str(&format!("- **{}** ↔ **{}**：{}（{}）\n", source, target, rtype, strength));
                out.push_str(&format!("  {}\n", logic));
            }
        }

        if json.as_array().map_or(true, |a| a.is_empty()) {
            out.push_str("（未发现关系）\n");
        }

        Ok(out)
    }
}
