好的，基于我们刚才的讨论，下面是对 **“思考云 TUI”** 设计思路的完整总结。

---

## 一、核心认知模型（一次思考的闭环）

1. **材料（可选）**：外部信息载体，如日志文件、文章、原始文档。  
2. **念头**：用户输入的极短文本（通常一句话），由材料或当前情境触发，是思考的原始驱动。  
3. **AI 计算**：后台异步处理，结合当前材料、历史念头流、以及隐含的情境与心智模型（这些模型是从想法流中逐渐发现的）。  
4. **想法**：AI 生成的总结性结论或洞察，用户可接受（结束本轮思考）或拒绝（继续输入新念头，让 AI 重新生成）。  
5. **情境 & 心智模型**（长期愿景，MVP 暂不涉及）：不要求用户显式定义，而是在想法和念头积累过程中被自动发现和归纳，作为元信息服务于后续思考。
   MVP 阶段的最小替代方案：**会话（Session）** 作为情境的轻量承载单元，用户可手动创建和切换不同主题的会话。

**一句话概括：**  
材料 →（念头 + AI 计算）→ 想法 → 用户确认 → 一轮思考结束。

---

## 二、产品形态：Rust TUI（终端用户界面）

选择 **Rust + TUI** 的核心理由：

- **念头输入极快**：一条命令或一个输入框即可记录，符合短文本特性。
- **轻量专注**：无 GUI 干扰，界面只需显示念头流、当前想法、输入框。
- **性能优秀**：二进制启动毫秒级，占用内存极小。
- **跨平台**：任何有终端的系统（Linux/macOS/Windows WSL）都能运行。
- **异步 AI 不阻塞**：用户输入念头后立即返回，AI 在后台处理，完成后刷新右侧想法区域。
- **本地优先**：数据存在本地 SQLite，可自包含，未来再扩展同步。

---

## 三、技术栈（推荐）

| 类别          | 库                                          | 用途                     |
|---------------|---------------------------------------------|--------------------------|
| TUI 框架      | `ratatui`                                   | 界面布局、事件处理         |
| 终端后端      | `crossterm`                                 | 跨平台终端控制             |
| 异步运行时    | `tokio`                                     | 非阻塞 AI 调用            |
| AI 接口       | `reqwest` + `serde_json`                    | 调用 OpenAI / Ollama API  |
| 本地存储      | `rusqlite`                                  | 轻量 SQLite 数据库        |
| 配置路径      | `dirs`                                      | 获取用户配置目录           |
| 配置文件     | `toml` / `serde`                             | 加载 AI 等用户配置         |
| 错误处理      | `anyhow` / `thiserror`                      | 简化错误传播              |
| 测试快照     | `insta`（可选）                              | TUI 渲染结果快照测试        |

---

## 四、数据模型（SQLite 表）

```sql
-- 会话表（情境的最小承载单元，不同主题的思考隔离）
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    title TEXT,                -- 会话标题，自动生成或用户命名
    material_id INTEGER,       -- 当前关联的材料
    created_at TEXT,
    updated_at TEXT,
    FOREIGN KEY (material_id) REFERENCES materials(id)
);

-- 材料表（可选，仅引用本地文件或文本片段）
CREATE TABLE materials (
    id INTEGER PRIMARY KEY,
    path TEXT,                 -- 文件路径或自定义标识
    content_snippet TEXT,      -- 摘要或前几行
    created_at TEXT
);

-- 念头表（用户每次输入的短文本）
CREATE TABLE thoughts (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL,   -- 所属会话
    material_id INTEGER,           -- 可选关联材料
    content TEXT,                  -- 念头内容
    status TEXT DEFAULT 'pending', -- pending | processing | completed | failed
    created_at TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (material_id) REFERENCES materials(id)
);

-- 想法表（AI 生成的结论，用户确认后保留）
CREATE TABLE ideas (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL,   -- 所属会话
    content TEXT,                  -- AI 生成的文本
    status TEXT DEFAULT 'pending', -- pending | accepted | rejected
    created_at TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- 想法与念头的关联表（多对多，替代原先的 JSON 字段）
CREATE TABLE idea_thoughts (
    idea_id INTEGER NOT NULL,
    thought_id INTEGER NOT NULL,
    PRIMARY KEY (idea_id, thought_id),
    FOREIGN KEY (idea_id) REFERENCES ideas(id),
    FOREIGN KEY (thought_id) REFERENCES thoughts(id)
);
```

