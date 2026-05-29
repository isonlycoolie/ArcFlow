//! Memory subsystem (Sprint 4).

mod chunking;
mod coordinator;
mod error;
mod hybrid;
mod namespace;
mod persistent;
mod provider;
mod session;
mod shared;
mod vector;

pub use chunking::{ChunkStrategy, RecursiveCharacterSplitter};
pub use coordinator::MemoryCoordinator;
pub use error::MemoryError;
pub use hybrid::{
    HybridHit, HybridRetriever, DEFAULT_DENSE_WEIGHT, DEFAULT_SPARSE_WEIGHT,
};
pub use namespace::{durable_key, session_key};
pub use persistent::PersistentMemory;
pub use provider::VectorStoreProvider;
pub use session::SessionMemory;
pub use shared::SharedMemory;
pub use vector::{
    ChunkingConfig, HybridRetrievalConfig, RetrievalMode, VectorMemory, VectorMemoryConfig,
};
pub use crate::embedding::stub_embedding;
