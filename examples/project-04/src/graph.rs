use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

use crate::models::*;
use crate::tokenizer;

fn parse_relation_name(name: &str) -> (String, String, bool) {
    for sep in &[" ⇄ ", " ↔ ", " → "] {
        if let Some(pos) = name.find(sep) {
            let source = name[..pos].trim().to_string();
            let target = name[pos + sep.len()..].trim().to_string();
            let bidirectional = *sep == " ⇄ " || *sep == " ↔ ";
            return (source, target, bidirectional);
        }
    }
    (String::new(), name.to_string(), false)
}

fn parse_relation_type(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(paren) = raw.find('（') {
        raw[..paren].trim().to_string()
    } else if let Some(paren) = raw.find('(') {
        raw[..paren].trim().to_string()
    } else if raw.contains("双向") {
        "支持".to_string()
    } else if raw.contains(" + ") {
        raw.split(" + ").next().unwrap_or(raw).trim().to_string()
    } else {
        raw.to_string()
    }
}

fn match_cluster_id(clusters: &[Cluster], reference: &str) -> Option<u32> {
    let ref_lower = reference.to_lowercase();

    // 1. exact substring match on whole reference
    for cluster in clusters {
        if cluster.name.to_lowercase().contains(&ref_lower) {
            return Some(cluster.id);
        }
    }

    // 1.5 search in evolution text as well
    for cluster in clusters {
        if let Some(ref evo) = cluster.evolution {
            if evo.to_lowercase().contains(&ref_lower) {
                return Some(cluster.id);
            }
        }
    }

    // 2. split reference into meaningful words (e.g. "研发/探索" -> ["研发", "探索"])
    let words: Vec<String> = ref_lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_string())
        .collect();
    for cluster in clusters {
        let name_lower = cluster.name.to_lowercase();
        for word in &words {
            // prefer longer words for matching
            if word.len() >= 3 && name_lower.contains(word.as_str()) {
                return Some(cluster.id);
            }
        }
    }
    // shorter word match (2 chars) as fallback
    for cluster in clusters {
        let name_lower = cluster.name.to_lowercase();
        for word in &words {
            if word.len() >= 2 && name_lower.contains(word.as_str()) {
                return Some(cluster.id);
            }
        }
    }

    // 3. bigram overlap (handles abbreviations and partial matches)
    let ref_bigrams: std::collections::HashSet<String> = ref_lower
        .chars()
        .collect::<Vec<char>>()
        .windows(2)
        .map(|w| w.iter().collect::<String>())
        .filter(|b| b.chars().all(|c| c.is_alphanumeric()))
        .collect();
    let mut best: Option<(u32, f64)> = None;
    for cluster in clusters {
        let name_lower = cluster.name.to_lowercase();
        let name_bigrams: std::collections::HashSet<String> = name_lower
            .chars()
            .collect::<Vec<char>>()
            .windows(2)
            .map(|w| w.iter().collect::<String>())
            .filter(|b| b.chars().all(|c| c.is_alphanumeric()))
            .collect();
        if ref_bigrams.is_empty() || name_bigrams.is_empty() {
            continue;
        }
        let intersection = ref_bigrams.intersection(&name_bigrams).count();
        let recall = intersection as f64 / ref_bigrams.len() as f64;
        if recall >= 0.3 && best.map_or(true, |(_, b)| recall > b) {
            best = Some((cluster.id, recall));
        }
    }
    best.map(|(id, _)| id)
}

pub struct IntentGraph {
    graph: DiGraph<NodeWeight, EdgeWeight>,
    node_index_by_id: HashMap<u32, NodeIndex>,
}

impl IntentGraph {
    pub fn new() -> Self {
        IntentGraph {
            graph: DiGraph::new(),
            node_index_by_id: HashMap::new(),
        }
    }

