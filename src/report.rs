use crate::data::OutputSchema;

pub struct DimensionScore {
    pub name: &'static str,
    pub score: u8,
    pub detail: String,
}

pub struct Assessment {
    pub dimensions: Vec<DimensionScore>,
    pub total: f64,
}

fn score_coverage(schema: &OutputSchema) -> DimensionScore {
    let n_props = schema.properties.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_dynamics = schema.dynamics.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_entities = schema.entities.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_causals = schema.causals.as_ref().map(|v| v.len()).unwrap_or(0);

    if n_entities == 0 {
        return DimensionScore { name: "A-1 覆盖度", score: 1, detail: "无实体定义".into() };
    }

    let total_kv = n_props + n_dynamics;
    let expected = n_entities + 2;
    let score = if total_kv >= expected + 3 { 5 }
        else if total_kv >= expected + 1 { 4 }
        else if total_kv >= expected.saturating_sub(1) { 3 }
        else if total_kv >= 1 { 2 }
        else { 1 };

    DimensionScore {
        name: "A-1 覆盖度",
        score,
        detail: format!("{}个属性+{}个动态 覆盖 {}个实体 {}条因果", n_props, n_dynamics, n_entities, n_causals),
    }
}

fn score_flexibility(schema: &OutputSchema) -> DimensionScore {
    let causals = match schema.causals.as_ref() {
        Some(c) => c,
        None => return DimensionScore { name: "A-2 灵活性", score: 1, detail: "无因果链".into() },
    };

    let total = causals.len();
    let typed: Vec<_> = causals.iter().filter(|c| c.causal_type.is_some()).collect();
    let verifiable: Vec<_> = causals.iter().filter(|c| c.causal_type.as_deref() == Some("需验证")).collect();
    let verified: Vec<_> = verifiable.iter().filter(|c| c.verify.is_some()).collect();

    if typed.is_empty() {
        DimensionScore { name: "A-2 灵活性", score: 1, detail: format!("{}条因果均无type标记", total) }
    } else if verifiable.is_empty() {
        DimensionScore { name: "A-2 灵活性", score: 2, detail: format!("{}条已标记但无需验证类", typed.len()) }
    } else if verified.is_empty() {
        DimensionScore { name: "A-2 灵活性", score: 3, detail: format!("{}条需验证类但无verify条件", verifiable.len()) }
    } else if verified.len() == verifiable.len() {
        DimensionScore { name: "A-2 灵活性", score: 5, detail: format!("全部{}条需验证类均有verify条件 ✓", verifiable.len()) }
    } else {
        DimensionScore { name: "A-2 灵活性", score: 4, detail: format!("{}/{}条需验证类有verify条件", verified.len(), verifiable.len()) }
    }
}

fn score_complexity(schema: &OutputSchema) -> DimensionScore {
    let n_entities = schema.entities.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_causals = schema.causals.as_ref().map(|v| v.len()).unwrap_or(0);

    if n_entities == 0 {
        return DimensionScore { name: "A-3 复杂度", score: 1, detail: "无实体".into() };
    }

    let entity_ok = (3..=7).contains(&n_entities);
    let ratio = n_causals as f64 / n_entities as f64;

    let (score, detail) = if entity_ok && (1.5..=4.0).contains(&ratio) {
        (5, format!("{}个实体 {}条因果 — 均衡", n_entities, n_causals))
    } else if entity_ok && (1.0..=5.0).contains(&ratio) {
        (4, format!("{}个实体 {}条因果 — 比例略偏", n_entities, n_causals))
    } else if (2..=10).contains(&n_entities) {
        (3, format!("{}个实体 {}条因果 — 可接受", n_entities, n_causals))
    } else if n_entities <= 12 {
        (2, format!("{}个实体 — 偏多", n_entities))
    } else {
        (1, format!("{}个实体 — 过多", n_entities))
    };

    DimensionScore { name: "A-3 复杂度", score, detail }
}

fn score_consistency(schema: &OutputSchema) -> DimensionScore {
    let n_causals = schema.causals.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_biases = schema.biases.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_mappings = schema.mappings.as_ref().map(|v| v.len()).unwrap_or(0);

    let (score, detail) = if n_causals >= 3 && n_biases >= 2 {
        (4, format!("{}条因果 {}条偏见 {}条映射 — 结构一致", n_causals, n_biases, n_mappings))
    } else if n_causals >= 1 && n_biases >= 1 {
        (3, format!("可交叉验证（{}条因果 {}条偏见）", n_causals, n_biases))
    } else {
        (2, format!("数据不足（{}条因果 {}条偏见）", n_causals, n_biases))
    };

    DimensionScore { name: "B-1 内部一致性", score, detail }
}

