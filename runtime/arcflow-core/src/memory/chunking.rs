//! Document chunking for vector memory (Phase 2.5).

/// Splits text into retrieval-sized chunks.
pub trait ChunkStrategy {
    /// Returns ordered chunk strings (may be empty for empty input).
    fn split(&self, text: &str) -> Vec<String>;
}

/// Recursive character splitter (paragraph → line → word → char).
#[derive(Clone, Debug)]
pub struct RecursiveCharacterSplitter {
    chunk_size: usize,
    overlap: usize,
    separators: Vec<&'static str>,
}

impl Default for RecursiveCharacterSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            overlap: 64,
            separators: vec!["\n\n", "\n", " ", ""],
        }
    }
}

impl RecursiveCharacterSplitter {
    /// Builds a splitter with explicit size and overlap.
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
            ..Self::default()
        }
    }

    fn merge_splits(&self, splits: Vec<String>) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current = String::new();

        for piece in splits {
            if piece.is_empty() {
                continue;
            }
            let candidate = if current.is_empty() {
                piece.clone()
            } else {
                format!("{current}{piece}")
            };
            if candidate.len() <= self.chunk_size {
                current = candidate;
                continue;
            }
            if !current.is_empty() {
                chunks.push(current.clone());
            }
            if piece.len() > self.chunk_size {
                chunks.extend(self.split(&piece));
                current.clear();
                continue;
            }
            current = piece;
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        self.apply_overlap(chunks)
    }

    fn apply_overlap(&self, mut chunks: Vec<String>) -> Vec<String> {
        if self.overlap == 0 || chunks.len() <= 1 {
            return chunks;
        }
        let mut with_overlap = Vec::with_capacity(chunks.len());
        for (i, chunk) in chunks.drain(..).enumerate() {
            if i == 0 {
                with_overlap.push(chunk);
                continue;
            }
            let prev = with_overlap.last().expect("previous chunk");
            let prefix = prev
                .chars()
                .rev()
                .take(self.overlap)
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>();
            if prefix.is_empty() {
                with_overlap.push(chunk);
            } else {
                with_overlap.push(format!("{prefix}{chunk}"));
            }
        }
        with_overlap
