use crate::models::diagram_model::Diagram;
use anyhow::Result;
use bson::{doc, oid::ObjectId, DateTime};
use futures::TryStreamExt;
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};

#[derive(Clone)]
pub struct DiagramRepository {
    diagram_collection: Collection<Diagram>,
}

impl DiagramRepository {
    pub fn new(db: &Database) -> Self {
        let diagram_collection = db.collection::<Diagram>("diagrams");
        Self { diagram_collection }
    }

    pub async fn create_diagram(&self, mut diagram: Diagram) -> Result<Diagram> {
        let insert_result = self.diagram_collection.insert_one(&diagram).await?;
        if let Some(inserted_id) = insert_result.inserted_id.as_object_id() {
            diagram.id = Some(inserted_id);
        }
        Ok(diagram)
    }

    pub async fn find_by_workspace(
        &self,
        workspace_id: &ObjectId,
        user_id: &ObjectId,
    ) -> Result<Vec<Diagram>> {
        let filter = doc! { "workspace_id": workspace_id, "user_id": user_id };
        let mut cursor = self.diagram_collection.find(filter).await?;
        let mut diagrams = Vec::new();
        while let Some(d) = cursor.try_next().await? {
            diagrams.push(d);
        }
        Ok(diagrams)
    }

    pub async fn find_recent_by_user(
        &self,
        user_id: &ObjectId,
        limit: i64,
    ) -> Result<Vec<Diagram>> {
        let filter = doc! { "user_id": user_id };
        let find_options = FindOptions::builder()
            .sort(doc! { "updated_at": -1 })
            .limit(limit)
            .build();

        let mut cursor = self.diagram_collection.find(filter).with_options(find_options).await?;
        let mut diagrams = Vec::new();
        while let Some(d) = cursor.try_next().await? {
            diagrams.push(d);
        }
        Ok(diagrams)
    }

    pub async fn find_by_id(&self, id: &ObjectId, user_id: &ObjectId) -> Result<Option<Diagram>> {
        let filter = doc! { "_id": id, "user_id": user_id };
        let diagram = self.diagram_collection.find_one(filter).await?;
        Ok(diagram)
    }

    pub async fn find_by_id_public(&self, id: &ObjectId) -> Result<Option<Diagram>> {
        let filter = doc! { "_id": id };
        let diagram = self.diagram_collection.find_one(filter).await?;
        Ok(diagram)
    }

    pub async fn update_diagram(
        &self,
        id: &ObjectId,
        user_id: &ObjectId,
        title: String,
        description: Option<String>,
        version: String,
        nodes: serde_json::Value,
        edges: serde_json::Value,
        configs: serde_json::Value,
        viewport: Option<serde_json::Value>,
    ) -> Result<Option<Diagram>> {
        let filter = doc! { "_id": id, "user_id": user_id };
        let now = DateTime::now();
        let update = doc! {
            "$set": {
                "title": title,
                "description": description,
                "version": version,
                "nodes": bson::to_bson(&nodes)?,
                "edges": bson::to_bson(&edges)?,
                "configs": bson::to_bson(&configs)?,
                "viewport": bson::to_bson(&viewport)?,
                "updated_at": now
            }
        };

        self.diagram_collection.update_one(filter, update).await?;
        self.find_by_id(id, user_id).await
    }

    pub async fn delete_diagram(&self, id: &ObjectId, user_id: &ObjectId) -> Result<bool> {
        let filter = doc! { "_id": id, "user_id": user_id };
        let result = self.diagram_collection.delete_one(filter).await?;
        Ok(result.deleted_count > 0)
    }
}