fn score_external_validity(schema: &OutputSchema) -> DimensionScore {
    let has_verify = schema.causals.as_ref()
        .map(|c| c.iter().any(|c| c.verify.is_some()))
        .unwrap_or(false);
    let has_ids = schema.biases.as_ref()
        .map(|b| b.iter().any(|b| !b.id.is_nil()))
        .unwrap_or(false);
    let has_sources = schema.causals.as_ref()
        .map(|c| c.iter().any(|c| c.note.is_some()))
        .unwrap_or(false);

    let indicators = [has_sources, has_verify, has_ids].iter().filter(|&&x| x).count();
    let (score, detail) = match indicators {
        3 => (5, "全部3项可追溯指标均存在"),
        2 => (4, "部分可追溯（2/3指标）"),
        1 => (3, "少量可追溯标记"),
        _ => (2, "无可追溯标记"),
    };

    DimensionScore { name: "B-2 外部有效性", score, detail: detail.into() }
}

fn score_task_fit(schema: &OutputSchema) -> DimensionScore {
    let usage = schema.usage.as_deref().unwrap_or("");
    let n_causals = schema.causals.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_mappings = schema.mappings.as_ref().map(|v| v.len()).unwrap_or(0);

    let has_good_usage = usage.len() > 30;
    let has_causals = n_causals >= 3;
    let has_mappings = n_mappings >= 2;

    let (score, detail) = match (has_good_usage, has_causals, has_mappings) {
        (true, true, true) => (5, format!("usage清晰 {}条因果 {}条映射 — 可执行", n_causals, n_mappings)),
        (_, true, true) => (4, "有因果和映射但usage可更精确".to_string()),
        (_, true, false) | (_, false, true) => (3, "部分覆盖（有因果或有映射）".to_string()),
        _ => (2, "usage已定义但缺少可执行内容".to_string()),
    };

    DimensionScore { name: "B-3 任务适用性", score, detail }
}

fn score_communicability(schema: &OutputSchema) -> DimensionScore {
    let usage = schema.usage.as_deref().unwrap_or("");
    let n_boundaries = schema.boundaries.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_entities = schema.entities.as_ref().map(|v| v.len()).unwrap_or(0);
    let n_biases = schema.biases.as_ref().map(|v| v.len()).unwrap_or(0);

    let mut score = 1u8;
    let mut parts: Vec<String> = Vec::new();
    if usage.len() > 20 { score += 1; parts.push("usage清晰".to_string()); }
    if n_boundaries >= 3 { score += 1; parts.push(format!("{}条边界", n_boundaries)); }
    if n_biases >= 1 { score += 1; parts.push(format!("{}条偏见", n_biases)); }
    if (3..=7).contains(&n_entities) { score += 1; parts.push(format!("{}个实体", n_entities)); }

    let detail = if parts.is_empty() { "可读性待改进".into() } else { parts.join(" ") };

    DimensionScore { name: "B-4 可沟通性", score: score.min(5), detail }
}

pub fn assess(schema: &OutputSchema) -> Assessment {
    let dims = vec![
        score_coverage(schema),
        score_flexibility(schema),
        score_complexity(schema),
        score_consistency(schema),
        score_external_validity(schema),
        score_task_fit(schema),
        score_communicability(schema),
    ];
    let total = dims.iter().map(|d| d.score as f64).sum::<f64>() / dims.len() as f64;
    Assessment { dimensions: dims, total }
}

pub fn format_report(assessment: &Assessment) -> String {
    let mut lines = vec!["# Schema 质量评估报告（自动）".to_string()];
    lines.push("".into());
    lines.push("| 维度 | 分数 | 说明 |".into());
    lines.push("|------|------|------|".into());
    for d in &assessment.dimensions {
        lines.push(format!("| {} | {}/5 | {} |", d.name, d.score, d.detail));
    }
    lines.push("".into());
    lines.push(format!("**总分**：{:.2}/5", assessment.total));
    lines.join("\n")
}
