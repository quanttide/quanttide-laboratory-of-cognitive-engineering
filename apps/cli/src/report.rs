use std::fs;
use std::path::PathBuf;

use quanttide_agent::message::Message;

use crate::models::{Intention, Schema, Situation, WeekData};
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

    /// Generate a gallery-format report with LLM synthesis + data fallback
    pub fn report(&self, week: &str) -> Result<String, String> {
        let data = self.engine.week(week)?;
        let registry = self.engine.registry().ok().unwrap_or_default();
        let category = self.engine.loader.load_category().ok().unwrap_or_default();
        // Use registry first, fall back to category for label map
        let mut label_map: std::collections::HashMap<String, String> = registry
            .into_iter()
            .map(|e| (e.name, e.label))
            .collect();
        if label_map.is_empty() {
            label_map = category.iter().map(|e| (e.name.clone(), e.label.clone())).collect();
        }
        let category_order: std::collections::HashMap<String, usize> = category
            .into_iter()
            .enumerate()
            .map(|(i, e)| (e.name, i))
            .collect();

        let mut sorted_sits = data.situations.clone();
        sorted_sits.sort_by(|a, b| {
            let ai = category_order.get(&a.name).copied().unwrap_or(usize::MAX);
            let bi = category_order.get(&b.name).copied().unwrap_or(usize::MAX);
            ai.cmp(&bi)
        });

        let schemas = self.engine.loader.load_schemas(week).ok().unwrap_or_default();

        // Try LLM report; fall back to data-only
        let result = self.llm_gallery_report(week, &data, &label_map, &sorted_sits, &schemas);
        let report = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("LLM report failed ({}), generating data-only report", e);
                self.data_gallery_report(week, &data, &label_map, &sorted_sits, &schemas)
            }
        };

        let report_path = self.reports_dir(week).join("report.md");
        fs::write(&report_path, &report).ok();
        println!("Report saved to: {:?}", report_path);
        Ok(report)
    }

    /// Gallery-format report using LLM synthesis
    fn llm_gallery_report(
        &self,
        week: &str,
        data: &WeekData,
        label_map: &std::collections::HashMap<String, String>,
        sorted_sits: &[Situation],
        schemas: &[Schema],
    ) -> Result<String, String> {
        let cache_path = self.reports_dir(week).join("gallery-report.json");
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                eprintln!("(loaded cached gallery report)");
                return Self::render_gallery_report(week, &json, label_map);
            }
        }

        let client = quanttide_agent::llm::LLM::default();

        // Build domain descriptions for prompt
        let mut domain_text = String::new();
        for sit in sorted_sits {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            let intents = data.intention_map.get(&sit.name).cloned().unwrap_or_default();
            let schema = schemas.iter().find(|s| s.name == sit.name);

            domain_text.push_str(&format!("## {} ({})\n", label, sit.name));
            domain_text.push_str(&format!("agenda: {}\n", sit.content.agenda));
            domain_text.push_str(&format!("ecology: {}\n", sit.content.ecology));
            domain_text.push_str(&format!("frame: {}\n", sit.content.frame));
            domain_text.push_str(&format!("dynamics: {}\n", sit.content.dynamics));
            if let Some(s) = schema {
                domain_text.push_str(&format!("schema_usage: {}\n", s.content.usage));
                if !s.content.causals.is_empty() {
                    domain_text.push_str("causals:\n");
                    for c in &s.content.causals {
                        domain_text.push_str(&format!("  - IF {} THEN {}\n", c.condition, c.outcome));
                    }
                }
            }
            if !intents.is_empty() {
                domain_text.push_str("intentions:\n");
                for i in &intents {
                    domain_text.push_str(&format!(
                        "  - {} [priority={}, risk={}, level={}, trigger={}]\n",
                        i.title, i.priority.label, i.risk.label, i.level.label, i.trigger.label
                    ));
                }
            }
            domain_text.push('\n');
        }

        // Collect domain names for pair generation
        let domain_names: Vec<String> = sorted_sits.iter()
            .map(|s| label_map.get(&s.name).cloned().unwrap_or_else(|| s.name.clone()))
            .collect();

        let prompt = format!(
            r#"你是一个认知工程报告分析师。基于以下一周的情境数据，生成一份结构化的认知工程报告。

报告格式要求（JSON）：
{{
  "summary": "一周摘要（3-5句，点出主线、变化、关键信号）",
  "domains": [
    {{
      "name": "情境标识",
      "schema_mining": "图式挖掘：一句话提炼该领域的认知模式/心智模型（从 frame 和 schema 中提取核心因果逻辑和边界条件）",
      "situation_awareness": "情境意识：一句话描述当前发生了什么（从 agenda 和 ecology 中提炼）",
      "intention_recognition": "意图识别：一句话概括该领域的关键意图及优先级/风险（列出顶层意图或最高优意图）"
    }}
  ],
  "relations": [
    {{
      "source": "情境A标识",
      "target": "情境B标识",
      "schema_association": "图式关联：两个领域的认知模式之间的关系",
      "situation_association": "情境关联：两个领域当前状态之间的关系",
      "intention_association": "意图关联：两个领域意图之间的张力或协作关系"
    }}
  ],
  "appendix_items": [
    "待决策项、跨领域风险等非标准内容，每条一句话"
  ]
}}

图式关联、情境关联、意图关联各写一句话。
关系对数量：不要超过所有领域两两组合数的一半，只选最有意义的关系。

周次：{}

领域列表（按此顺序排序）：{}

以下是各领域数据：
{}

只输出JSON，不要额外文字。"#,
            week,
            domain_names.join(", "),
            domain_text,
        );

        let raw = client.complete(&[Message::new("user", &prompt)], Default::default()).map_err(|e| format!("LLM: {}", e.0))?.content;
        let json = crate::llm::parse_structured_output(&raw)?;

        if let Ok(content) = serde_json::to_string_pretty(&json) {
            fs::write(&cache_path, &content).ok();
            eprintln!("(cached gallery report to {:?})", cache_path);
        }

        Self::render_gallery_report(week, &json, label_map)
    }

    /// Render LLM JSON into gallery-format markdown
    fn render_gallery_report(
        week: &str,
        json: &serde_json::Value,
        label_map: &std::collections::HashMap<String, String>,
    ) -> Result<String, String> {
        let mut out = String::new();
        out.push_str(&format!("# {} 认知工程报告\n\n", week));

        // Summary
        let summary = json["summary"].as_str().unwrap_or("（待生成）");
        out.push_str("## 摘要\n\n");
        out.push_str(summary);
        out.push_str("\n\n");

        // Domains
        out.push_str("## 领域描述\n\n");
        if let Some(domains) = json["domains"].as_array() {
            for d in domains {
                let name = d["name"].as_str().unwrap_or("?");
                let label = label_map.get(name).cloned().unwrap_or_else(|| name.to_string());
                let schema = d["schema_mining"].as_str().unwrap_or("（待生成）");
                let awareness = d["situation_awareness"].as_str().unwrap_or("（待生成）");
                let intention = d["intention_recognition"].as_str().unwrap_or("（待生成）");

                out.push_str(&format!("### {}\n\n", label));
                out.push_str(&format!("- 图式挖掘：{}\n", schema));
                out.push_str(&format!("- 情境意识：{}\n", awareness));
                out.push_str(&format!("- 意图识别：{}\n\n", intention));
            }
        }

        // Relations
        out.push_str("## 领域关系\n\n");
        if let Some(relations) = json["relations"].as_array() {
            if !relations.is_empty() {
                for r in relations {
                    let source = r["source"].as_str().unwrap_or("?");
                    let target = r["target"].as_str().unwrap_or("?");
                    let slabel = label_map.get(source).cloned().unwrap_or_else(|| source.to_string());
                    let tlabel = label_map.get(target).cloned().unwrap_or_else(|| target.to_string());
                    let sa = r["schema_association"].as_str().unwrap_or("");
                    let sia = r["situation_association"].as_str().unwrap_or("");
                    let ia = r["intention_association"].as_str().unwrap_or("");

                    out.push_str(&format!("### {} vs {}\n\n", slabel, tlabel));
                    if !sa.is_empty() {
                        out.push_str(&format!("- 图式关联：{}\n", sa));
                    }
                    if !sia.is_empty() {
                        out.push_str(&format!("- 情境关联：{}\n", sia));
                    }
                    if !ia.is_empty() {
                        out.push_str(&format!("- 意图关联：{}\n", ia));
                    }
                    out.push('\n');
                }
            } else {
                out.push_str("（未发现显著关系）\n\n");
            }
        } else {
            out.push_str("（未发现显著关系）\n\n");
        }

        // Appendix
        if let Some(items) = json["appendix_items"].as_array() {
            if !items.is_empty() {
                out.push_str("## 附录\n\n");
                for item in items {
                    let text = item.as_str().unwrap_or("");
                    if !text.is_empty() {
                        out.push_str(&format!("- {}\n", text));
                    }
                }
                out.push('\n');
            }
        }

        Ok(out)
    }

    /// Data-only gallery report with synthesized content (no LLM)
    fn data_gallery_report(
        &self,
        week: &str,
        data: &WeekData,
        label_map: &std::collections::HashMap<String, String>,
        sorted_sits: &[Situation],
        schemas: &[Schema],
    ) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {} 认知工程报告\n\n", week));

        // — Summary —
        let total = data.intentions.len();
        let high_p = data.intentions.iter().filter(|i| i.priority.name == "high").count();
        let high_r = data.intentions.iter().filter(|i| i.risk.name == "high").count();
        let top = data.intentions.iter().filter(|i| i.level.name == "top").count();
        let bottom = data.intentions.iter().filter(|i| i.level.name == "bottom").count();
        let top_intents: Vec<&str> = data.intentions.iter().filter(|i| i.level.name == "top").map(|i| i.title.as_str()).collect();

        out.push_str("## 摘要\n\n");
        out.push_str(&format!(
            "本周 {} 个领域，{} 条意图（高优先级 {}，高风险 {}，顶层 {}，底层 {}）。",
            sorted_sits.len(), total, high_p, high_r, top, bottom,
        ));
        if !top_intents.is_empty() {
            out.push_str(&format!(" 顶层意图：{}。", top_intents.join("、")));
        }
        out.push_str(&format!(
            " 领域排序：{}。",
            sorted_sits.iter()
                .map(|s| label_map.get(&s.name).cloned().unwrap_or_else(|| s.name.clone()))
                .collect::<Vec<_>>().join("、")
        ));
        out.push_str("\n\n");

        // — Domains —
        out.push_str("## 领域描述\n\n");
        for sit in sorted_sits {
            let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
            let intents = data.intention_map.get(&sit.name).cloned().unwrap_or_default();
            let schema = schemas.iter().find(|s| s.name == sit.name);

            // 图式挖掘: synthesize from frame + schema
            let mut schema_parts: Vec<String> = Vec::new();
            if let Some(s) = schema {
                if !s.content.usage.is_empty() {
                    schema_parts.push(s.content.usage.clone());
                }
                for c in &s.content.causals {
                    schema_parts.push(format!("若{}则{}", c.condition, c.outcome));
                }
                for b in &s.content.boundaries {
                    schema_parts.push(format!("边界：{}", b));
                }
                for b in &s.content.biases {
                    schema_parts.push(format!("常见误区：{}（事实：{}）", b.belief, b.fact));
                }
            }
            let schema_text = if schema_parts.is_empty() {
                sit.content.frame.clone()
            } else {
                schema_parts.join("；")
            };

            // 情境意识: clean agenda + ecology
            let awareness_text = format!("议程：{} 现状：{}", sit.content.agenda, sit.content.ecology);

            // 意图识别: group by level with details
            let intention_text = if intents.is_empty() {
                "（无明确意图数据）".to_string()
            } else {
                let detail: Vec<String> = intents.iter().map(|i| {
                    format!("{}（{}优先级，{}风险，{}）", i.title, i.priority.label, i.risk.label, i.trigger.label)
                }).collect();
                detail.join("；")
            };

            out.push_str(&format!("### {}\n\n", label));
            out.push_str(&format!("- 图式挖掘：{}\n", schema_text));
            out.push_str(&format!("- 情境意识：{}\n", awareness_text));
            out.push_str(&format!("- 意图识别：{}\n\n", intention_text));
        }

        // — Relations: compute from shared frame + schema patterns —
        out.push_str("## 领域关系\n\n");
        let mut rel_count = 0usize;
        for i in 0..sorted_sits.len() {
            for j in (i+1)..sorted_sits.len() {
                let a = &sorted_sits[i];
                let b = &sorted_sits[j];
                let alabel = label_map.get(&a.name).cloned().unwrap_or_else(|| a.name.clone());
                let blabel = label_map.get(&b.name).cloned().unwrap_or_else(|| b.name.clone());
                let sa = schemas.iter().find(|s| s.name == a.name);
                let sb = schemas.iter().find(|s| s.name == b.name);

                // Score how connected these two domains are
                let a_entities: Vec<String> = sa.map(|s| s.content.entities.iter()
                    .map(|e| e.name.clone())
                    .collect()).unwrap_or_default();
                let b_entities: Vec<String> = sb.map(|s| s.content.entities.iter()
                    .map(|e| e.name.clone())
                    .collect()).unwrap_or_default();
                let shared_entities: Vec<&str> = a_entities.iter().filter_map(|e|
                    if b_entities.contains(e) { Some(e.as_str()) } else { None }
                ).collect();

                let a_frame = a.content.frame.to_lowercase();
                let b_frame = b.content.frame.to_lowercase();
                let keywords = ["设计", "体系", "范式", "平衡", "转型", "迭代", "风险", "演化", "框架", "边界", "可持续", "探索", "整合", "人机", "协作"];
                let shared_kw: Vec<&str> = keywords.iter().filter(|kw|
                    a_frame.contains(*kw) && b_frame.contains(*kw)
                ).copied().collect();

                let a_has_top = data.intention_map.get(&a.name).map(|v| v.iter().any(|i| i.level.name == "top")).unwrap_or(false);
                let b_has_bottom = data.intention_map.get(&b.name).map(|v| v.iter().any(|i| i.level.name == "bottom")).unwrap_or(false);
                let a_has_bottom = data.intention_map.get(&a.name).map(|v| v.iter().any(|i| i.level.name == "bottom")).unwrap_or(false);
                let b_has_top = data.intention_map.get(&b.name).map(|v| v.iter().any(|i| i.level.name == "top")).unwrap_or(false);
                let has_top_bottom = (a_has_top && b_has_bottom) || (b_has_top && a_has_bottom);

                // Skip if no meaningful connection
                let meaningful = shared_entities.len() >= 1 || shared_kw.len() >= 2 || has_top_bottom;
                if !meaningful {
                    continue;
                }

                let mut schema_assoc = String::new();
                let mut situation_assoc = String::new();
                let mut intention_assoc = String::new();

                if !shared_entities.is_empty() {
                    schema_assoc.push_str(&format!("共享概念：{}。", shared_entities.join("、")));
                }
                if !shared_kw.is_empty() {
                    schema_assoc.push_str(&format!("共同认知框架：{}。", shared_kw.join("、")));
                }

                let a_dyn = &a.content.dynamics;
                let b_dyn = &b.content.dynamics;
                let a_has_evolve = a_dyn.contains("演化") || a_dyn.contains("迭代") || a_dyn.contains("转型");
                let b_has_evolve = b_dyn.contains("演化") || b_dyn.contains("迭代") || b_dyn.contains("转型");
                if a_has_evolve && b_has_evolve {
                    situation_assoc.push_str(&format!("{}和{}都处于演化阶段。", alabel, blabel));
                }

                if has_top_bottom {
                    let (top_label, bottom_label) = if a_has_top && b_has_bottom {
                        (alabel.as_str(), blabel.as_str())
                    } else {
                        (blabel.as_str(), alabel.as_str())
                    };
                    intention_assoc.push_str(&format!("{}的顶层意图与{}的底层意图形成张力——前者提供方向，后者提供基础条件。", top_label, bottom_label));
                }

                if schema_assoc.is_empty() {
                    schema_assoc = "暂无直接共享图式，由意图层级互补关系发现。".to_string();
                }
                if situation_assoc.is_empty() {
                    situation_assoc = "（无显著情境关联）".to_string();
                }
                out.push_str(&format!("### {} vs {}\n\n", alabel, blabel));
                out.push_str(&format!("- 图式关联：{}\n", schema_assoc));
                out.push_str(&format!("- 情境关联：{}\n", situation_assoc));
                out.push_str(&format!("- 意图关联：{}\n\n", intention_assoc));
                rel_count += 1;
            }
        }
        if rel_count == 0 {
            out.push_str("（未发现显著关系）\n\n");
        }

        // — Appendix: decision items from tensions, cross-domain risks —
        out.push_str("## 附录\n\n");

        // Decision items from high-risk top intentions
        let tension_items: Vec<&Intention> = data.intentions.iter()
            .filter(|i| i.risk.name == "high" && i.level.name == "top").collect();
        for i in &tension_items {
            let sit_label = sorted_sits.iter()
                .find(|s| data.intention_map.get(&s.name).map_or(false, |v| v.iter().any(|x| x.id == i.id)))
                .and_then(|s| label_map.get(&s.name))
                .cloned().unwrap_or_default();
            out.push_str(&format!("- 待决策（{}，{}）：{}——{}\n", sit_label, i.title, i.description, i.motivation));
        }

        // Cross-domain: same entity appearing in multiple domains
        let entity_domains: std::collections::HashMap<String, Vec<String>> = {
            let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
            for sit in sorted_sits {
                let label = label_map.get(&sit.name).cloned().unwrap_or_else(|| sit.name.clone());
                let schema = schemas.iter().find(|s| s.name == sit.name);
                if let Some(s) = schema {
                    for e in &s.content.entities {
                        map.entry(e.name.clone()).or_default().push(label.clone());
                    }
                }
            }
            map
        };
        for (entity, domains) in &entity_domains {
            if domains.len() > 1 {
                out.push_str(&format!("- 跨领域信号：「{}」出现在 {} 中，表明该概念在不同认知域间流动。\n", entity, domains.join("、")));
            }
        }

        // Priority conflict: high-bot vs top-low
        let high_bot: Vec<&Intention> = data.intentions.iter()
            .filter(|i| i.priority.name == "high" && i.level.name == "bottom").collect();
        for i in &high_bot {
            let sit_label = sorted_sits.iter()
                .find(|s| data.intention_map.get(&s.name).map_or(false, |v| v.iter().any(|x| x.id == i.id)))
                .and_then(|s| label_map.get(&s.name))
                .cloned().unwrap_or_default();
            out.push_str(&format!("- 注意力竞争：{}的底层意图「{}」被标记为高优先级，说明基础事务正在争夺战略注意力。\n", sit_label, i.title));
        }

        out.push('\n');
        out
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

        let client = quanttide_agent::llm::LLM::default();
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

        let raw = client.complete(&[Message::new("user", &prompt)], Default::default()).map_err(|e| format!("LLM: {}", e.0))?.content;
        let json = crate::llm::parse_structured_output(&raw)?;

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

        let client = quanttide_agent::llm::LLM::default();
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

        let raw = client.complete(&[Message::new("user", &prompt)], Default::default()).map_err(|e| format!("LLM: {}", e.0))?.content;
        let json = crate::llm::parse_structured_output(&raw)?;
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

}
