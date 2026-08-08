use crate::models::workspace_model::Workspace;
use anyhow::Result;
use bson::{doc, oid::ObjectId, DateTime};
use futures::TryStreamExt;
use mongodb::{Collection, Database};

#[derive(Clone)]
pub struct WorkspaceRepository {
    workspace_collection: Collection<Workspace>,
}

impl WorkspaceRepository {
    pub fn new(db: &Database) -> Self {
        let workspace_collection = db.collection::<Workspace>("workspaces");
        Self {
            workspace_collection,
        }
    }

    pub async fn create_workspace(&self, mut workspace: Workspace) -> Result<Workspace> {
        let insert_result = self.workspace_collection.insert_one(&workspace).await?;
        if let Some(inserted_id) = insert_result.inserted_id.as_object_id() {
            workspace.id = Some(inserted_id);
        }
        Ok(workspace)
    }

    pub async fn find_by_user(&self, user_id: &ObjectId) -> Result<Vec<Workspace>> {
        let filter = doc! { "user_id": user_id };
        let mut cursor = self.workspace_collection.find(filter).await?;
        let mut workspaces = Vec::new();
        while let Some(ws) = cursor.try_next().await? {
            workspaces.push(ws);
        }
        Ok(workspaces)
    }

    pub async fn find_by_id(&self, id: &ObjectId, user_id: &ObjectId) -> Result<Option<Workspace>> {
        let filter = doc! { "_id": id, "user_id": user_id };
        let workspace = self.workspace_collection.find_one(filter).await?;
        Ok(workspace)
    }

    pub async fn update_workspace(
        &self,
        id: &ObjectId,
        user_id: &ObjectId,
        name: String,
        description: Option<String>,
        env: String,
        color: Option<String>,
        icon_type: Option<String>,
    ) -> Result<Option<Workspace>> {
        let filter = doc! { "_id": id, "user_id": user_id };
        let now = DateTime::now();
        let update = doc! {
            "$set": {
                "name": name,
                "description": description,
                "env": env,
                "color": color,
                "icon_type": icon_type,
                "updated_at": now
            }
        };

        self.workspace_collection.update_one(filter.clone(), update).await?;
        self.find_by_id(id, user_id).await
    }

    pub async fn delete_workspace(&self, id: &ObjectId, user_id: &ObjectId) -> Result<bool> {
        let filter = doc! { "_id": id, "user_id": user_id };
        let result = self.workspace_collection.delete_one(filter).await?;
        Ok(result.deleted_count > 0)
    }
}
