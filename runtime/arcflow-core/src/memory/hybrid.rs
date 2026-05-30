//! Dense + sparse score fusion for hybrid retrieval (Phase 2.5).

use std::collections::HashMap;

/// Default dense vector weight in hybrid fusion.
pub const DEFAULT_DENSE_WEIGHT: f32 = 0.7;
/// Default sparse (lexical) weight in hybrid fusion.
pub const DEFAULT_SPARSE_WEIGHT: f32 = 0.3;

/// One retrieval candidate with separate dense and sparse scores.
#[derive(Clone, Debug, PartialEq)]
pub struct HybridHit {
    pub point_id: String,
    pub dense_score: f32,
    pub sparse_score: f32,
}

/// Fuses dense and sparse scores with configurable weights.
#[derive(Clone, Debug)]
pub struct HybridRetriever {
    dense_weight: f32,
    sparse_weight: f32,
}

impl Default for HybridRetriever {
    fn default() -> Self {
        Self::new(DEFAULT_DENSE_WEIGHT, DEFAULT_SPARSE_WEIGHT)
    }
}

impl HybridRetriever {
    /// Creates a retriever with the given fusion weights.
    pub fn new(dense_weight: f32, sparse_weight: f32) -> Self {
        Self {
            dense_weight,
            sparse_weight,
        }
    }

    /// Weighted linear fusion of normalized scores.
    pub fn fuse_score(&self, dense_score: f32, sparse_score: f32) -> f32 {
        self.dense_weight * dense_score + self.sparse_weight * sparse_score
    }

    /// Merges hits by `point_id`, fuses scores, returns top `limit` by fused score.
    pub fn rank(&self, hits: Vec<HybridHit>, limit: usize) -> Vec<(String, f32)> {
        let mut by_id: HashMap<String, (f32, f32)> = HashMap::new();
        for hit in hits {
            let entry = by_id
                .entry(hit.point_id)
                .or_insert((hit.dense_score, hit.sparse_score));
            entry.0 = entry.0.max(hit.dense_score);
            entry.1 = entry.1.max(hit.sparse_score);
        }
        let mut ranked: Vec<(String, f32)> = by_id
            .into_iter()
            .map(|(id, (dense, sparse))| (id, self.fuse_score(dense, sparse)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(limit);
        ranked
    }
}

/// Lexical overlap score in `[0.0, 1.0]` for MVP sparse leg (no Qdrant sparse index yet).
pub fn sparse_lexical_score(query: &str, document: &str) -> f32 {
    let query_tokens = tokenize(query);
    if query_tokens.is_empty() {
        return 0.0;
    }
    let doc_tokens: std::collections::HashSet<_> = tokenize(document).into_iter().collect();
    let matches = query_tokens
        .iter()
        .filter(|t| doc_tokens.contains(*t))
        .count();
    (matches as f32) / (query_tokens.len() as f32)
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuse_score_uses_weights() {
        let retriever = HybridRetriever::new(0.7, 0.3);
        let fused = retriever.fuse_score(1.0, 0.0);
        assert!((fused - 0.7).abs() < f32::EPSILON);
