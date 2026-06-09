# Project 11: Situation Engine

基于 `docs/gallery/` 最新数据结构重新设计的情境工具。

## 背景

旧的 `apps/situation` 基于 `situation-graph.json`（单文件、意图聚类、关键词索引、图推理）。经过 W19-W23 的数据重构，情境数据已采用全新的结构化格式：

- `situation/2026-W{19,20,21,22,23}/`：每个情境按周组织为独立 YAML 文件
- `intention/2026-W{19,20,21,22,23}/`：每个意向按情境组织为独立 YAML 文件
- `situation/registry.yaml`：情境名称与标签映射表

每条情境记录包含 `agenda/ecology/frame/dynamics` 四个认知工程维度；每条意向记录包含 `agent/level/priority/trigger/risk` 等结构化字段。

## 目标

读取 gallery 数据源，提供以下能力：

### 1. 情境感知

- 加载每周的 situation YAML 文件
- 按 `name`（如 `org`、`infra`、`think`）跨周聚合，追踪演化轨迹
- 对比同一情境在不同周的 `agenda/ecology/frame/dynamics` 变化

### 2. 意向管理

- 按周、按情境查询意向
- 基于 `priority/risk` 进行过滤与排序
- 跨情境发现高优先级的意图集群

### 3. 关系推理

- 基于 `agenda` 和 `ecology` 的内容，让 LLM 动态推理情境间关系（支持、冲突、触发、演化等）
- 不再依赖预定义的 `situation-relation.yaml`
- 输出结构化的关系图谱

### 4. 周报生成

- 输入一周数据，输出该周的情境全景
- 结构：情境维度摘要 → 高优先级意向 → 情境间关系 → 演化趋势

## 数据结构

### 情境输入

```yaml
# situation/2026-W23/org.yaml
id: uuid
name: org
label: 组织管理
content:
  agenda: 任务与目标描述
  ecology: 外部环境与系统状态
  frame: 用户心理表征
  dynamics: 动态时间演化
```

### 意向输入

```yaml
# intention/2026-W23/org.yaml
- id: uuid
  title: 意向名称
  description: 意向描述
  motivation: 动机
  agent:
    name: founder
    label: 创始人
  level:
    name: middle
    label: 中层
  priority:
    name: high
    label: 高
    description: 优先级说明
  trigger:
    name: persistent
    label: 持续
    description: 触发条件
  risk:
    name: medium
    label: 中
    description: 风险说明
```

### 关系输出

```yaml
- source: org
  target: meta
  type: support
  logic: 团队反脆弱需要标准化的制度支撑
  strength: medium
  evidence_week: W23
```

## 设计原则

1. **数据源即 gallery**：不复制数据，通过路径引用 `docs/gallery/` 的 YAML 文件
2. **运行时推理替代预定义**：关系不由 YAML 定义，由 LLM 在运行时动态推理
3. **按周聚合，跨周追踪**：情境有演化轨迹，而非静态标签
4. **意图优先**：每条意向是独立的认知单元，情境是其聚合容器

## 任务分解

### Phase 1: 数据加载

- 加载 registry 获取情境映射表
- 加载指定周的所有 situation YAML
- 加载指定周的所有 intention YAML
- 支持按周、按情境名、按名称三种查询方式

### Phase 2: 核心查询

- `get_week(week)`: 返回该周的情境全景
- `get_situation(name, week?)`: 返回指定情境（可跨周）
- `get_intentions(name, week)`: 返回指定情境的意向列表
- `list_situations(week)`: 返回该周有哪些情境

### Phase 3: 关系推理

- `infer_relations(week)`: 让 LLM 分析同周内所有情境对，输出关系
- `infer_motif(week)`: 扫描跨情境的持续关切（母题发现）

### Phase 4: REPL / CLI

- `show <week>`: 打印该周全景
- `explore <name>`: 追踪某情境的跨周演化
- `relate <week>`: 推理该周的情境关系
- `report <week>`: 生成周报

## 数据路径约定

```
# 情境文件
docs/gallery/situation/{week}/{name}.yaml

# 意向文件
docs/gallery/intention/{week}/{name}.yaml

# 注册表
docs/gallery/situation/registry.yaml
```

## 与旧 apps/situation 的区别

| 维度 | 旧版 (apps/situation) | 新版 (project-11) |
|------|----------------------|-------------------|
| 数据源 | `situation-graph.json`（单文件） | `gallery/` 目录（多 YAML） |
| 数据格式 | 意图聚类 + 关键词索引 + 图边 | `agenda/ecology/frame/dynamics` 四维 |
| 关系来源 | 预定义 YAML | LLM 运行时推理 |
| 推理引擎 | 关键词匹配 + BFS | 多维内容分析 + LLM |
| 输出 | 交互式 REPL | REPL + 结构化的情境报告 |
