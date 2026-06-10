# TODO

## 阶段一：自然语言 REPL

当前 REPL 是命令驱动的（`show <week>`、`report <week>`、`discover <query>`）。目标：用户输入自然语言，系统自动路由。

- [x] 添加意图分类器：根据输入关键词/模式判断意图类型（查询/分析/探索/报告）
- [x] 合并 REPL 帮助到单一入口：`help` 显示分类，不再列出所有命令
- [x] 去除 `discover` 命令，改为默认行为：无法匹配任何已知命令时，走关键词搜索 + 反馈

## 阶段二：关系替代图

situation app 的图导航能力用 gallery 的 `situation-relation` 数据重建，不用 petgraph。

- [ ] `KeywordIndex::search()` 返回匹配后，自动加载该情境的 `SituationRelation`（source/target）
- [ ] 添加图导航函数：`neighbors(name) -> Vec<String>`、`bfs(start, depth)`
- [ ] 搜索结果显示匹配情境 + 邻居 + 关系类型

## 阶段三：LLM 增强

situation app 的 LLM prompt 模板适配到 gallery 数据，非 LLM 替代。

- [ ] 当关键词匹配 + 图导航不足以形成答案时，构建 prompt 调用 `quanttide-agent`
- [ ] prompt 模板：匹配的情境 + 关系 + 意图 → LLM 生成综合分析
- [ ] 母题发现：跨情境的模式识别 prompt 模板

## 阶段四：完全废弃 situation app

- [ ] 从 workspace 中移除 `apps/situation/`（保留代码，不编译）
- [ ] 资产文件 `assets/situation-graph.json` 归档或删除
- [ ] 在 AGENTS.md 中记录废弃说明
