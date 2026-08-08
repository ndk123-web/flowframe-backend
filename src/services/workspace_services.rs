use crate::dtos::workspace_dto::{CreateWorkspaceRequest, UpdateWorkspaceRequest, WorkspaceResponse};
use crate::models::workspace_model::Workspace;
use crate::repositories::diagram_repository::DiagramRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use anyhow::{anyhow, Result};
use bson::oid::ObjectId;
use bson::DateTime;

#[derive(Clone)]
pub struct WorkspaceService {
    workspace_repo: WorkspaceRepository,
    diagram_repo: DiagramRepository,
}

impl WorkspaceService {
    pub fn new(workspace_repo: WorkspaceRepository, diagram_repo: DiagramRepository) -> Self {
        Self {
            workspace_repo,
            diagram_repo,
        }
    }

    pub async fn create_workspace(
        &self,
        user_id_str: &str,
        req: CreateWorkspaceRequest,
    ) -> Result<WorkspaceResponse> {
        let user_id = ObjectId::parse_str(user_id_str)?;

        // Enforce 5 workspace limit per user
        let existing = self.workspace_repo.find_by_user(&user_id).await?;
        if existing.len() >= 5 {
            return Err(anyhow!("Workspace limit reached (5/5 for Personal Plan)"));
        }

        let now = DateTime::now();

        let new_workspace = Workspace {
            id: None,
            user_id,
            name: req.name,
            description: req.description,
            env: req.env.unwrap_or_else(|| "DEV".to_string()),
            color: req.color,
            icon_type: req.icon_type,
            created_at: now,
            updated_at: now,
        };

        let created = self.workspace_repo.create_workspace(new_workspace).await?;
        let ws_id = created.id.map(|id| id.to_hex()).unwrap_or_default();

        Ok(WorkspaceResponse {
            id: ws_id,
            user_id: user_id_str.to_string(),
            name: created.name,
            description: created.description,
            env: created.env,
            color: created.color,
            icon_type: created.icon_type,
            diagrams_count: 0,
            created_at: created.created_at.to_string(),
            updated_at: created.updated_at.to_string(),
        })
    }

    pub async fn get_user_workspaces(&self, user_id_str: &str) -> Result<Vec<WorkspaceResponse>> {
        let user_id = ObjectId::parse_str(user_id_str)?;
        let workspaces = self.workspace_repo.find_by_user(&user_id).await?;

        let mut responses = Vec::new();
        for ws in workspaces {
            let ws_id = ws.id.unwrap();
            let diagrams = self
                .diagram_repo
                .find_by_workspace(&ws_id, &user_id)
                .await?;

            responses.push(WorkspaceResponse {
                id: ws_id.to_hex(),
                user_id: user_id_str.to_string(),
                name: ws.name,
                description: ws.description,
                env: ws.env,
                color: ws.color,
                icon_type: ws.icon_type,
                diagrams_count: diagrams.len() as i64,
                created_at: ws.created_at.to_string(),
                updated_at: ws.updated_at.to_string(),
            });
        }

        Ok(responses)
    }

    pub async fn get_workspace_by_id(
        &self,
        workspace_id_str: &str,
        user_id_str: &str,
    ) -> Result<WorkspaceResponse> {
        let ws_id = ObjectId::parse_str(workspace_id_str)?;
        let user_id = ObjectId::parse_str(user_id_str)?;

        let ws = self
            .workspace_repo
            .find_by_id(&ws_id, &user_id)
            .await?
            .ok_or_else(|| anyhow!("Workspace not found"))?;

        let diagrams = self
            .diagram_repo
            .find_by_workspace(&ws_id, &user_id)
            .await?;

        Ok(WorkspaceResponse {
            id: ws_id.to_hex(),
            user_id: user_id_str.to_string(),
            name: ws.name,
            description: ws.description,
            env: ws.env,
            color: ws.color,
            icon_type: ws.icon_type,
            diagrams_count: diagrams.len() as i64,
            created_at: ws.created_at.to_string(),
            updated_at: ws.updated_at.to_string(),
        })
    }

    pub async fn update_workspace(
        &self,
        workspace_id_str: &str,
        user_id_str: &str,
        req: UpdateWorkspaceRequest,
    ) -> Result<WorkspaceResponse> {
        let ws_id = ObjectId::parse_str(workspace_id_str)?;
        let user_id = ObjectId::parse_str(user_id_str)?;

        let existing = self
            .workspace_repo
            .find_by_id(&ws_id, &user_id)
            .await?
            .ok_or_else(|| anyhow!("Workspace not found"))?;

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let env = req.env.unwrap_or(existing.env);
        let color = req.color.or(existing.color);
        let icon_type = req.icon_type.or(existing.icon_type);

        let updated = self
            .workspace_repo
            .update_workspace(
                &ws_id,
                &user_id,
                name,
                description,
                env,
                color,
                icon_type,
            )
            .await?
            .ok_or_else(|| anyhow!("Failed to update workspace"))?;

        let diagrams = self
            .diagram_repo
            .find_by_workspace(&ws_id, &user_id)
            .await?;

        Ok(WorkspaceResponse {
            id: ws_id.to_hex(),
            user_id: user_id_str.to_string(),
            name: updated.name,
            description: updated.description,
            env: updated.env,
            color: updated.color,
            icon_type: updated.icon_type,
            diagrams_count: diagrams.len() as i64,
            created_at: updated.created_at.to_string(),
            updated_at: updated.updated_at.to_string(),
        })
    }

    pub async fn delete_workspace(&self, workspace_id_str: &str, user_id_str: &str) -> Result<bool> {
        let ws_id = ObjectId::parse_str(workspace_id_str)?;
        let user_id = ObjectId::parse_str(user_id_str)?;
        self.workspace_repo.delete_workspace(&ws_id, &user_id).await
    }
}
