# A/B Comparison Evaluation Report

**Generated**: 2026-06-08 17:35
**Total samples**: 5

> ⚠️ **Caveat**: Baseline annotations are best-effort based on W23 intent analysis, 
> not human-verified for these segments. Metrics should be interpreted as indicative, not absolute.

---

## T1

### Segment

> 我刚才在想，就是无论是代码编辑器还是文档编辑器，其实都不能够完全去适应当前 AI 的这种越来越多样和复杂的表达形式，及对它的交互的细节上的一些可能不是很适合，这这个就是我感觉我们可能需要或者说理论上存在一种新的交互方式，就是它和现在的这种交互方式会表面上差不多，可能还是三栏，但是在使用上可能会有一些本质区别。

### Cluster Match Comparison

| Scheme | Matched Clusters | Recall | False Positive |
|--------|-----------------|--------|---------------|
| **Baseline** | [4] | — | — |
| **Scheme A** (text match) | [4] | 100.00% | 0.00% |
| **Scheme B** (graph inference) | [4] | 100.00% | 0.00% |

### Scheme B Additional Discoveries

- **Incremental nodes**: 0
- **Incremental relations**: 5
- **L1 neighbors**: 2
- **L2 BFS paths**: 3
- **L3 conflicts**: 0
- **L4 candidate edges**: 0

| From | To | Relation | Direction |
|------|----|----------|-----------|
| 4 | 3 | 支持 | outgoing |
| 3 | 4 | 支持 | incoming |

#### BFS Paths

1. 4→3(支持)
2. 4→3(支持) → 3→4(支持)
3. 4→3(支持) → 3→1(支持)

### Path Quality Review

| From | To | Depth | Relation | Grade | Note |
|------|----|-------|----------|-------|------|
| 4 | 3 | 1 | 支持 | — | |
| 3 | 4 | 1 | 支持 | — | |
| 4 | 3 | 1 | 支持 | — | |
| 4 | 3 | 2 | 支持 | — | |
| 3 | 4 | 2 | 支持 | — | |
| 4 | 3 | 2 | 支持 | — | |
| 3 | 1 | 2 | 支持 | — | |

---

## T2

### Segment

> 所有人类意图驱动的结构化生成过程，包括产品设计、架构设计等。这确实是我的直觉，被 AI 给讲清楚了。确实是抱着这个潜意识探究的。智谱提出了一个叫做意图工程的新概念，说我实际上发现了这个雏形。我确实有在考虑对意图建模，但是没想到居然可成为一门工程。确实很意外。

### Cluster Match Comparison

| Scheme | Matched Clusters | Recall | False Positive |
|--------|-----------------|--------|---------------|
| **Baseline** | [3] | — | — |
| **Scheme A** (text match) | [4] | 0.00% | 100.00% |
| **Scheme B** (graph inference) | [4] | 0.00% | 100.00% |

### Scheme B Additional Discoveries

- **Incremental nodes**: 0
- **Incremental relations**: 5
- **L1 neighbors**: 2
- **L2 BFS paths**: 3
- **L3 conflicts**: 0
- **L4 candidate edges**: 0

| From | To | Relation | Direction |
|------|----|----------|-----------|
| 4 | 3 | 支持 | outgoing |
| 3 | 4 | 支持 | incoming |

#### BFS Paths

1. 4→3(支持)
2. 4→3(支持) → 3→4(支持)
3. 4→3(支持) → 3→1(支持)

### Path Quality Review

| From | To | Depth | Relation | Grade | Note |
|------|----|-------|----------|-------|------|
| 4 | 3 | 1 | 支持 | — | |
| 3 | 4 | 1 | 支持 | — | |
| 4 | 3 | 1 | 支持 | — | |
| 4 | 3 | 2 | 支持 | — | |
| 3 | 4 | 2 | 支持 | — | |
| 4 | 3 | 2 | 支持 | — | |
| 3 | 1 | 2 | 支持 | — | |

