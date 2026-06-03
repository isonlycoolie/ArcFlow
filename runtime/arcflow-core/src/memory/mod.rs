//! Memory subsystem (Sprint 4).

mod chunking;
mod coordinator;
mod error;
mod hybrid;
mod namespace;
mod persistent;
mod provider;
mod rerank;
mod session;
mod shared;
mod vector;

pub use crate::embedding::stub_embedding;
pub use chunking::{
    extract_chunk_metadata, ChunkMetadata, ChunkStrategy, RecursiveCharacterSplitter,
};
pub use coordinator::MemoryCoordinator;
pub use error::MemoryError;
pub use hybrid::{
    sparse_lexical_score, HybridHit, HybridRetriever, DEFAULT_DENSE_WEIGHT, DEFAULT_SPARSE_WEIGHT,
};
pub use namespace::{durable_key, session_key};
pub use persistent::PersistentMemory;
pub use provider::VectorStoreProvider;
pub use rerank::{resolve_rerank_provider, RankedChunk, RerankError, RerankProvider};
pub use session::SessionMemory;
pub use shared::SharedMemory;
pub use vector::{
    vector_config_from_memory, ChunkingConfig, HybridRetrievalConfig, RetrievalMode, VectorMemory,
    VectorMemoryConfig,
};
