# 项目二：Thought 驱动的叙事生成

用 Project 1 产出的 13 个 Thought 作为叙事单元，生成一篇结构化的周叙事总结——直接验证 Intent 能否作为叙事工程的基本单元。

**输入来源**：`../../assets/refined/`（thoughts.md + intent-relations.md）+ `../../assets/raw/`

## 任务

### 2.1 Thought 编排

**输入**：`../../assets/refined/thoughts.md`（13 个 Thought）+ `../../assets/refined/intent-relations.md`（关系图）

**方法**：
1. 以 3 个元意图（Thought 7 意图工程 / Thought 8 POC / Thought 13 心理调节）为叙事主线
2. 其余 10 个 Thought 按关系图中的支持/依赖关系挂载到主线上
3. 确定叙事顺序：从基础层（Thought 13 心理 / Thought 3 基础设施）出发 → 经过方法层（Thought 7+8）→ 抵达应用层（Thought 5 写作云 / Thought 11 商业模式等）
4. 冲突关系（Thought 1↔2, Thought 7↔8）作为叙事张力点

**输出**：叙事大纲

### 2.2 叙事写作

**输入**：2.1 的叙事大纲 + `../../assets/raw/` 对应日期的关键段落

**方法**：
1. 按大纲顺序，以每个 Thought 为一个叙事节（section），展开为连贯的叙事段落
2. 每个叙事节需引用原始日志中的关键段落作为证据
3. 冲突点展开为"困境→思考→解决线索"的三段式
4. 最终收束到元意图（意图工程）——点明一周思考如何汇聚到这个核心发现

**输出**：`../../assets/refined/weekly-narrative.md`，一篇完整的周叙事总结
