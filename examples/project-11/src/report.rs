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

        // — Relations (placeholder) —
        out.push_str("## 关键关系\n\n");
        out.push_str("（待 LLM 推理）\n\n");

        // — Mental Models (placeholder) —
        out.push_str("## 跨情境心智模型\n\n");
        out.push_str("（待 LLM 推理）\n\n");

        // — Comparison —
        out.push_str("## 与前周对比\n\n");
        out.push_str("（待实现跨周差异分析）\n");

        Ok(out)
    }
}
