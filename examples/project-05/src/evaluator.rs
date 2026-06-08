use std::collections::HashSet;
use std::process::Command;

use crate::models::*;

pub fn run_binary(
    bin_path: &str,
    args: &[&str],
    input_text: &str,
) -> Result<serde_json::Value, String> {
    let output = Command::new(bin_path)
        .args(args)
        .arg(input_text)
        .output()
        .map_err(|e| format!("failed to run {}: {}", bin_path, e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("exit code {:?}: {}", output.status.code(), stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| format!("invalid JSON: {}", e))
}

pub fn run_binary_stdin(
    bin_path: &str,
    args: &[String],
    input_text: &str,
) -> Result<serde_json::Value, String> {
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut cmd = Command::new(bin_path);
    cmd.args(&args_refs);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());

    let mut child = cmd.spawn().map_err(|e| format!("spawn failed: {}", e))?;
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input_text.as_bytes())
        .map_err(|e| format!("stdin write failed: {}", e))?;

    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait failed: {}", e))?;
    if !output.status.success() {
        return Err(format!("exit code: {:?}", output.status.code()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| format!("invalid JSON: {}", e))
}

pub fn get_matched_a(value: &serde_json::Value) -> Vec<u32> {
    let default = serde_json::Value::Null;
    let matched = value.get("matched").unwrap_or(&default);
    if !matched.is_array() {
        return vec![];
    }
    matched
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m.get("id").and_then(|id| id.as_u64()))
        .map(|id| id as u32)
        .collect()
}

pub fn get_matched_b(value: &serde_json::Value) -> Vec<u32> {
    let default = serde_json::Value::Null;
    let nodes = value.get("match_nodes").unwrap_or(&default);
    if !nodes.is_array() {
        return vec![];
    }
    nodes
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| n.get("id").and_then(|id| id.as_u64()))
        .map(|id| id as u32)
        .collect()
}

pub fn recall(matched: &[u32], expected: &[u32]) -> f64 {
    if expected.is_empty() {
        return 0.0;
    }
    let expected_set: HashSet<u32> = expected.iter().copied().collect();
    let tp = matched.iter().filter(|m| expected_set.contains(m)).count();
    tp as f64 / expected.len() as f64
}

pub fn false_positive_rate(matched: &[u32], expected: &[u32]) -> f64 {
    if matched.is_empty() {
        return 0.0;
    }
    let expected_set: HashSet<u32> = expected.iter().copied().collect();
    let fp = matched.iter().filter(|m| !expected_set.contains(m)).count();
    fp as f64 / matched.len() as f64
}

pub fn incremental_nodes(matched_a: &[u32], matched_b: &[u32], expected: &[u32]) -> usize {
    let expected_set: HashSet<u32> = expected.iter().copied().collect();
    let a_set: HashSet<u32> = matched_a.iter().copied().collect();
    matched_b
        .iter()
        .filter(|m| expected_set.contains(m) && !a_set.contains(m))
        .count()
}

pub fn incremental_relations(
    b_value: &serde_json::Value,
    expected: &[u32],
) -> usize {
    let expected_set: HashSet<u32> = expected.iter().copied().collect();
    let mut count = 0;

    if let Some(neighbors) = b_value.get("neighbors").and_then(|n| n.as_array()) {
        for n in neighbors {
            let from = n.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let to = n.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            if expected_set.contains(&from) || expected_set.contains(&to) {
                count += 1;
            }
        }
    }

    if let Some(paths) = b_value.get("bfs_paths").and_then(|p| p.as_array()) {
        for path in paths {
            if let Some(steps) = path.as_array() {
                for step in steps {
                    let from = step.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let to = step.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    if expected_set.contains(&from) || expected_set.contains(&to) {
                        count += 1;
                        break;
                    }
                }
            }
        }
    }

    count
}

pub fn parse_neighbors(value: &serde_json::Value) -> Vec<NeighborEntry> {
    let default = serde_json::Value::Null;
    let neighbors = value.get("neighbors").unwrap_or(&default);
    serde_json::from_value(neighbors.clone()).unwrap_or_default()
}

pub fn parse_bfs_paths(value: &serde_json::Value) -> Vec<Vec<PathStep>> {
    let default = serde_json::Value::Null;
    let paths = value.get("bfs_paths").unwrap_or(&default);
    serde_json::from_value(paths.clone()).unwrap_or_default()
}

pub fn parse_conflicts(value: &serde_json::Value) -> Vec<ConflictEntry> {
    let default = serde_json::Value::Null;
    let conflicts = value.get("conflicts").unwrap_or(&default);
    serde_json::from_value(conflicts.clone()).unwrap_or_default()
}

pub fn parse_candidates(value: &serde_json::Value) -> Vec<CandidateEntry> {
    let default = serde_json::Value::Null;
    let candidates = value.get("candidate_edges").unwrap_or(&default);
    serde_json::from_value(candidates.clone()).unwrap_or_default()
}

pub fn build_path_grades(b_value: &serde_json::Value) -> Vec<PathGrade> {
    let mut grades = Vec::new();
    if let Some(neighbors) = b_value.get("neighbors").and_then(|n| n.as_array()) {
        for n in neighbors {
            let from = n.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let to = n.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let rel = n
                .get("relation")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            grades.push(PathGrade {
                from,
                to,
                depth: 1,
                relation: rel,
                grade: None,
            });
        }
    }
    if let Some(paths) = b_value.get("bfs_paths").and_then(|p| p.as_array()) {
        for path in paths {
            if let Some(steps) = path.as_array() {
                let depth = steps.len();
                for step in steps {
                    let from = step.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let to = step.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let rel = step
                        .get("relation")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    grades.push(PathGrade {
                        from,
                        to,
                        depth,
                        relation: rel,
                        grade: None,
                    });
                }
            }
        }
    }
    grades
}
