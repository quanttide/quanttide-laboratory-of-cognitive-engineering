# Causal Decomposition Prompt

Use this prompt to classify causals extracted from journal data into retainable (保留类) or verifiable (需验证类).

## Context

**Retainable (保留类)**: Cross-organization通用 causal that can be directly written into schema.
Corresponds to Piaget's **assimilation** — new info fits into existing schema.

**Verifiable (需验证类)**: Causal that depends on specific client conditions (org structure, power relations, culture, etc.).
Corresponds to Piaget's **accommodation** — schema must be adjusted to fit new info.
Must include a `verify` field describing what conditions need to be checked.

## Classification Rules

A causal is **retainable** if:
- It describes universal human/technical规律 (e.g. "AI自收敛存在局限" → needs human feedback)
- It is a general engineering principle (e.g. "过度形式化扼杀探索空间")
- It is a core theoretical assumption that跨组织成立 (e.g. "所有结构化产出始于意图")

A causal is **verifiable** if:
- It depends on the client's organizational structure (e.g. "顾问处于信息孤岛 → 无法感知政治博弈")
- It depends on specific team capabilities (e.g. "需要形式化工具 → 团队是否有基础理解")
- It depends on the client's decision-making culture (e.g. "客户对范式共识为伪共识")
- Its truth条件 varies by organization (e.g. "契约在多个场景验证有效 → 方法论可深化")

## Input Format

```yaml
domain: <domain_name>
weeks: [W19, W21, ...]
causals:
  - condition: "..."
    outcome: "..."
    source_week: W23
```

## Output Format

```yaml
causals:
  - condition: "..."
    outcome: "..."
    type: 保留 | 需验证
    rationale: "简短的理由说明"
    verify:  # only for 需验证
      condition: "验证通过的条件"
      method: "interview | observation | document_review"
```

## Examples

Input:
```yaml
causals:
  - condition: 单一AI自收敛存在局限性
    outcome: 需要人机协同反思
  - condition: 契约在多个场景验证有效
    outcome: 方法论可深化
```

Output:
```yaml
causals:
  - condition: 单一AI自收敛存在局限性
    outcome: 需要人机协同反思
    type: 保留
    rationale: "跨组织通用的人机协同局限"
  - condition: 契约在多个场景验证有效
    outcome: 方法论可深化
    type: 需验证
    rationale: "依赖客户是否有多个实际场景"
    verify:
      condition: "至少覆盖3个不同领域"
      method: interview
```
