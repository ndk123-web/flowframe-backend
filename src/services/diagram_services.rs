use crate::dtos::diagram_dto::{
    CreateDiagramRequest, DiagramResponse, RecentDiagramResponse, UpdateDiagramRequest,
};
use crate::models::diagram_model::Diagram;
use crate::repositories::diagram_repository::DiagramRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use anyhow::{anyhow, Result};
use bson::oid::ObjectId;
use bson::DateTime;

#[derive(Clone)]
pub struct DiagramService {
    diagram_repo: DiagramRepository,
    workspace_repo: WorkspaceRepository,
}

impl DiagramService {
    pub fn new(diagram_repo: DiagramRepository, workspace_repo: WorkspaceRepository) -> Self {
        Self {
            diagram_repo,
            workspace_repo,
        }
    }

    pub async fn create_diagram(
        &self,
        workspace_id_str: &str,
        user_id_str: &str,
        req: CreateDiagramRequest,
    ) -> Result<DiagramResponse> {
        let ws_id = ObjectId::parse_str(workspace_id_str)?;
        let user_id = ObjectId::parse_str(user_id_str)?;

        // Ensure workspace belongs to this user
        let _ws = self
            .workspace_repo
            .find_by_id(&ws_id, &user_id)
            .await?
            .ok_or_else(|| anyhow!("Workspace not found"))?;

        // Enforce 5 diagram limit per workspace
        let existing = self.diagram_repo.find_by_workspace(&ws_id, &user_id).await?;
        if existing.len() >= 5 {
            return Err(anyhow!("Diagram limit reached (5/5) for this workspace"));
        }

        let now = DateTime::now();
        let default_nodes = serde_json::json!([]);
        let default_edges = serde_json::json!([]);
        let default_configs = serde_json::json!({});

        let nodes = req.nodes.unwrap_or(default_nodes);
        let edges = req.edges.unwrap_or(default_edges);
        let configs = req.configs.unwrap_or(default_configs);

        let new_diagram = Diagram {
            id: None,
            workspace_id: ws_id,
            user_id,
            title: req.title,
            description: req.description,
            version: req.version.unwrap_or_else(|| "1.0".to_string()),
            nodes: nodes.clone(),
            edges: edges.clone(),
            configs: configs.clone(),
            viewport: req.viewport.clone(),
            created_at: now,
            updated_at: now,
        };

        let created = self.diagram_repo.create_diagram(new_diagram).await?;
        let d_id = created.id.unwrap().to_hex();

        let nodes_count = nodes.as_array().map(|a| a.len()).unwrap_or(0);
        let edges_count = edges.as_array().map(|a| a.len()).unwrap_or(0);

        Ok(DiagramResponse {
            id: d_id,
            workspace_id: workspace_id_str.to_string(),
            user_id: user_id_str.to_string(),
            title: created.title,
            description: created.description,
            version: created.version,
            nodes,
            edges,
            configs,
            viewport: created.viewport,
            nodes_count,
            edges_count,
            created_at: created.created_at.to_string(),
            updated_at: created.updated_at.to_string(),
        })
    }

    pub async fn get_workspace_diagrams(
        &self,
        workspace_id_str: &str,
        user_id_str: &str,
    ) -> Result<Vec<DiagramResponse>> {
        let ws_id = ObjectId::parse_str(workspace_id_str)?;
        let user_id = ObjectId::parse_str(user_id_str)?;

        let diagrams = self.diagram_repo.find_by_workspace(&ws_id, &user_id).await?;

        let mut responses = Vec::new();
        for d in diagrams {
            let nodes_count = d.nodes.as_array().map(|a| a.len()).unwrap_or(0);
            let edges_count = d.edges.as_array().map(|a| a.len()).unwrap_or(0);

            responses.push(DiagramResponse {
                id: d.id.unwrap().to_hex(),
                workspace_id: workspace_id_str.to_string(),
                user_id: user_id_str.to_string(),
                title: d.title,
                description: d.description,
                version: d.version,
                nodes: d.nodes,
                edges: d.edges,
                configs: d.configs,
                viewport: d.viewport,
                nodes_count,
                edges_count,
                created_at: d.created_at.to_string(),
                updated_at: d.updated_at.to_string(),
            });
        }

        Ok(responses)
    }

