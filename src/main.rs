#![feature(try_blocks)]
#![feature(decl_macro)]
#![feature(yeet_expr)]
#![feature(let_chains)]

use axum_template as server;

use axum::extract::DefaultBodyLimit;
use axum::{Extension, Router};
use clap::Parser;
use log::{debug, info};
use server::config::get_config;
use server::{handlers, mutex_lock, set_up_logging, ApiContext, Args, ARGS, CONFIG};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = get_config(&args.config)?;
    if let Some(l) = &config.logging
        && let Some(f) = &l.file
    {
        set_up_logging(f)?;
    }
    debug!("Args: {:?}", args);
    debug!("Configs: {:?}", config);
    *mutex_lock!(ARGS) = args.clone();
    *mutex_lock!(CONFIG) = config.clone();

    info!("Connecting to the database...");
    let db_string = format!(
        "postgres://postgres:{}@{}/{}",
        config.db.password, config.db.ip, config.db.database
    );
    let pool = PgPoolOptions::new().connect(db_string.as_str()).await?;

    info!("Testing the connection...");
    let row: (i32,) = sqlx::query_as("SELECT $1")
        .bind(42_i32)
        .fetch_one(&pool)
        .await?;
    assert_eq!(row.0, 42);
    info!("Done");

    start_axum(Arc::new(ApiContext { db: pool })).await?;

    Ok(())
}

fn router() -> Router {
    Router::new().nest("/", handlers::router())
}

async fn start_axum(api_context: Arc<ApiContext>) -> anyhow::Result<()> {
    let listen_port = mutex_lock!(CONFIG).listen_port;

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), listen_port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let router = router()
        .layer(Extension(api_context))
        .layer(DefaultBodyLimit::max(1048576 * 50));
    info!("Server started on {}", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
