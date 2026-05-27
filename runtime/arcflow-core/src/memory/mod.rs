//! Memory subsystem (Sprint 4).

mod coordinator;
mod error;
mod namespace;
mod persistent;
mod provider;
mod session;
mod shared;
mod vector;

pub use coordinator::MemoryCoordinator;
pub use error::MemoryError;
pub use namespace::{durable_key, session_key};
pub use persistent::PersistentMemory;
pub use provider::VectorStoreProvider;
pub use session::SessionMemory;
pub use shared::SharedMemory;
pub use vector::{stub_embedding, VectorMemory};
