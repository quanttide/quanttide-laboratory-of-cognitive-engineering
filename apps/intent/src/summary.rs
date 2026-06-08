use crate::SessionFile;

pub struct SessionSummary {
    pub total_turns: usize,
    pub cluster_count: usize,
    pub edge_count: usize,
    pub insight_count: usize,
    pub motif_count: usize,
}

impl SessionSummary {
    pub fn from_session(session: &SessionFile) -> Self {
        let total_turns = session.turns.len();
        let cluster_count: usize = session.turns.iter()
            .flat_map(|t| t.parsed.discovery_update.new_clusters.iter())
            .count();
        let edge_count: usize = session.turns.iter()
            .flat_map(|t| t.parsed.discovery_update.new_edge_ids.iter())
            .count();
        let insight_count: usize = session.turns.iter()
            .flat_map(|t| t.parsed.discovery_update.new_insights.iter())
            .count();
        let motif_count: usize = session.turns.iter()
            .filter(|t| t.parsed.motif.is_some())
            .count();
        Self { total_turns, cluster_count, edge_count, insight_count, motif_count }
    }
}

impl std::fmt::Display for SessionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "总轮次: {}\n探索: {} 簇, {} 边, {} 洞察, {} 母题",
            self.total_turns, self.cluster_count, self.edge_count, self.insight_count, self.motif_count
        )
    }
}
