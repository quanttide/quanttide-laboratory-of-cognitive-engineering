# TODO: 清理已迁移到正式代码的实验功能

以下功能已迁移到正式代码 `apps/qtcloud-think/src/cli`，需要从本实验代码中清理。

## 已迁移功能清单

### 1. 数据访问层 (`loader.rs` → `repo.rs`)

| 实验代码功能 | 正式代码实现 | 状态 |
|-------------|-------------|------|
| `GalleryLoader::load_registry()` | `Repo::worlds()` | ✅ 已迁移 |
| `GalleryLoader::list_weeks()` | `Repo::periods(world)` | ✅ 已迁移 |
| `GalleryLoader::load_week(week)` | `Repo::domains(world, period)` | ✅ 已迁移 |
| `GalleryLoader::load_situations(week)` | `Repo::load(world, period, domain)` | ✅ 已迁移 |
| `GalleryLoader::load_intentions(week)` | `Repo::load(world, period, domain)` | ✅ 已迁移 |
| `GalleryLoader::load_schemas(week)` | `Repo::load(world, period, domain)` | ✅ 已迁移 |
| `GalleryLoader::load_situation_relations(week)` | `Repo::load(world, period, domain)` | ✅ 已迁移 |

### 2. 数据模型 (`model.rs`)

| 实验代码类型 | 正式代码类型 | 状态 |
|-------------|-------------|------|
| 分散的 `Situation`、`Intention`、`Schema` | `DomainFile` 统一结构 | ✅ 已迁移 |
| `QueryEngine::WeekData` | `Snapshot` + `IntentionEntry` | ✅ 已迁移 |

### 3. 配置管理 (`main.rs` → `config.rs`)

| 实验代码配置 | 正式代码配置 | 状态 |
|-------------|-------------|------|
| `GALLERY_PATH` 环境变量 | `JOURNAL_PATH` 环境变量 | ✅ 已迁移 |
| 命令行参数 `--gallery` | `Config::from_env()` | ✅ 已迁移 |

### 4. 分析功能 (`query.rs` → `analyze.rs`)

| 实验代码功能 | 正式代码实现 | 状态 |
|-------------|-------------|------|
| `QueryEngine::week(week)` | `Repo::load()` + `DomainFile` | ✅ 已迁移 |
| `QueryEngine::situation(name)` | `analyze::track_evolution()` | ✅ 已迁移 |
| `QueryEngine::list_weeks()` | `Repo::periods()` | ✅ 已迁移 |
| `QueryEngine::registry()` | `Repo::worlds()` | ✅ 已迁移 |
| 无 | `Repo::describe()` | 🆕 新增功能 |

## 待清理代码

### `loader.rs` - 数据加载器
- [ ] 删除 `GalleryLoader` 结构体
- [ ] 删除所有 `load_*` 方法
- [ ] 删除 `list_weeks()` 方法

### `query.rs` - 查询引擎
- [ ] 删除 `QueryEngine` 结构体
- [ ] 删除 `WeekData` 类型定义
- [ ] 删除 `week()`、`situation()`、`list_weeks()`、`registry()` 方法

### `model.rs` - 数据模型
- [ ] 删除与 `DomainFile`、`Snapshot`、`IntentionEntry` 重复的类型定义

### `main.rs` - 入口文件
- [ ] 删除 `--gallery` 参数解析
- [ ] 删除 `GALLERY_PATH` 环境变量读取
- [ ] 删除 `QueryEngine` 初始化代码

### `lib.rs` - 库根模块
- [ ] 移除 `loader` 模块声明（如果完全删除）
- [ ] 移除 `query` 模块声明（如果完全删除）

## 保留功能（未迁移）

以下功能尚未迁移到正式代码，需要保留：

### `report.rs` - 报告生成器
- [ ] 保留 `summary()`、`landscape()`、`evolution()`、`diff()`、`report()` 方法
- [ ] 保留 `list_intentions()`、`show_intention()`、`filter_intentions()`、`trace()`、`drift()`、`evolution_table()` 方法
- [ ] 保留 `relate_llm()`、`llm_gallery_report()` LLM 推理功能
- [ ] 保留 `compute_relations()`、`render_gallery_report()`、`data_gallery_report()` 辅助函数

### `repl.rs` - 交互式命令行
- [ ] 保留所有 REPL 命令实现

### `templates/` - 模板文件
- [ ] 保留 `weekly_report.md` 模板

### `samples/` - 示例文件
- [ ] 保留 `2026-W23.md` 示例

## 清理顺序建议

1. 先更新 `lib.rs`，移除已迁移模块的导出
2. 删除 `loader.rs` 和 `query.rs` 文件
3. 清理 `model.rs` 中重复的类型定义
4. 更新 `main.rs`，移除已迁移的配置和初始化代码
5. 更新 `repl.rs`，移除对已迁移模块的引用
6. 更新 `report.rs`，使用正式代码的 `repo` 模块替代 `loader`
7. 运行测试确保功能正常
