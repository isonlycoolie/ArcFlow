//! Document chunking for vector memory (Phase 2.5).

use serde::{Deserialize, Serialize};

/// Heuristic metadata extracted from document headers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub title: Option<String>,
    pub source: Option<String>,
    pub page: Option<u32>,
}

/// Extracts title, source, and page hints from leading markdown or plain headers.
pub fn extract_chunk_metadata(text: &str) -> ChunkMetadata {
    let mut meta = ChunkMetadata::default();
    for line in text.lines().take(12) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if meta.title.is_none() {
            if let Some(title) = trimmed.strip_prefix("# ") {
                meta.title = Some(title.trim().to_string());
                continue;
            }
            if trimmed.starts_with("Title:") {
                meta.title = Some(trimmed.trim_start_matches("Title:").trim().to_string());
                continue;
            }
        }
        if meta.source.is_none() {
            if let Some(source) = trimmed.strip_prefix("Source:") {
                meta.source = Some(source.trim().to_string());
                continue;
            }
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                meta.source = Some(trimmed.to_string());
                continue;
            }
        }
        if meta.page.is_none() {
            if let Some(page) = trimmed.strip_prefix("Page:") {
                if let Ok(num) = page.trim().parse::<u32>() {
                    meta.page = Some(num);
                }
            }
        }
    }
    meta
}

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
    }

    fn split_on_separator(&self, text: &str, separator: &str) -> Vec<String> {
        if separator.is_empty() {
            return text.chars().map(|c| c.to_string()).collect();
        }
        text.split(separator)
            .map(|part| {
                if part.is_empty() {
                    String::new()
                } else {
                    format!("{part}{separator}")
                }
            })
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl ChunkStrategy for RecursiveCharacterSplitter {
    fn split(&self, text: &str) -> Vec<String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        if trimmed.len() <= self.chunk_size {
            return vec![trimmed.to_string()];
        }

        for separator in &self.separators {
            if separator.is_empty() || trimmed.contains(separator) {
                let splits = self.split_on_separator(trimmed, separator);
                return self.merge_splits(splits);
            }
        }
        vec![trimmed.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_no_chunks() {
        let splitter = RecursiveCharacterSplitter::default();
        assert!(splitter.split("").is_empty());
        assert!(splitter.split("   ").is_empty());
    }

    #[test]
    fn short_text_single_chunk() {
        let splitter = RecursiveCharacterSplitter::new(100, 0);
        let chunks = splitter.split("hello world");
        assert_eq!(chunks, vec!["hello world".to_string()]);
    }

    #[test]
    fn splits_on_paragraph_boundaries() {
        let splitter = RecursiveCharacterSplitter::new(40, 0);
        let text = "aaaa bbbb cccc dddd.\n\neeee ffff gggg hhhh.";
        let chunks = splitter.split(text);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.len() <= 60, "chunk too large: {}", chunk.len());
        }
    }

    #[test]
    fn overlap_prefixes_later_chunks() {
        let splitter = RecursiveCharacterSplitter::new(20, 5);
        let text = "one two three four five six seven eight nine ten";
        let chunks = splitter.split(text);
        assert!(chunks.len() >= 2);
        if chunks.len() >= 2 {
            let second_start: String = chunks[1].chars().take(5).collect();
            let first_end: String = chunks[0]
                .chars()
                .rev()
                .take(5)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            assert_eq!(second_start, first_end);
        }
    }

    #[test]
    fn extract_metadata_from_markdown_header() {
        let text = "# ArcFlow Guide\nSource: https://arcflows.vercel.com\nPage: 3\n\nBody text.";
        let meta = extract_chunk_metadata(text);
        assert_eq!(meta.title.as_deref(), Some("ArcFlow Guide"));
        assert_eq!(meta.source.as_deref(), Some("https://arcflows.vercel.com"));
        assert_eq!(meta.page, Some(3));
    }
}
