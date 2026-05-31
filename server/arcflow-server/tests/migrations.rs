//! Postgres migration integration (requires ARCFLOW_POSTGRESQL_URL).

use sqlx::postgres::PgPool;

#[tokio::test]
#[ignore = "requires postgres"]
async fn migrate_up_creates_sites_table() {
    let url = std::env::var("ARCFLOW_POSTGRESQL_URL").expect("ARCFLOW_POSTGRESQL_URL");
    let pool = PgPool::connect(&url).await.expect("connect");
    arcflow_core::migrate::run(&pool).await.expect("migrate");
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = 'arcflow_sites'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("query");
    assert!(exists);
    arcflow_core::migrate::validate(&pool).await.expect("validate");
}
