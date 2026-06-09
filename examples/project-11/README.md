# Project 11: Situation + Intention Engine

基于 `docs/gallery/` 的结构化 YAML 数据，提供情境分析和意向管理的能力。

## 数据源

- `situation/{week}/{name}.yaml`：情境定义（agenda/ecology/frame/dynamics）
- `intention/{week}/{name}.yaml`：意向定义（agent/level/priority/trigger/risk）
- `situation/registry.yaml`：情境名称与标签映射表

## 命令参考

### 情境

| 命令 | 功能 |
|------|------|
| `weeks` | 列出可用周 |
| `show <week>` | 周报摘要（数据转储） |
| `landscape <week>` | 紧凑全景表 |
| `explore <name>` | 跨周演化追踪 |
| `report <week>` | 六特征结构化周报（保存至 `reports/{week}/`）|
| `diff <weekA> <weekB>` | 跨周差异分析 |
| `relate <week>` | LLM 推理情境间关系（缓存至 `reports/{week}/relations.json`）|

### 意向

| 命令 | 功能 |
|------|------|
| `intentions [week] [name]` | 按周/按情境列出意向 |
| `intention <uuid>` | 按 UUID 查看意向详情 |
| `filter --priority high --risk high` | 多条件过滤（支持 `--week`/`--sit`/`--priority`/`--risk`/`--level`/`--agent`）|
| `trace <title>` | 跨周模糊搜索意图 |
| `drift <weekA> <weekB> <name>` | 比较两周间 priority/risk 偏移 |
| `evolve <name>` | 意图演化矩阵（周×意向×优先级/风险）|
| `ri <week>` | LLM 推理同周意向间关系 + DAG 分析（缓存至 `reports/{week}/intention-relations.json`）|

### 交叉分析

| 命令 | 功能 |
|------|------|
| `coverage <week>` | 情境覆盖分析（意向数/高优先级/高风险/层级/agent 分布）|
| `tension <week>` | 层级冲突检测（顶层 vs 底层意图）|

## 环境变量

- `GALLERY_PATH`：指向 `docs/gallery/` 目录（默认 `../../../docs/gallery`）
- `DEEPSEEK_API_KEY`：LLM 推理所需的 API 密钥
- `DEEPSEEK_MODEL`：模型名（默认 `deepseek-chat`）

## 已实现

- 情境：跨周加载、演化追踪、关系推理（LLM）、差异分析、结构化周报
- 意向：按周/情境/UUID 查询、多条件过滤、跨周追踪、priority/risk drift、演化矩阵
- 意图关系：LLM 推理同周意向关系 + DAG 分析
- 交叉分析：coverage 覆盖分析、tension 层级冲突检测
- 持久化：`relate` 和 `report` 输出保存至 `reports/{week}/`
