# 映射类型验证 + 验证协议模板

## 1.2 映射类型 vs CTA 分解一致性验证

验证对象：旧实验中定义的 4 种映射类型。
参考理论：`library/theories/cognitive-task-analysis.md`（GOMS / SRK / CDM）。

### 类型一：结构同构（Structural Isomorphism）

**定义**：将同一结构模式映射到不同领域（如单租户→多租户的升级阶段模型直接迁移到外部咨询场景）。

**CTA 对照**：GOMS 的 Methods 概念——方法是一组可跨上下文复用的步骤序列。专家能识别不同领域中的同构结构（CDM 的关键发现）。

**判定**：✅ 一致。CTA 支持"结构可跨上下文复用"的假设。

### 类型二：隐喻迁移（Metaphor Transfer）

**定义**：用一个领域的认知框架来理解另一个领域的类似问题（如"升级像做手术"——诊断→方案→实施→恢复）。

**CTA 对照**：SRK 的 Knowledge-based behavior——面对新情境时，人使用类比推理调用已有的知识结构。认知任务图的"意图-行动链"也依赖隐喻连接。

**判定**：✅ 一致。CTA 将类比推理视为知识层行为的核心机制。

### 类型三：反例学习（Counterexample Learning）

**定义**：从认知偏差和错误案例中提取"不要做什么"的规则（如 biases 字段）。

**CTA 对照**：CTA 的错误分析——追溯认知错误路径以设计防错机制。CDM 的"关键决策点"也关注失败决策的认知根源。

**判定**：✅ 一致。CTA 明确将错误分析纳入方法体系。

### 类型四：约束标记（Constraint Marking）

**定义**：显式标记 schema 的适用条件和排除条件（如 boundaries 字段）。

**CTA 对照**：GOMS 的 Selection rules——在不同方法之间选择时，判定条件本质上就是约束标记。SRK 的 Rule-based behavior 同样依赖 if-condition-then-action 的约束结构。

**判定**：✅ 一致。CTA 的规则选择机制与约束标记同构。

### 验证结论

4 种映射类型全部与 CTA 的认知任务分解方法一致。**无需修改分类体系，但应补充 CTA 引用到各类型的定义中。**

---

## 1.3 准入期验证协议模板

适用于"需验证类"因果链——在 schema 迁移的准入期（前 1-2 周），通过结构化验证判断该因果链是否适用于目标客户。

### 协议结构

```yaml
verification_protocol:
  id: vp-{uuid}
  causal: "{condition} → {outcome}"
  type: verifiable
  
  # 假设描述
  hypothesis: >
    这个因果链成立的隐含前提是什么？
    （如"客户CTO会亲自参与认知对齐工作坊"）
  
  # 验证方法
  method:
    type: interview | observation | document_review | survey
    detail: 具体怎么验证
    duration: 验证周期（天）
  
  # 通过/拒绝阈值
  pass_criteria:
    - 标准一
    - 标准二
  fail_criteria:
    - 什么情况下判定不通过
  
  # 验证记录
  result:
    status: pending | passed | failed | adjusted
    evidence: 验证证据引用
    adjust_note: 如果 adjusted，修改后的因果链是什么
```

### 示例

```yaml
verification_protocol:
  id: vp-a1b2c3d4
  causal: "顾问处于信息孤岛 → 无法感知客户政治博弈，升级节奏错位"
  type: verifiable
  hypothesis: >
    这个因果链成立的前提是"顾问对客户组织动态的感知通道有限"。
    如果客户建立了透明的汇报机制（每日站会 + 决策记录公开），
    顾问可能不会处于信息孤岛。
  method:
    type: interview
    detail: >
      在准入期（第一周）与客户 PM 和 技术负责人分别进行 30 分钟访谈，
      了解：
      1. 日常沟通渠道是什么
      2. 哪些决策信息会共享给外部顾问
      3. 是否有信息保密的场景
    duration: 5
  pass_criteria:
    - "客户确认所有与升级相关的决策信息对顾问开放"
    - "建立了每日 15 分钟站会机制"
  fail_criteria:
    - "客户明确表示某些决策信息不能共享"
    - "客户拒绝建立定期沟通机制"
  result:
    status: pending
    evidence: null
```

### 通用协议模板

```yaml
verification_protocol:
  id: vp-{uuid}
  causal: "{condition} → {outcome}"
  type: verifiable
  hypothesis: ""
  method:
    type: interview | observation | document_review | survey
    detail: ""
    duration: null
  pass_criteria: []
  fail_criteria: []
  result:
    status: pending
    evidence: null
    adjust_note: null
```
