use crate::models::ai_model::{AiChatMessage, AiUsage};
use anyhow::Result;
use bson::doc;
use bson::oid::ObjectId;
use bson::DateTime;
use futures::TryStreamExt;
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::{Collection, Database};

#[derive(Clone)]
pub struct AiRepository {
    usage_collection: Collection<AiUsage>,
    message_collection: Collection<AiChatMessage>,
}

impl AiRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            usage_collection: db.collection::<AiUsage>("ai_usages"),
            message_collection: db.collection::<AiChatMessage>("ai_messages"),
        }
    }

    /// Read current usage count for a user
    pub async fn get_usage_count(&self, user_id: &ObjectId) -> Result<i32> {
        let res = self
            .usage_collection
            .find_one(doc! { "user_id": user_id })
            .await?;
        Ok(res.map(|u| u.request_count).unwrap_or(0))
    }

    /// Check if user is eligible to send an AI request (< max_limit) without incrementing
    pub async fn check_can_request(&self, user_id: &ObjectId, max_limit: i32) -> Result<bool> {
        let count = self.get_usage_count(user_id).await?;
        Ok(count < max_limit)
    }

    /// Atomically increments user usage count by 1 (ONLY called AFTER AI call succeeds).
    /// Returns Some(updated_usage) if successful, or None if the limit is reached.
    pub async fn check_and_increment_usage(
        &self,
        user_id: &ObjectId,
        max_limit: i32,
    ) -> Result<Option<AiUsage>> {
        let now = DateTime::now();

        // 1. Try to atomically find and increment if request_count < max_limit
        let filter = doc! {
            "user_id": user_id,
            "request_count": { "$lt": max_limit }
        };
        let update = doc! {
            "$inc": { "request_count": 1 },
            "$set": { "updated_at": now }
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        if let Some(updated) = self
            .usage_collection
            .find_one_and_update(filter, update)
            .with_options(options)
            .await?
        {
            return Ok(Some(updated));
        }

        // 2. If not updated, check if document exists at all
        let existing = self
            .usage_collection
            .find_one(doc! { "user_id": user_id })
            .await?;

        if existing.is_none() {
            // User has no record yet -> Insert initial usage with 1
            let new_usage = AiUsage {
                id: None,
                user_id: *user_id,
                request_count: 1,
                created_at: now,
                updated_at: now,
            };
            let insert_res = self.usage_collection.insert_one(&new_usage).await?;
            let mut created = new_usage;
            created.id = insert_res.inserted_id.as_object_id();
            return Ok(Some(created));
        }

        // 3. User already has request_count >= max_limit
        Ok(None)
    }

    /// Decrement usage count (refund) if any failure occurred after incrementing
    pub async fn decrement_usage(&self, user_id: &ObjectId) -> Result<()> {
        let now = DateTime::now();
        let filter = doc! {
            "user_id": user_id,
            "request_count": { "$gt": 0 }
        };
        let update = doc! {
            "$inc": { "request_count": -1 },
            "$set": { "updated_at": now }
        };
        self.usage_collection.update_one(filter, update).await?;
        Ok(())
    }

    /// Reset usage count to 0 for a user
    pub async fn reset_usage(&self, user_id: &ObjectId) -> Result<()> {
        let now = DateTime::now();
        let filter = doc! { "user_id": user_id };
        let update = doc! {
            "$set": { "request_count": 0, "updated_at": now }
        };
        self.usage_collection.update_one(filter, update).await?;
        Ok(())
    }

    /// Save an AI conversation message (user or assistant)
    pub async fn save_message(&self, msg: &AiChatMessage) -> Result<()> {
        self.message_collection.insert_one(msg).await?;
        Ok(())
    }

    /// Fetch recent isolated messages for a specific workspace and diagram
    pub async fn find_recent_history(
        &self,
        workspace_id: &ObjectId,
        diagram_id: &ObjectId,
        user_id: &ObjectId,
        limit: i64,
    ) -> Result<Vec<AiChatMessage>> {
        let filter = doc! {
            "workspace_id": workspace_id,
            "diagram_id": diagram_id,
            "user_id": user_id
        };
        let find_options = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .build();

        let mut cursor = self
            .message_collection
            .find(filter)
            .with_options(find_options)
            .await?;
        let mut messages = Vec::new();

        while let Some(msg) = cursor.try_next().await? {
            messages.push(msg);
        }

        // Reverse so chronological order (oldest to newest)
        messages.reverse();
        Ok(messages)
    }
}
