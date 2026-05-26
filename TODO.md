# TODO (CLI)

## P0 — 核心流程

- [x] `thinkcloud think <text>` — 提交念头 + AI 处理
- [x] `thinkcloud process` — 手动触发 AI 处理
- [x] `thinkcloud ideas` — 列出想法
- [x] `thinkcloud accept/reject <id>` — 接受/拒绝想法
- [x] 念头模板提示词（config 可配，`templates` 命令查看）

## P1 — 会话管理

- [x] `thinkcloud session new/list/switch` — 会话生命周期
- [x] `thinkcloud status` — 当前会话概览
- [x] `thinkcloud export` — 导出 JSON

## P2 — 基础设施

- [x] `DEEPSEEK_API_KEY` 环境变量
- [x] `tracing` 日志
- [x] token 预算控制

## 测试覆盖率

- 40 个测试全部通过，0 失败
- 覆盖模块：config（默认值/序列化/TOML解析）、db（CRUD/关联/上下文构建/串行队列）、ai（prompt构建/截断）、main（CLI 命令路由/会话切换/想法操作/导出）
