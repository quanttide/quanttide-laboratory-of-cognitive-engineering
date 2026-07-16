//! 报告输出

use crate::detect::{Mode, RuleResult};

/// 输出检测报告
pub fn print(results: &[RuleResult], mode: Mode) {
    let total: f64 = results.iter().map(|r| r.score).sum();
    let max: f64 = results.iter().map(|r| r.max_score).sum();
    let pct = total / max * 100.0;
    let level = if pct >= 80.0 { "低" } else if pct >= 60.0 { "中" } else { "高" };
    match mode {
        Mode::Summary => println!("{}% ({})", pct as u32, level),
        Mode::Normal => {
            println!("{}% ({})", pct as u32, level);
            for r in results { let p = r.score / r.max_score * 100.0; if p < 80.0 { println!("  {}:{:.0}%", r.name, p); } }
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
