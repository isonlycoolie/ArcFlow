//! Runs async provider futures from synchronous workflow steps.

use std::future::Future;

/// Blocks on a provider future using a dedicated current-thread Tokio runtime.
pub fn block_on_provider<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("provider tokio runtime")
        .block_on(future)
}