    pub async fn get_diagram_by_id(
        &self,
        diagram_id_str: &str,
        user_id_str: &str,
    ) -> Result<DiagramResponse> {
        let d_id = ObjectId::parse_str(diagram_id_str)?;
        let user_id = ObjectId::parse_str(user_id_str)?;

        let d = self
            .diagram_repo
            .find_by_id(&d_id, &user_id)
            .await?
            .ok_or_else(|| anyhow!("Diagram not found"))?;

        let nodes_count = d.nodes.as_array().map(|a| a.len()).unwrap_or(0);
        let edges_count = d.edges.as_array().map(|a| a.len()).unwrap_or(0);

        Ok(DiagramResponse {
            id: d.id.unwrap().to_hex(),
            workspace_id: d.workspace_id.to_hex(),
            user_id: user_id_str.to_string(),
            title: d.title,
            description: d.description,
            version: d.version,
            nodes: d.nodes,
            edges: d.edges,
            configs: d.configs,
            viewport: d.viewport,
            nodes_count,
            edges_count,
            created_at: d.created_at.to_string(),
            updated_at: d.updated_at.to_string(),
        })
    }

    pub async fn update_diagram(
        &self,
        diagram_id_str: &str,
        user_id_str: &str,
        req: UpdateDiagramRequest,
    ) -> Result<DiagramResponse> {
        let d_id = ObjectId::parse_str(diagram_id_str)?;
        let user_id = ObjectId::parse_str(user_id_str)?;

        let existing = self
            .diagram_repo
            .find_by_id(&d_id, &user_id)
            .await?
            .ok_or_else(|| anyhow!("Diagram not found"))?;

        let title = req.title.unwrap_or(existing.title);
        let description = req.description.or(existing.description);
        let version = req.version.unwrap_or(existing.version);
        let nodes = req.nodes.unwrap_or(existing.nodes);
        let edges = req.edges.unwrap_or(existing.edges);
        let configs = req.configs.unwrap_or(existing.configs);
        let viewport = req.viewport.or(existing.viewport);

        let updated = self
            .diagram_repo
            .update_diagram(
                &d_id,
                &user_id,
                title,
                description,
                version,
                nodes.clone(),
                edges.clone(),
                configs.clone(),
                viewport.clone(),
            )
            .await?
            .ok_or_else(|| anyhow!("Failed to update diagram"))?;

        let nodes_count = nodes.as_array().map(|a| a.len()).unwrap_or(0);
        let edges_count = edges.as_array().map(|a| a.len()).unwrap_or(0);

        Ok(DiagramResponse {
            id: d_id.to_hex(),
            workspace_id: updated.workspace_id.to_hex(),
            user_id: user_id_str.to_string(),
            title: updated.title,
            description: updated.description,
            version: updated.version,
            nodes,
            edges,
            configs,
            viewport,
            nodes_count,
            edges_count,
            created_at: updated.created_at.to_string(),
            updated_at: updated.updated_at.to_string(),
        })
    }

    pub async fn delete_diagram(&self, diagram_id_str: &str, user_id_str: &str) -> Result<bool> {
        let d_id = ObjectId::parse_str(diagram_id_str)?;
        let user_id = ObjectId::parse_str(user_id_str)?;
        self.diagram_repo.delete_diagram(&d_id, &user_id).await
    }

    pub async fn get_recent_diagrams(
        &self,
        user_id_str: &str,
        limit: i64,
    ) -> Result<Vec<RecentDiagramResponse>> {
        let user_id = ObjectId::parse_str(user_id_str)?;
        let diagrams = self.diagram_repo.find_recent_by_user(&user_id, limit).await?;

        let mut responses = Vec::new();
        for d in diagrams {
            let ws_name = match self.workspace_repo.find_by_id(&d.workspace_id, &user_id).await? {
                Some(ws) => ws.name,
                None => "Workspace".to_string(),
            };
            let ws_env = match self.workspace_repo.find_by_id(&d.workspace_id, &user_id).await? {
                Some(ws) => ws.env,
                None => "DEV".to_string(),
            };

            let nodes_count = d.nodes.as_array().map(|a| a.len()).unwrap_or(0);

            responses.push(RecentDiagramResponse {
                id: d.id.unwrap().to_hex(),
                workspace_id: d.workspace_id.to_hex(),
                workspace_name: ws_name,
                title: d.title,
                env: ws_env,
                nodes_count,
                updated_at: d.updated_at.to_string(),
            });
        }

        Ok(responses)
    }
}
