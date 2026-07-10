use crate::config::configs::Config;
use sqlx::SqlitePool;

#[allow(dead_code)]
pub struct AppState {
    config: Config,
    database_pool: SqlitePool,
}
