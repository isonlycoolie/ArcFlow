//! Optional shared Postgres pool for recovery writes (Phase C).

use std::sync::OnceLock;

use sqlx::postgres::PgPool;

static SHARED_POOL: OnceLock<PgPool> = OnceLock::new();

/// Installs the server pool for best-effort recovery persistence.
pub fn install_shared_pool(pool: PgPool) {
    let _ = SHARED_POOL.set(pool);
}

pub(crate) fn shared_pool() -> Option<&'static PgPool> {
    SHARED_POOL.get()
}
