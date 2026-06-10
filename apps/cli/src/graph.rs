use std::collections::{HashMap, HashSet};
use quanttide_think::situation_relation::{SituationRelation, SituationRelationType};

pub struct RelationGraph {
    /// adjacency list: name -> Vec<(neighbor_name, relation_type)>
    adj: HashMap<String, Vec<(String, SituationRelationType)>>,
}

impl RelationGraph {
    pub fn new(relations: &[SituationRelation]) -> Self {
        let mut adj: HashMap<String, Vec<(String, SituationRelationType)>> = HashMap::new();
        for r in relations {
            adj.entry(r.source.clone())
                .or_default()
                .push((r.target.clone(), r.relation_type.clone()));
            adj.entry(r.target.clone())
                .or_default()
                .push((r.source.clone(), r.relation_type.clone()));
        }
        Self { adj }
    }

    pub fn neighbors(&self, name: &str) -> Vec<(String, SituationRelationType)> {
        self.adj.get(name).cloned().unwrap_or_default()
    }

    pub fn bfs(&self, start: &str, max_depth: usize) -> Vec<(String, SituationRelationType, usize)> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut result = Vec::new();
        let mut queue: Vec<(String, usize)> = vec![(start.to_string(), 0)];
        visited.insert(start.to_string());

        while let Some((current, depth)) = queue.pop() {
            if depth >= max_depth {
                continue;
            }
            if let Some(neighbors) = self.adj.get(&current) {
                for (n, rel) in neighbors {
                    if visited.insert(n.clone()) {
                        result.push((n.clone(), rel.clone(), depth + 1));
                        queue.push((n.clone(), depth + 1));
                    }
                }
            }
        }
        result
    }
}
