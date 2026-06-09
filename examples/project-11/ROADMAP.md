# Project 11: Situation Engine — ROADMAP

## 当前状态

`cargo build` 0 warning ✅ | `cargo run` REPL 可用 ✅

## 已完成

- [x] 数据模型：Situation / Intention / Registry / Relation / MentalModel / WeeklyReport
- [x] YAML 加载器：从 `docs/gallery/situation/` 和 `docs/gallery/intention/` 读取数据
- [x] 查询引擎：`week()` / `situation()` / `intentions()` / `list_weeks()`
- [x] 报告生成：`summary()` / `evolution()` / `landscape()`
- [x] REPL 命令：`weeks` / `show` / `landscape` / `explore` / `registry`

## 未完成

### Phase 1: 基础设施修补

- [ ] 路径硬编码 → 支持 `GALLERY_PATH` 环境变量或 `--gallery` 参数
- [ ] `intention→situation` 映射重构：文件名即情境名，不需要 O(n²) 扫 UUID
- [ ] `serde_json` 和 `uuid` 依赖清理：未使用，应移除
- [ ] 添加 `--help` 参数

### Phase 2: 核心功能补齐

- [ ] `report <week>` 命令：按六特征模板输出结构化周报
  - 核心判断 / 行动建议(含负责人+时限) / 逐情境分析(现象→原因→所以) / 关键关系(证据链) / 跨情境心智模型 / 与前周对比
- [ ] `relate <week>` 命令：基于情境内容让 LLM 推理情境间关系
  - 引入 llm crate（或直接调用 DeepSeek API）
  - 输出关系表：source → target → type → strength → logic

### Phase 3: LLM 推理集成

- [ ] 引入 `llm` crate 或重写轻量 HTTP 客户端
- [ ] 关系推理 prompt：输入全周所有情境的 agenda/ecology，输出情境对关系
- [ ] 心智模型推理 prompt：输入关系结果，识别跨情境反复出现的模式
- [ ] 周报自动生成：`report` 命令调用 LLM 填充判断/行动/心智模型

### Phase 4: 增强功能

- [ ] 跨周差异分析：`diff W22 W23` 自动输出对比表
- [ ] 心智模型追踪：`model <name>` 查看某心智模型在各周的 evidence
- [ ] 报告导出：`report <week> --format markdown` 输出到文件
