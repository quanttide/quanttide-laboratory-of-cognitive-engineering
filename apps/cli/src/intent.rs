/// 意图分类：根据输入判断用户想做什么

#[derive(Debug, PartialEq)]
pub enum Intent {
    /// 纯粹的关键词探索，无明确命令匹配
    Explore(String),
    /// 时间/演化相关（演化、趋势、变化）
    Evolution(String),
    /// 对比分析（对比、diff、vs）
    Compare(String),
    /// 关系分析（关系、关联、冲突）
    Relate(String),
}

/// 根据输入文本判断意图
pub fn classify(input: &str) -> Intent {
    let lower = input.to_lowercase();

    // 演化关键词
    let evolve_words = ["演化", "演变", "趋势", "变化", "发展", "进化", "evolve", "evolution"];
    if evolve_words.iter().any(|w| lower.contains(w)) {
        // 提取情境名：输入中可能包含情境名称
        return Intent::Evolution(input.to_string());
    }

    // 对比关键词
    let compare_words = ["对比", "不同", "差异", "diff", "vs", "versus", "比较"];
    if compare_words.iter().any(|w| lower.contains(w)) {
        return Intent::Compare(input.to_string());
    }

    // 关系关键词
    let relate_words = ["关系", "关联", "冲突", "支持", "触发", "relate", "relation", "tension"];
    if relate_words.iter().any(|w| lower.contains(w)) {
        return Intent::Relate(input.to_string());
    }

    // 默认：探索
    Intent::Explore(input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_explore() {
        assert_eq!(classify("组织管理"), Intent::Explore("组织管理".into()));
        assert_eq!(classify("基础设施"), Intent::Explore("基础设施".into()));
    }

    #[test]
    fn test_classify_evolution() {
        let r = classify("认知工程的演化趋势");
        assert_eq!(r, Intent::Evolution("认知工程的演化趋势".into()));
    }

    #[test]
    fn test_classify_compare() {
        let r = classify("对比 W22 和 W23");
        assert_eq!(r, Intent::Compare("对比 W22 和 W23".into()));
    }

    #[test]
    fn test_classify_relate() {
        let r = classify("组织和基础设施的关系");
        assert_eq!(r, Intent::Relate("组织和基础设施的关系".into()));
    }
}