    pub fn from_yaml(
        intent_path: &str,
        relation_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut ig = IntentGraph::new();

        let intent_content = fs::read_to_string(intent_path)?;
        let intent_yaml: IntentYaml = serde_yaml::from_str(&intent_content)?;
        let clusters = intent_yaml.clusters;

        for cluster in &clusters {
            let per_week_intents: Vec<String> = cluster
                .per_week
                .values()
                .flat_map(|v| v.clone())
                .collect();
            let node = NodeWeight {
                id: cluster.id,
                name: cluster.name.clone(),
                r#type: cluster.cluster_type.clone().unwrap_or_default(),
                evolution: cluster.evolution.clone().unwrap_or_default(),
                per_week_intents,
            };
            let idx = ig.graph.add_node(node);
            ig.node_index_by_id.insert(cluster.id, idx);
        }

        let relation_content = fs::read_to_string(relation_path)?;
        let relation_yaml: RelationYaml = serde_yaml::from_str(&relation_content)?;

        for entry in &relation_yaml.stable_relations {
            ig.add_edges_from_entry(entry, "stable", &clusters);
        }
        for entry in &relation_yaml.periodic_tensions {
            ig.add_edges_from_entry(entry, "periodic", &clusters);
        }
        for entry in &relation_yaml.situational_relations {
            ig.add_edges_from_entry(
                &RelationEntry {
                    name: entry.name.clone(),
                    relation_type: entry.relation_type.clone(),
                    weeks: entry.weeks.clone(),
                    logic: String::new(),
                },
                "situational",
                &clusters,
            );
        }

        Ok(ig)
    }

    fn add_edges_from_entry(
        &mut self,
        entry: &RelationEntry,
        period_type: &str,
        clusters: &[Cluster],
    ) {
        let (source_ref, target_ref, bidirectional) = parse_relation_name(&entry.name);
        if source_ref.is_empty() {
            return;
        }
        let source_id = match_cluster_id(clusters, &source_ref);
        let target_id = match_cluster_id(clusters, &target_ref);
        if let (Some(sid), Some(tid)) = (source_id, target_id) {
            let rel_type = parse_relation_type(&entry.relation_type);
            let weight = EdgeWeight {
                relation_type: rel_type.clone(),
                logic: entry.logic.clone(),
                weeks: entry.weeks.clone(),
                period_type: period_type.to_string(),
            };
            if let (Some(&si), Some(&ti)) =
                (self.node_index_by_id.get(&sid), self.node_index_by_id.get(&tid))
            {
                self.graph.add_edge(si, ti, weight);
                if bidirectional {
                    let rev_weight = EdgeWeight {
                        relation_type: rel_type,
                        logic: entry.logic.clone(),
                        weeks: entry.weeks.clone(),
                        period_type: period_type.to_string(),
                    };
                    self.graph.add_edge(ti, si, rev_weight);
                }
            }
        }
    }

