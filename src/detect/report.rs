//! 报告输出

use crate::detect::{Mode, RuleResult};

/// 评分校准：将原始均值映射到用户感知的质量评分
///
/// 原始评分是各规则的简单平均，但"未检测到问题"的规则默认给 100 分，
/// 导致整体虚高。校准曲线将 [0,100] 映射到用户感知区间：
///
///   raw 100% → 80（认知清晰度极高）
///   raw 70%  → 57（良好）
///   raw 40%  → 35（中等偏下）
///   raw 0%   → 5 （几乎不可读）
fn calibrate(raw: f64) -> f64 {
    5.0 + raw * 0.75
}

/// 输出检测报告
pub fn print(results: &[RuleResult], mode: Mode) {
    let total: f64 = results.iter().map(|r| r.score).sum();
    let max: f64 = results.iter().map(|r| r.max_score).sum();
    let raw = total / max * 100.0;
    let pct = calibrate(raw);
    let level = if pct >= 65.0 { "优" } else if pct >= 55.0 { "良" } else if pct >= 45.0 { "中" } else { "差" };
    match mode {
        Mode::Summary => println!("{}% ({})", pct as u32, level),
        Mode::Normal => {
            println!("{}% ({})", pct as u32, level);
            for r in results {
                let p = r.score / r.max_score * 100.0;
                if p < 80.0 { println!("  {}:{:.0}%", r.name, p); }
            }
        }
        Mode::Verbose => {
            println!("{}% ({})", pct as u32, level);
            for r in results {
                let p = r.score / r.max_score * 100.0;
                if p < 80.0 { println!("  {}:{:.0}%", r.name, p); for d in &r.details { println!("    {}", d); } }
            }
        }
    }
}
