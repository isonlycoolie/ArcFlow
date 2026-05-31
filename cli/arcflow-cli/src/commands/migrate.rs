//! `arcflow migrate up` — apply embedded PostgreSQL migrations.

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct MigrateArgs {
    #[command(subcommand)]
    pub command: MigrateCommand,
}

#[derive(Subcommand)]
pub enum MigrateCommand {
    /// Apply pending schema migrations.
    Up(MigrateUpArgs),
}

#[derive(Parser)]
pub struct MigrateUpArgs {}

pub fn run(args: MigrateArgs) -> i32 {
    match args.command {
        MigrateCommand::Up(_) => migrate_up(),
    }
}

fn migrate_up() -> i32 {
    let url = match std::env::var("ARCFLOW_POSTGRESQL_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => {
            eprintln!("[ArcFlow] ARCFLOW_POSTGRESQL_URL must be set for migrate up.");
            return 2;
        }
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ArcFlow] failed to start async runtime: {e}");
            return 1;
        }
    };
    rt.block_on(async {
        let pool = match sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[ArcFlow] postgres connect failed: {e}");
                return 1;
            }
        };
        match arcflow_core::migrate::run(&pool).await {
            Ok(()) => {
                println!("[ArcFlow] migrations applied.");
                0
            }
            Err(e) => {
                eprintln!("[ArcFlow] migration failed: {e}");
                1
            }
        }
    })
}