    pub fn save_json(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let id_by_index: HashMap<NodeIndex, u32> = self
            .node_index_by_id
            .iter()
            .map(|(&id, &idx)| (idx, id))
            .collect();
        for idx in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight(idx) {
                nodes.push(node.clone());
            }
        }
        for e in self.graph.edge_references() {
            let source = id_by_index[&e.source()];
            let target = id_by_index[&e.target()];
            edges.push(EdgeData {
                source,
                target,
                weight: e.weight().clone(),
            });
        }
        let data = GraphData { nodes, edges };
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_json(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let data: GraphData = serde_json::from_str(&content)?;
        let mut ig = IntentGraph::new();
        for node in &data.nodes {
            let idx = ig.graph.add_node(node.clone());
            ig.node_index_by_id.insert(node.id, idx);
        }
        for edge in &data.edges {
            if let (Some(&si), Some(&ti)) =
                (ig.node_index_by_id.get(&edge.source), ig.node_index_by_id.get(&edge.target))
            {
                ig.graph.add_edge(si, ti, edge.weight.clone());
            }
        }
        Ok(ig)
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn match_nodes(
        &self,
        keywords: &KeywordTable,
        text: &str,
        threshold: f64,
    ) -> Vec<MatchedNode> {
        let text_words = tokenizer::tokenize(text);
        let text_word_set: HashSet<&str> = text_words.iter().map(|s| s.as_str()).collect();
        let mut matched = Vec::new();
        for entry in keywords {
            let cluster_keywords: Vec<&str> = entry.keywords.iter().map(|s| s.as_str()).collect();
            let cluster_count = cluster_keywords.len();
            if cluster_count == 0 {
                continue;
            }
            let intersection_count = cluster_keywords
                .iter()
                .filter(|kw| text_word_set.contains(**kw))
                .count();
            let score = intersection_count as f64 / cluster_count as f64;
            if score > threshold {
                let evidence: Vec<String> = cluster_keywords
                    .iter()
                    .filter(|kw| text_word_set.contains(**kw))
                    .map(|s| s.to_string())
                    .collect();
                matched.push(MatchedNode {
                    id: entry.id,
                    name: entry.name.clone(),
                    score: (score * 100.0).round() / 100.0,
                    evidence,
                });
            }
        }
        matched.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        matched
    }

    pub fn neighbors(&self, node_id: u32) -> Vec<NeighborInfo> {
        let mut result = Vec::new();
        if let Some(&idx) = self.node_index_by_id.get(&node_id) {
            for e in self.graph.edges(idx) {
                let target_id = self.get_node_id(e.target());
                if let (Some(target), weight) = (target_id, e.weight()) {
                    result.push(NeighborInfo {
                        from: node_id,
                        to: target,
                        relation: weight.relation_type.clone(),
                        logic: weight.logic.clone(),
                        direction: "outgoing".to_string(),
                    });
                }
            }
            for e in self.graph.edges_directed(idx, petgraph::Direction::Incoming) {
                let source_id = self.get_node_id(e.source());
                if let (Some(source), weight) = (source_id, e.weight()) {
                    result.push(NeighborInfo {
                        from: source,
                        to: node_id,
                        relation: weight.relation_type.clone(),
                        logic: weight.logic.clone(),
                        direction: "incoming".to_string(),
                    });
                }
            }
        }
        result
    }

    pub fn bfs(&self, start_id: u32, max_depth: usize) -> Vec<Vec<PathStep>> {
        let mut all_paths = Vec::new();
        let start_idx = match self.node_index_by_id.get(&start_id) {
            Some(&idx) => idx,
            None => return all_paths,
        };
        let mut queue: VecDeque<(NodeIndex, Vec<PathStep>)> = VecDeque::new();
        queue.push_back((start_idx, Vec::new()));
        while let Some((current, path)) = queue.pop_front() {
            if path.len() >= max_depth {
                continue;
            }
            for e in self.graph.edges(current) {
                let next = e.target();
                let weight = e.weight();
                let from_id = self.get_node_id(current).unwrap_or(0);
                let to_id = self.get_node_id(next).unwrap_or(0);
                let mut new_path = path.clone();
                new_path.push(PathStep {
                    from: from_id,
                    to: to_id,
                    relation: weight.relation_type.clone(),
                });
                all_paths.push(new_path.clone());
                queue.push_back((next, new_path));
            }
        }
        all_paths
    }

    pub fn detect_conflicts(&self, node_ids: &[u32]) -> Vec<ConflictInfo> {
        let mut conflicts = Vec::new();
        let id_set: HashSet<u32> = node_ids.iter().copied().collect();
        for &a in node_ids {
            for &b in node_ids {
                if a >= b {
                    continue;
                }
                if let (Some(&ai), Some(&bi)) =
                    (self.node_index_by_id.get(&a), self.node_index_by_id.get(&b))
                {
                    for e in self.graph.edges(ai) {
                        if e.target() == bi && e.weight().relation_type == "冲突" {
                            conflicts.push(ConflictInfo {
                                node_a: a,
                                node_b: b,
                                relation_type: "冲突".to_string(),
                                via: vec![],
                            });
                        }
                    }
                    for e in self.graph.edges(bi) {
                        if e.target() == ai && e.weight().relation_type == "冲突" {
                            conflicts.push(ConflictInfo {
                                node_a: a,
                                node_b: b,
                                relation_type: "冲突".to_string(),
                                via: vec![],
                            });
                        }
                    }
                }
            }
        }
        for &a in node_ids {
            let paths = self.bfs(a, 2);
            for path in &paths {
                if path.is_empty() {
                    continue;
                }
                let last = &path[path.len() - 1];
                if last.relation == "冲突" && id_set.contains(&last.to) && last.to != a {
                    if !conflicts.iter().any(|c| {
                        (c.node_a == a && c.node_b == last.to)
                            || (c.node_a == last.to && c.node_b == a)
                    }) {
                        let via: Vec<u32> = path.iter().map(|s| s.to).collect();
                        conflicts.push(ConflictInfo {
                            node_a: a,
                            node_b: last.to,
                            relation_type: "冲突".to_string(),
                            via,
                        });
                    }
                }
            }
        }
        conflicts
    }

    pub fn candidate_edges(&self, node_ids: &[u32]) -> Vec<CandidateEdge> {
        let mut candidates = Vec::new();
        for &a in node_ids {
            for &b in node_ids {
                if a >= b {
                    continue;
                }
                if self.has_direct_edge(a, b) {
                    continue;
                }
                let paths = self.bfs(a, 2);
                for path in &paths {
                    if path.is_empty() {
                        continue;
                    }
                    let last = &path[path.len() - 1];
                    if last.to == b {
                        let types_in_path: HashSet<&str> =
                            path.iter().map(|s| s.relation.as_str()).collect();
                        let proposed_type = if types_in_path.contains("冲突") {
                            "冲突"
                        } else {
                            "支持"
                        };
                        let evidence = format!("路径经由{}个中间节点", path.len());
                        let confidence = 1.0 / (path.len() as f64 + 1.0);
                        candidates.push(CandidateEdge {
                            from: a,
                            to: b,
                            proposed_type: proposed_type.to_string(),
                            evidence,
                            confidence: (confidence * 100.0).round() / 100.0,
                        });
                        break;
                    }
                }
            }
        }
        candidates
    }

    fn has_direct_edge(&self, a: u32, b: u32) -> bool {
        if let (Some(&ai), Some(&bi)) =
            (self.node_index_by_id.get(&a), self.node_index_by_id.get(&b))
        {
            for e in self.graph.edges(ai) {
                if e.target() == bi {
                    return true;
                }
            }
        }
        false
    }

    fn get_node_id(&self, idx: NodeIndex) -> Option<u32> {
        self.graph.node_weight(idx).map(|n| n.id)
    }

    pub fn add_edge(&mut self, from: u32, to: u32, weight: EdgeWeight) {
        if let (Some(&fi), Some(&ti)) =
            (self.node_index_by_id.get(&from), self.node_index_by_id.get(&to))
        {
            self.graph.add_edge(fi, ti, weight);
        }
    }

    pub fn infer(
        &self,
        keywords: &KeywordTable,
        text: &str,
        threshold: f64,
    ) -> InferenceOutput {
        let match_nodes = self.match_nodes(keywords, text, threshold);
        let matched_ids: Vec<u32> = match_nodes.iter().map(|n| n.id).collect();

        let mut all_neighbors = Vec::new();
        for &id in &matched_ids {
            all_neighbors.extend(self.neighbors(id));
        }

        let mut all_paths = Vec::new();
        for &id in &matched_ids {
            all_paths.extend(self.bfs(id, 2));
        }

        let conflicts = self.detect_conflicts(&matched_ids);

        let candidate_edges = self.candidate_edges(&matched_ids);

        InferenceOutput {
            match_nodes,
            neighbors: all_neighbors,
            bfs_paths: all_paths,
            conflicts,
            candidate_edges,
        }
    }
}
