//! Runs async provider futures from synchronous workflow steps.

use std::future::Future;
use std::sync::OnceLock;

static PROVIDER_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn provider_runtime() -> &'static tokio::runtime::Runtime {
    PROVIDER_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|err| {
                eprintln!("arcflow: failed to build provider tokio runtime: {err}");
                std::process::exit(1);
            })
    })
}

/// Blocks on a provider future using a dedicated current-thread Tokio runtime.
pub fn block_on_provider<F: Future>(future: F) -> F::Output {
    provider_runtime().block_on(future)
}