---

## T3

### Segment

> 财务突然离职，对我的心理造成了一些冲击。这加剧了我的不安感。我觉得我们得预防人突然跑路。这种时刻其实就是团队的反脆弱实验，就是说我们能不能维持住团队基本盘。

### Cluster Match Comparison

| Scheme | Matched Clusters | Recall | False Positive |
|--------|-----------------|--------|---------------|
| **Baseline** | [2, 8] | — | — |
| **Scheme A** (text match) | [] | 0.00% | 0.00% |
| **Scheme B** (graph inference) | [] | 0.00% | 0.00% |

### Scheme B Additional Discoveries

- **Incremental nodes**: 0
- **Incremental relations**: 0
- **L1 neighbors**: 0
- **L2 BFS paths**: 0
- **L3 conflicts**: 0
- **L4 candidate edges**: 0

---

## T4

### Segment

> AI 在进行相互辩论的时候，会有一个特点，就是说如果没有人类干预的话，它就只能在一种状态之下收敛。然后人类的反馈丢进去过之后，会让 AI 跳出现有的框架。就我觉得这是一个很大的发现，就是怎么去定义反思，就是这种自己跟自己辩论，然后加入人类的反思，人人机反思或者人机辩论。

### Cluster Match Comparison

| Scheme | Matched Clusters | Recall | False Positive |
|--------|-----------------|--------|---------------|
| **Baseline** | [3, 4] | — | — |
| **Scheme A** (text match) | [] | 0.00% | 0.00% |
| **Scheme B** (graph inference) | [] | 0.00% | 0.00% |

### Scheme B Additional Discoveries

- **Incremental nodes**: 0
- **Incremental relations**: 0
- **L1 neighbors**: 0
- **L2 BFS paths**: 0
- **L3 conflicts**: 0
- **L4 candidate edges**: 0

---

## T5

### Segment

> 这种低耗能模式之下，我的脑子里面只能去用一些现有的概念。如果我在这种状态下都能够维持产出的话，那么这个系统的自动化程度就非常高了，就是说我只要把想法吐出去就可以了，那么这样就可以极大地减少我发现新洞察的这种压力。

### Cluster Match Comparison

| Scheme | Matched Clusters | Recall | False Positive |
|--------|-----------------|--------|---------------|
| **Baseline** | [1, 2] | — | — |
| **Scheme A** (text match) | [1] | 50.00% | 0.00% |
| **Scheme B** (graph inference) | [1] | 50.00% | 0.00% |

### Scheme B Additional Discoveries

- **Incremental nodes**: 0
- **Incremental relations**: 3
- **L1 neighbors**: 3
- **L2 BFS paths**: 0
- **L3 conflicts**: 0
- **L4 candidate edges**: 0

| From | To | Relation | Direction |
|------|----|----------|-----------|
| 8 | 1 | 支持 | incoming |
| 3 | 1 | 支持 | incoming |
| 2 | 1 | 支持 | incoming |

### Path Quality Review

| From | To | Depth | Relation | Grade | Note |
|------|----|-------|----------|-------|------|
| 8 | 1 | 1 | 支持 | — | |
| 3 | 1 | 1 | 支持 | — | |
| 2 | 1 | 1 | 支持 | — | |

---

## Summary Metrics

| Metric | Value |
|--------|-------|
| Total samples | 5 |
| Avg recall (Scheme A) | 30.00% |
| Avg recall (Scheme B) | 30.00% |
| Total incremental nodes | 0 |
| Total incremental relations | 13 |
| Avg false positive (Scheme A) | 20.00% |
| Avg false positive (Scheme B) | 20.00% |
| Avg path grade | — |

### Notes

- **Limitation**: Test segments are from W23 (not W24), which was used for graph construction.
  This means both schemes may score higher on recall than on truly unseen data.
- **Baseline**: Generated from W23 intent analysis, not human-verified per segment.
- **Path grades**: To be filled in by human reviewer against original text.