- 所有数据默认保存在用户数据目录，如 `~/.local/share/thinkcloud/`。
- 用户配置文件位于 `~/.config/thinkcloud/config.toml`（见下方配置说明）。

### 配置文件（`config.toml`）

```toml
[ai]
provider = "openai"       # 或 "ollama"
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
# api_key 优先从环境变量 THINKCLOUD_API_KEY 读取，避免明文写入文件

[storage]
data_dir = "~/.local/share/thinkcloud/"

[ui]
thought_window = 10        # AI 调用时携带的最近念头数
```

---

## 五、TUI 界面布局（极简两栏）

```
┌────────────────────────────────────────┐
│ 思考云 - 材料: bug_report.txt (按 :m 换)│
├─────────────────────┬──────────────────┤
│ 念头流 (最近10条)     │ 最新想法           │
│ > 复现步骤缺少环境变量 │ 想法 #12：        │
│ > 会不会是时区问题？   │ 问题根因可能是...  │
│ > 查日志中的时间戳     │                  │
│                     │ [y] 接受 [n] 拒绝  │
├─────────────────────┴──────────────────┤
│ 输入新念头: _                            │
└────────────────────────────────────────┘
```

- **左侧**：按时间倒序显示念头，可滚动。  
- **右侧**：展示 AI 最近生成的**想法**，并提示接受/拒绝快捷键。  
- **底部**：固定输入框，用户输入后回车即存入并触发后台 AI。  
- **命令模式**：支持 `:material <路径>` 加载材料，`:quit` 退出等。

---

## 六、用户交互流程（MVP）

1. **启动 TUI**  
   - 自动加载最近的 10 条念头（若无则空白）。  
   - 默认无材料，用户可随时通过 `:material` 加载。

2. **输入念头**
   - 在底部输入框键入短文本，按回车。
   - 念头立即存入数据库并显示在左侧列表顶部。
   - 后台异步调用 AI。UI 不阻塞，用户可以继续输入下一个念头。
   - **AI 上下文策略**（滑动窗口）：
     - 始终携带当前会话的材料摘要（而非全文）。
     - 按时间倒序取最近 N 条念头（默认 10 条，由 `config.toml` 的 `thought_window` 控制）。
     - 若超出窗口，首条补一句 `……（之前还有 M 条）` 摘要说明。
   - **错误处理**：若 AI 调用失败（网络、认证、超时），该念头标记为 `failed` 状态，右侧显示红色错误提示及重试快捷键（`r`）。

3. **AI 返回想法**
   - AI 完成后，右侧面板更新为新的想法文本，并显示"新想法！"标记。
   - 用户按 `y` 接受该想法 → 标记 `status='accepted'`，本轮思考可结束（或继续新念头）。
   - 用户按 `n` 拒绝 → 标记 `status='rejected'`，清空右侧面板，等待用户继续输入新念头后重新触发 AI。
   - **想法与念头关联**：当前想法通过 `idea_thoughts` 表关联参与生成的所有念头 ID，支持追溯生成依据。

4. **一轮思考结束**  
   - 用户接受想法后，可根据需要清空当前会话或保留念头流作为历史。  
   - 可开始新的材料/念头序列。

---

## 七、设计原则

- **极简主义**：只做 TUI，不提前引入 Web 服务端或移动端。  
- **会话隔离**：每个会话独立管理念头和想法，切换主题只需 `:session new` / `:session switch`。  
- **可配置性**：AI 端点、模型、窗口大小等通过 `~/.config/thinkcloud/config.toml` 配置，API key 优先从环境变量读取。
- **念头为核心**：用户唯一的高频操作就是输入短文本。  
- **AI 作为后台助手**：不干扰输入流，结果实时刷新。  
- **本地优先**：数据归属用户，格式开放，便于导出。  
- **渐进发现**：情境和心智模型不是预先定义的，而是从想法和念头中自动挖掘，初期版本可不实现，先保证闭环可用。

---

## 八、未来可扩展点（暂不实现）

- 心智模型自动发现（从会话积累的历史中归纳用户思维模式）
- 云端同步（可选）  
- 本地 LLM 集成（通过 Ollama 或 `candle`）  
- 想法图谱可视化（导出到浏览器查看）

---

这就是目前形成的“思考云 TUI”完整设计思路。核心一句话：**用 Rust 写一个极速的终端工具，让用户只管丢念头，AI 在背后产出想法，人按 y 确认，一轮思考丝滑结束。**
