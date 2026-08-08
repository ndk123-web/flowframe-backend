use crate::config::configs::Config;
use crate::repositories::auth_repositories::AuthRepository;
use crate::repositories::diagram_repository::DiagramRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::services::auth_services::AuthService;
use crate::services::diagram_services::DiagramService;
use crate::services::workspace_services::WorkspaceService;
use mongodb::Database;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Database,
    pub auth_service: AuthService,
    pub workspace_service: WorkspaceService,
    pub diagram_service: DiagramService,
}

impl AppState {
    pub fn new(config: Config, db: Database) -> Self {
        let auth_repo = AuthRepository::new(&db);
        let workspace_repo = WorkspaceRepository::new(&db);
        let diagram_repo = DiagramRepository::new(&db);

        let auth_service = AuthService::new(auth_repo, config.jwt_secret.clone());
        let workspace_service =
            WorkspaceService::new(workspace_repo.clone(), diagram_repo.clone());
        let diagram_service =
            DiagramService::new(diagram_repo.clone(), workspace_repo.clone());

        Self {
            config,
            db,
            auth_service,
            workspace_service,
            diagram_service,
        }
    }
}
