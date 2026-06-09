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
        let total = data.intentions.len();
        let high_p = data.intentions.iter().filter(|i| i.priority.name == "high").count();
        let high_r = data.intentions.iter().filter(|i| i.risk.name == "high").count();
        let top = data.intentions.iter().filter(|i| i.level.name == "top").count();
        let bottom = data.intentions.iter().filter(|i| i.level.name == "bottom").count();
        out.push_str(&format!(
            "{} 个情境，{} 条意向（高优先 {}，高风 {}，顶层 {}，底层 {}）\n\n",
            data.situations.len(), total, high_p, high_r, top, bottom
        ));

        out.push_str("| 情境 | 意向数 | 高优先级 | 高风险 | 顶层 | 底层 |\n");
        out.push_str("|------|--------|---------|-------|------|------|\n");
        for sit in &data.situations {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            let intents = data.intention_map.get(&sit.name);
            let c = intents.map(|v| v.len()).unwrap_or(0);
            let hp = intents.map(|v| v.iter().filter(|i| i.priority.name == "high").count()).unwrap_or(0);
            let hr = intents.map(|v| v.iter().filter(|i| i.risk.name == "high").count()).unwrap_or(0);
            let t = intents.map(|v| v.iter().filter(|i| i.level.name == "top").count()).unwrap_or(0);
            let b = intents.map(|v| v.iter().filter(|i| i.level.name == "bottom").count()).unwrap_or(0);
            out.push_str(&format!("| {} | {} | {} | {} | {} | {} |\n", label, c, hp, hr, t, b));
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
                out.push_str("| 关键意向 | 优先级 | 风险 | 层级 | 触发 |\n");
                out.push_str("|---------|--------|------|------|------|\n");
                for i in intents {
                    out.push_str(&format!(
                        "| {} | {} | {} | {} | {} |\n",
                        i.title, i.priority.label, i.risk.label, i.level.label, i.trigger.label
                    ));
                }
            }
            out.push_str("\n---\n\n");
        }

        // — Relations: merge situation + intention —
        let mut has_relations = false;
        let rel_path = self.reports_dir(week).join("relations.json");
        let irel_path = self.reports_dir(week).join("intention-relations.json");

        // Build index map for intention relations: idx → title
        let mut idx_map: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
        if irel_path.exists() {
            let mut i = 0usize;
            for (_, intents) in &data.intention_map {
                for intent in intents {
                    idx_map.insert(i, intent.title.clone());
                    i += 1;
                }
            }
        }

        let rel_ok = rel_path.exists();
        let irel_ok = irel_path.exists();

        if rel_ok || irel_ok {
            has_relations = true;
            out.push_str("## 关系分析\n\n");

            if rel_ok {
                if let Ok(content) = fs::read_to_string(&rel_path) {
                    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                        out.push_str("### 情境层\n\n");
                        for rel in &arr {
                            let s = rel["source"].as_str().unwrap_or("?");
                            let t = rel["target"].as_str().unwrap_or("?");
                            let r = rel["type"].as_str().unwrap_or("?");
                            let st = rel["strength"].as_str().unwrap_or("?");
                            let l = rel["logic"].as_str().unwrap_or("");
                            out.push_str(&format!("- **{}** ↔ **{}**：{}（{}）\n  {}\n", s, t, r, st, l));
                        }
                        out.push('\n');
                    }
                }
            }

            if irel_ok {
                if let Ok(content) = fs::read_to_string(&irel_path) {
                    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                        out.push_str("### 意图层\n\n");
                        for rel in &arr {
                            let si = rel["source"].as_u64().unwrap_or(999) as usize;
                            let ti = rel["target"].as_u64().unwrap_or(999) as usize;
                            let stitle = idx_map.get(&si).map(|s| s.as_str()).unwrap_or("?");
                            let ttitle = idx_map.get(&ti).map(|s| s.as_str()).unwrap_or("?");
                            let rtype = rel["type"].as_str().unwrap_or("?");
                            let logic = rel["logic"].as_str().unwrap_or("");
                            out.push_str(&format!("- **{}** → **{}**：{} — {}\n", stitle, ttitle, rtype, logic));
                        }
                        out.push('\n');
                    }
                }
            }
        }

        if !has_relations {
            out.push_str("## 关系分析\n\n（运行 `relate` 和 `ri` 生成）\n\n");
        }

        // — Drift from previous week —
        let prev_week = self.previous_week(week);
        if let Some(ref pw) = prev_week {
            out.push_str("## 与前周对比\n\n");
            if let Ok(prev_data) = self.engine.week(pw) {
                for sit in &data.situations {
                    let prev = prev_data.intention_map.get(&sit.name).cloned().unwrap_or_default();
                    let curr = data.intention_map.get(&sit.name).cloned().unwrap_or_default();
                    if prev.is_empty() && curr.is_empty() { continue; }
                    let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
                    let pc = prev.len();
                    let cc = curr.len();
                    if pc != cc {
                        let diff = if cc > pc { format!("+{}", cc - pc) } else { format!("-{}", pc - cc) };
                        out.push_str(&format!("- **{}**：{} → {}（{}）\n", label, pc, cc, diff));
                    }
                    // Check priority drift for matched titles
                    for a in &prev {
                        for b in &curr {
                            if a.title == b.title && a.priority.name != b.priority.name {
                                out.push_str(&format!("  - 「{}」优先级：{} → {}\n", a.title, a.priority.label, b.priority.label));
                            }
                        }
                    }
                }
                out.push('\n');
            }
        } else {
            out.push_str("## 与前周对比\n\n");
            out.push_str("（无前周数据）\n\n");
        }

        // — Mental Models (schemas from gallery) —
        out.push_str("## 跨情境心智模型\n\n");
        match self.engine.loader.load_schemas(week) {
            Ok(schemas) => {
                for s in &schemas {
                    out.push_str(&format!("### {}\n\n", s.label));
                    if !s.content.usage.is_empty() {
                        out.push_str(&format!("{}\n\n", s.content.usage));
                    }
                    for c in &s.content.causals {
                        out.push_str(&format!("- IF {} THEN {}\n", c.condition, c.outcome));
                    }
                    for b in &s.content.biases {
                        out.push_str(&format!("- 信念：{}（事实：{}）\n", b.belief, b.fact));
                    }
                    for b in &s.content.boundaries {
                        out.push_str(&format!("- 边界：{}\n", b));
                    }
                    out.push('\n');
                }
                if schemas.is_empty() {
                    out.push_str("（暂无心智模型数据）\n\n");
                }
            }
            Err(e) => {
                out.push_str(&format!("（加载失败：{}）\n\n", e));
            }
        }

        // Save
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

    // ── Phase 2: Cross-week intention tracking ──

    /// Trace an intention title across weeks by substring match
    pub fn trace(&self, title: &str) -> Result<String, String> {
        let results = self.engine.all_intentions(None, None, None, None, None, None)?;
        let reg = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = reg
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let matched: Vec<_> = results
            .into_iter()
            .filter(|(_, _, i)| i.title.contains(title))
            .collect();

        if matched.is_empty() {
            return Ok(format!("No intentions match '{}'", title));
        }

        let mut out = String::new();
        out.push_str(&format!("# Trace: '{}'\n\n", title));
        out.push_str("| Week | Situation | Title | Priority | Risk | Level |\n");
        out.push_str("|------|-----------|-------|----------|------|-------|\n");
        for (w, sn, i) in &matched {
            let label = label_map.get(sn.as_str()).cloned().unwrap_or_else(|| sn.clone());
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                w, label, i.title, i.priority.label, i.risk.label, i.level.label
            ));
        }
        out.push_str(&format!("\n{} matches across {} weeks\n", matched.len(), {
            let mut weeks: Vec<_> = matched.iter().map(|(w, _, _)| w.as_str()).collect();
            weeks.sort();
            weeks.dedup();
            weeks.len()
        }));
        Ok(out)
    }

    /// Drift: compare intention priority/risk shifts for a situation across two weeks
    pub fn drift(&self, week_a: &str, week_b: &str, sit_name: &str) -> Result<String, String> {
        let reg = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = reg
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();
        let label = label_map.get(sit_name).cloned().unwrap_or_else(|| sit_name.to_string());

        let data_a = self.engine.week(week_a).ok();
        let data_b = self.engine.week(week_b).ok();
        let intents_a = data_a.as_ref().and_then(|d| d.intention_map.get(sit_name)).cloned().unwrap_or_default();
        let intents_b = data_b.as_ref().and_then(|d| d.intention_map.get(sit_name)).cloned().unwrap_or_default();

        if intents_a.is_empty() && intents_b.is_empty() {
            return Ok(format!("No intentions for {} in {} or {}", label, week_a, week_b));
        }

        let mut out = String::new();
        out.push_str(&format!("# Drift: {} ({})\n\n", label, sit_name));
        out.push_str(&format!("Comparing {} → {}\n\n", week_a, week_b));

        // Match intentions by title
        let mut matched = Vec::new();
        for a in &intents_a {
            for b in &intents_b {
                if a.title == b.title {
                    let prio_drift = if a.priority.name != b.priority.name {
                        format!("{} → {}", a.priority.label, b.priority.label)
                    } else {
                        format!("{} (不变)", a.priority.label)
                    };
                    let risk_drift = if a.risk.name != b.risk.name {
                        format!("{} → {}", a.risk.label, b.risk.label)
                    } else {
                        format!("{} (不变)", a.risk.label)
                    };
                    matched.push((a.title.clone(), prio_drift, risk_drift));
                }
            }
        }

        // Unmatched in A
        let unmatched_a: Vec<_> = intents_a.iter().filter(|a| !intents_b.iter().any(|b| b.title == a.title)).collect();
        // Unmatched in B (new)
        let unmatched_b: Vec<_> = intents_b.iter().filter(|b| !intents_a.iter().any(|a| a.title == b.title)).collect();

        if !matched.is_empty() {
            out.push_str("### 匹配的意向\n\n");
            out.push_str("| 意向 | Priority 变化 | Risk 变化 |\n");
            out.push_str("|------|-------------|----------|\n");
            for (t, p, r) in &matched {
                out.push_str(&format!("| {} | {} | {} |\n", t, p, r));
            }
            out.push('\n');
        }

        if !unmatched_a.is_empty() {
            out.push_str(&format!("### 仅在 {} 存在\n\n", week_a));
            for i in &unmatched_a {
                out.push_str(&format!("- {} [{}]\n", i.title, i.priority.label));
            }
            out.push('\n');
        }

        if !unmatched_b.is_empty() {
            out.push_str(&format!("### 仅在 {} 存在（新增）\n\n", week_b));
            for i in &unmatched_b {
                out.push_str(&format!("- {} [{}]\n", i.title, i.priority.label));
            }
            out.push('\n');
        }

        Ok(out)
    }

    /// Intention evolution table: show all intentions for a situation across weeks
    pub fn evolution_table(&self, sit_name: &str) -> Result<String, String> {
        let results = self.engine.all_intentions(None, Some(sit_name), None, None, None, None)?;
        let reg = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = reg
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();
        let label = label_map.get(sit_name).cloned().unwrap_or_else(|| sit_name.to_string());

        if results.is_empty() {
            return Ok(format!("No intentions for {} ({})", label, sit_name));
        }

        // Group by title
        let mut by_title: std::collections::BTreeMap<String, Vec<(String, String, String)>> = std::collections::BTreeMap::new();
        // (week, priority, risk)
        for (w, _, i) in &results {
            by_title
                .entry(i.title.clone())
                .or_default()
                .push((w.clone(), i.priority.label.clone(), i.risk.label.clone()));
        }

        let mut out = String::new();
        out.push_str(&format!("# 意图演化：{}（{}）\n\n", label, sit_name));
        out.push_str("| 意向 |", );

        // collect all weeks
        let mut all_weeks: Vec<&str> = results.iter().map(|(w, _, _)| w.as_str()).collect();
        all_weeks.sort();
        all_weeks.dedup();
        for w in &all_weeks {
            out.push_str(&format!(" {} | {} |", w, w));
        }
        out.push_str("\n|------|");
        for _ in &all_weeks {
            out.push_str("--------|-------|");
        }
        out.push('\n');

        for (title, entries) in &by_title {
            out.push_str(&format!("| {} |", title));
            for w in &all_weeks {
                if let Some((_, p, r)) = entries.iter().find(|(ww, _, _)| ww == w) {
                    out.push_str(&format!(" {} | {} |", p, r));
                } else {
                    out.push_str(" - | - |");
                }
            }
            out.push('\n');
        }

        out.push('\n');
        out.push_str(&format!("{} unique intentions across {} weeks\n", by_title.len(), all_weeks.len()));
        Ok(out)
    }

    // ── Intention queries ──

    /// List intentions for a week, optionally filtered by situation name
    pub fn list_intentions(&self, week: &str, name: Option<&str>) -> Result<String, String> {
        let registry = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = registry
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let data = self.engine.week(week)?;
        let mut out = String::new();
        out.push_str(&format!("# Intentions: {}\n\n", week));

        for sit in &data.situations {
            if let Some(ref n) = name {
                if sit.name != *n {
                    continue;
                }
            }
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            if let Some(intents) = data.intention_map.get(&sit.name) {
                out.push_str(&format!("## {}（{}）\n\n", label, sit.name));
                for i in intents {
                    out.push_str(&format!(
                        "- {} | level={} | priority={} | risk={} | trigger={}\n",
                        i.title, i.level.label, i.priority.label, i.risk.label, i.trigger.label
                    ));
                    out.push_str(&format!("  {}\n", i.description));
                }
                out.push('\n');
            }
        }
        Ok(out)
    }

    /// Show a single intention by UUID
    pub fn show_intention(&self, id: &str) -> Result<String, String> {
        match self.engine.intention_by_id(id)? {
            None => Ok(format!("Intention not found: {}", id)),
            Some((week, sit_name, i)) => {
                let reg = self.engine.registry().ok();
                let label = reg
                    .unwrap_or_default()
                    .into_iter()
                    .find(|e| e.name == sit_name)
                    .map(|e| e.label)
                    .unwrap_or_else(|| sit_name.clone());
                let mut out = String::new();
                out.push_str(&format!("# {}\n\n", i.title));
                out.push_str(&format!("**ID**: {}\n", i.id));
                out.push_str(&format!("**Week**: {}\n", week));
                out.push_str(&format!("**Situation**: {}（{}）\n", label, sit_name));
                out.push_str(&format!("**Description**: {}\n", i.description));
                out.push_str(&format!("**Motivation**: {}\n", i.motivation));
                out.push_str(&format!("**Agent**: {} | **Level**: {}\n", i.agent.label, i.level.label));
                out.push_str(&format!(
                    "**Priority**: {} | **Risk**: {} | **Trigger**: {}\n",
                    i.priority.label, i.risk.label, i.trigger.label
                ));
                Ok(out)
            }
        }
    }

    /// Show filtered intentions table
    pub fn filter_intentions(
        &self,
        week: Option<&str>,
        sit_name: Option<&str>,
        priority: Option<&str>,
        risk: Option<&str>,
        level: Option<&str>,
        agent: Option<&str>,
    ) -> Result<String, String> {
        let results = self.engine.all_intentions(week, sit_name, priority, risk, level, agent)?;
        let reg = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = reg
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let mut out = String::new();
        out.push_str("| Week | Situation | Title | Priority | Risk | Level |\n");
        out.push_str("|------|-----------|-------|----------|------|-------|\n");
        for (w, sn, i) in &results {
            let label = label_map.get(sn.as_str()).cloned().unwrap_or_else(|| sn.clone());
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                w, label, i.title, i.priority.label, i.risk.label, i.level.label
            ));
        }
        out.push_str(&format!("\n{} results\n", results.len()));
        Ok(out)
    }

    // ── Phase 3: Intention relations ──

    /// LLM inference of intention relations within a week
    pub fn relate_intentions_llm(&self, week: &str) -> Result<String, String> {
        let cache_path = self.reports_dir(week).join("intention-relations.json");
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                eprintln!("(loaded cached intention relations)");
                return Self::format_intention_relations(week, &json);
            }
        }

        let client = crate::llm::DeepSeekClient::from_env()?;
        let data = self.engine.week(week)?;

        let mut intent_list = String::new();
        let mut idx = 0usize;
        let mut id_map = std::collections::HashMap::new();
        for (sn, intents) in &data.intention_map {
            for i in intents {
                id_map.insert(idx, (sn.clone(), i.title.clone()));
                intent_list.push_str(&format!(
                    "[{}] {} (情境: {}, 层级: {}, 优先级: {}, 风险: {})\n",
                    idx, i.title, sn, i.level.label, i.priority.label, i.risk.label
                ));
                idx += 1;
            }
        }

        let prompt = format!(
            r#"分析以下 {} 个意向之间的关系。关系类型：支持、冲突、触发、依赖、包含、演进。
每个关系输出格式：{{"source": 编号, "target": 编号, "type": "关系类型", "logic": "为什么"}}
只输出JSON数组。

{}"#,
            idx, intent_list
        );

        let raw = client.chat(&prompt)?;
        let json = crate::llm::extract_json(&raw)?;
        if let Ok(content) = serde_json::to_string_pretty(&json) {
            fs::write(&cache_path, &content).ok();
        }
        Self::format_intention_relations(week, &json)
    }

    fn format_intention_relations(week: &str, json: &serde_json::Value) -> Result<String, String> {
        let mut out = String::new();
        out.push_str(&format!("# Intention Relations: {}\n\n", week));

        // Build DAG: count in/out edges per node
        let mut in_deg: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let mut out_deg: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

        if let Some(arr) = json.as_array() {
            for rel in arr {
                let s = rel["source"].as_u64().unwrap_or(999) as usize;
                let t = rel["target"].as_u64().unwrap_or(999) as usize;
                let rtype = rel["type"].as_str().unwrap_or("?");
                let logic = rel["logic"].as_str().unwrap_or("");

                out.push_str(&format!("- [{}] → [{}]：{} — {}\n", s, t, rtype, logic));
                *in_deg.entry(t).or_default() += 1;
                *out_deg.entry(s).or_default() += 1;
            }
        }

        // Identify root intentions (high out_deg, low in_deg) and leaf intentions (high in_deg, low out_deg)
        out.push('\n');
        out.push_str("### DAG 分析\n\n");
        let mut root_items: Vec<(usize, usize)> = out_deg.iter().map(|(&k, &v)| (k, v)).filter(|(k, v)| *v > 1 && in_deg.get(k).copied().unwrap_or(0) <= 1).collect();
        root_items.sort_by(|a, b| b.1.cmp(&a.1));

        if !root_items.is_empty() {
            out.push_str("**核心驱动力（出度>1 且入度≤1）**:\n");
            for (k, v) in &root_items {
                out.push_str(&format!("  - [{}] (出度={})\n", k, v));
            }
            out.push('\n');
        }

        out.push_str("**元意图候选（身心健康等基础条件）**: 由 LLM 识别，需人工验证\n");

        Ok(out)
    }

    // ── Phase 4: Cross analysis ──

    /// Coverage: which situations have which intentions
    pub fn coverage(&self, week: &str) -> Result<String, String> {
        let data = self.engine.week(week)?;
        let reg = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = reg
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let mut out = String::new();
        out.push_str(&format!("# Coverage: {}\n\n", week));

        // Cross matrix: situations × intention properties
        out.push_str("| 情境 | 意向数 | 高优先级 | 高风险 | 顶层 | 底层 | agent:创始人 |\n");
        out.push_str("|------|--------|---------|-------|------|------|-------------|\n");

        for sit in &data.situations {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            let intents = data.intention_map.get(&sit.name);
            let count = intents.map(|v| v.len()).unwrap_or(0);
            let high_p = intents.map(|v| v.iter().filter(|i| i.priority.name == "high").count()).unwrap_or(0);
            let high_r = intents.map(|v| v.iter().filter(|i| i.risk.name == "high").count()).unwrap_or(0);
            let top = intents.map(|v| v.iter().filter(|i| i.level.name == "top").count()).unwrap_or(0);
            let bottom = intents.map(|v| v.iter().filter(|i| i.level.name == "bottom").count()).unwrap_or(0);
            let founder = intents.map(|v| v.iter().filter(|i| i.agent.name == "founder").count()).unwrap_or(0);
            out.push_str(&format!("| {} | {} | {} | {} | {} | {} | {} |\n",
                label, count, high_p, high_r, top, bottom, founder));
        }
        out.push('\n');
        Ok(out)
    }

    /// Tension: detect conflicts between top-level and bottom-level intentions
    pub fn tension(&self, week: &str) -> Result<String, String> {
        let data = self.engine.week(week)?;
        let reg = self.engine.registry().ok();
        let label_map: std::collections::HashMap<String, String> = reg
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();

        let mut out = String::new();
        out.push_str(&format!("# Tension: {}\n\n", week));

        // Group intentions by level
        let mut top: Vec<(String, String)> = Vec::new();  // (sit_label, title)
        let mut bottom: Vec<(String, String)> = Vec::new();

        for sit in &data.situations {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            if let Some(intents) = data.intention_map.get(&sit.name) {
                for i in intents {
                    match i.level.name.as_str() {
                        "top" => top.push((label.clone(), i.title.clone())),
                        "bottom" => bottom.push((label.clone(), i.title.clone())),
                        _ => {}
                    }
                }
            }
        }

        if top.is_empty() && bottom.is_empty() {
            out.push_str("未检测到层级冲突（无顶层或底层意图）。\n");
            return Ok(out);
        }

        out.push_str(&format!("**顶层意图**（{} 条）\n\n", top.len()));
        for (l, t) in &top {
            out.push_str(&format!("- {}：{}\n", l, t));
        }
        out.push('\n');

        out.push_str(&format!("**底层意图**（{} 条）\n\n", bottom.len()));
        for (l, t) in &bottom {
            out.push_str(&format!("- {}：{}\n", l, t));
        }
        out.push('\n');

        // Detect potential tensions: top intent in one situation vs bottom in same situation
        out.push_str("### 潜在冲突\n\n");
        let mut found = false;
        for (tl, tt) in &top {
            for (bl, bt) in &bottom {
                if tl == bl {
                    out.push_str(&format!("- **{}**：顶层「{}」与底层「{}」之间存在资源或注意力竞争\n", tl, tt, bt));
                    found = true;
                }
            }
        }
        if !found {
            out.push_str("（无同情境内的层级冲突）\n");
        }
        out.push('\n');

        Ok(out)
    }

    /// List schemas from gallery (loaded from file, can be empty)
    pub fn list_schemas(&self, week: &str) -> Result<String, String> {
        let schemas = self.engine.loader.load_schemas(week)?;
        if schemas.is_empty() {
            return Ok(format!("No schemas for {}", week));
        }
        let mut out = String::new();
        out.push_str(&format!("# Schemas: {}\n\n", week));
        for s in &schemas {
            out.push_str(&format!("## {}\n\n", s.label));
            out.push_str(&format!("**ID**: {}\n\n", s.id));

            let c = &s.content;
            if !c.entities.is_empty() {
                out.push_str("### Entities\n\n");
                for e in &c.entities {
                    out.push_str(&format!("- {}\n", serde_yaml::to_string(e).unwrap_or_default()));
                }
            }
            if !c.causals.is_empty() {
                out.push_str("### Causals\n\n");
                for f in &c.causals {
                    out.push_str(&format!("- IF {} THEN {}\n", f.condition, f.outcome));
                }
            }
            if !c.boundaries.is_empty() {
                out.push_str("### Boundaries\n\n");
                for b in &c.boundaries {
                    out.push_str(&format!("- {}\n", b));
                }
            }
            if !c.biases.is_empty() {
                out.push_str("### Biases\n\n");
                for b in &c.biases {
                    out.push_str(&format!("- **{}**（事实：{}）\n", b.belief, b.fact));
                }
            }
            out.push('\n');
        }
        Ok(out)
    }

    /// Public display command for schema inference
    /// Helper: get the previous week string
    fn previous_week(&self, week: &str) -> Option<String> {
        let weeks = self.engine.list_weeks().ok()?;
        let pos = weeks.iter().position(|w| w == week)?;
        if pos > 0 { Some(weeks[pos - 1].clone()) } else { None }
    }
}
