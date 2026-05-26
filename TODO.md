# TODO

## P0 — 必须完成

- [x] AI 上下文策略补充：已接受的 ideas 作为可选上下文加入 prompt
- [x] 定义并发 AI 调用策略：串行队列（每 session 一个 pending flag），禁止并行
- [x] 修正错误状态模型：`thoughts.status` 仅表示念头本身状态，`ideas.status` 增加 `failed`
- [x] 空状态处理：首次启动无念头、无材料、无想法时的界面展示

## P1 — 重要

- [x] 数据模型：`sessions.material_id` 改为 `session_materials` 关联表（一对多）
- [x] 数据模型：`thoughts` 和 `ideas` 增加 `sort_order INTEGER` 排序字段
- [x] 统一快捷键文档：`y`/`n`/`r`/`:m`/`:material` 在界面和交互流程中一致列出
- [x] 定义键盘导航：tab 切换面板、方向键滚动念头列表

## P2 — 增强

- [x] 集成 `tracing` 日志，方便诊断 AI 调用失败原因
- [x] 定义 token 预算策略：窗口超限时控制输入长度
- [x] MVP 即支持 `:export json` 数据导出
- [x] 确认环境变量命名规范：`THINKCLOUD_API_KEY`

## 测试覆盖率

- 49 个测试全部通过，0 失败
- 覆盖模块：models（状态枚举序列化/反序列化）、config（默认值/序列化/TOML解析）、db（CRUD/关联/上下文构建/串行队列）、ai（prompt构建/截断）、ui（空状态/错误/处理中/渲染）、main（App状态机/会话切换/命令处理/导出）
