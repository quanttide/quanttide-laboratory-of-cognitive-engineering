# 项目三：方案A — 文本匹配意图识别

基于关键词匹配的纯文本意图识别，不使用知识图谱。

## 核心问题

仅靠关键词重叠能否可靠识别一段日志文本所涉及的核心关切？

## 技术栈

- Rust + serde + serde_json（JSON 序列化）
- 单 binary：`approach_a`

## 任务

### 3.1 构建关键词表

从 `intent.yaml` 提取 10 个 Intent 簇的信息，构建簇→关键词映射表。

**输入**：`../../../assets/refined/intent.yaml`

**方法**：
1. 解析 YAML，提取每个节点的 `id`、`name`、`per_week` 中的 intent 表述
2. 对每个簇，将其名称和 per_week 中的表述全部拆分为单字/双字词，去重后作为该簇的关键词表
3. 序列化为 JSON

**输出**：`data/keywords.json`

```json
{
  "clusters": [
    {
      "id": 1,
      "name": "研发方法论",
      "keywords": ["研发", "方法", "迭代", "效率", "流程", "代码", ...]
    }
  ]
}
```

### 3.2 匹配算法

**输入**：`data/keywords.json` + stdin 文本

**算法**：
1. 对测试文本做分词（按标点和空格切分，保留双字及以上词）
2. 对每个 Intent 簇，计算匹配度：matches / cluster_total_keywords
3. 匹配度 > 阈值（默认 0.1）则判定为该簇
4. 记录匹配到的关键词证据（原文中出现的具体词）

**输出**：stdout JSON

```json
{
  "text": "输入文本片段",
  "matched_clusters": [
    {
      "id": 1,
      "name": "研发方法论",
      "score": 0.15,
      "evidence": ["研发", "效率", "迭代"]
    }
  ],
  "unmatched_clusters": [2, 3, 4, 5, 6, 7, 8, 9, 10]
}
```

### 3.3 执行

```bash
echo "测试文本..." | cargo run --bin approach_a
```

**源码**：`src/main.rs`
