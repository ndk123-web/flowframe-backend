use crate::config::configs::Config;
use sqlx::SqlitePool;

pub struct AppState {
    config: Config,
    database_pool: SqlitePool,
}