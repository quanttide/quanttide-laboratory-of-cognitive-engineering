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

## 清理完成

### 已删除文件
- [x] `loader.rs` - 数据加载器
- [x] `query.rs` - 查询引擎
- [x] `report.rs` - 报告生成器
- [x] `templates/weekly_report.md` - 模板文件
- [x] `samples/2026-W23.md` - 示例文件
- [x] `reports/2026-W23/report.md` - 报告输出

### 已更新文件
- [x] `lib.rs` - 移除 `loader`、`query`、`report` 模块声明
- [x] `main.rs` - 移除旧配置，简化为基本 REPL 启动
- [x] `repl.rs` - 移除对 `QueryEngine` 和 `ReportGenerator` 的引用
- [x] `Cargo.toml` - 移除 `quanttide-think`、`quanttide-agent`、`serde`、`serde_yaml`、`chrono`、`serde_json` 依赖
- [x] `README.md` - 更新为清理后的状态说明
- [x] `TODO.md` - 创建清理清单

## 清理结果

- **删除文件数**: 6 个
- **删除代码行数**: 2969 行
- **新增代码行数**: 127 行
- **提交**: `83ad6ff` - "chore: 清理已迁移到正式代码的实验功能"

## 相关文档

- [正式代码文档](../../../../apps/qtcloud-think/src/cli/README.md)
