use crate::config::configs::Config;
use crate::repositories::auth_repositories::AuthRepository;
use crate::services::auth_services::AuthService;
use mongodb::Database;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Database,
    pub auth_service: AuthService,
}

impl AppState {
    pub fn new(config: Config, db: Database) -> Self {
        let auth_repo = AuthRepository::new(&db);
        let auth_service = AuthService::new(auth_repo, config.jwt_secret.clone());

        Self {
            config,
            db,
            auth_service,
        }
    }
}
