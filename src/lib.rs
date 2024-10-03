#![feature(try_blocks)]
#![feature(decl_macro)]
#![feature(yeet_expr)]

use crate::config::Config;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::{Pool, Postgres};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use axum_extra::headers::authorization::Bearer;
use axum_extra::{headers, TypedHeader};

pub mod config;
pub mod handlers;

macro lazy_default() {
    Lazy::new(|| Mutex::new(Default::default()))
}

pub static ARGS: Lazy<Mutex<Args>> = lazy_default!();
pub static CONFIG: Lazy<Mutex<Config>> = lazy_default!();

#[derive(clap::Parser, Debug, Default, Clone)]
pub struct Args {
    #[arg(default_value = "./server.toml")]
    pub config: PathBuf,
}

pub fn set_up_logging(file: impl AsRef<Path>) -> anyhow::Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(io::stdout())
        .chain(fern::log_file(file)?)
        .apply()?;
    Ok(())
}

#[derive(Serialize)]
pub struct ResponseJson<D: Serialize> {
    data: Option<D>,
    code: u32,
    message: Option<String>,
}

impl<D: Serialize> ResponseJson<D> {
    pub fn ok(data: D) -> Self {
        Self {
            data: Some(data),
            code: 0,
            message: None,
        }
    }

    pub fn error() -> Self {
        Self {
            data: None,
            code: 1,
            message: None,
        }
    }

    pub fn error_msg<S: Into<String>>(message: S) -> Self {
        Self {
            data: None,
            code: 1,
            message: Some(message.into()),
        }
    }
}

impl<D: Serialize> IntoResponse for ResponseJson<D> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub macro mutex_lock($e:expr) {
    $e.lock().unwrap()
}

pub macro api_ok($d:expr) {
    crate::ResponseJson::ok($d).into_response()
}

pub macro include_sql($name:literal) {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sqls/", $name, ".sql"))
}

pub type DbPool = Pool<Postgres>;

pub struct ApiContext {
    pub db: DbPool,
}

pub type ApiExtension = Extension<Arc<ApiContext>>;
pub type AuthHeader = TypedHeader<headers::Authorization<Bearer>>;
