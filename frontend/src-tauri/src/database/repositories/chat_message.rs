use crate::database::models::ChatMessage;
use chrono::Utc;
use sqlx::{Error as SqlxError, SqlitePool};
use tracing::info;
use uuid::Uuid;

pub struct ChatMessagesRepository;

impl ChatMessagesRepository {
    /// Get all chat messages for a meeting, ordered by created_at
    pub async fn get_messages(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<Vec<ChatMessage>, SqlxError> {
        if meeting_id.trim().is_empty() {
            return Err(SqlxError::Protocol(
                "meeting_id cannot be empty".to_string(),
            ));
        }

        let messages = sqlx::query_as::<_, ChatMessage>(
            "SELECT * FROM chat_messages WHERE meeting_id = ? ORDER BY created_at ASC",
        )
        .bind(meeting_id)
        .fetch_all(pool)
        .await?;

        Ok(messages)
    }

    /// Save a new chat message
    pub async fn save_message(
        pool: &SqlitePool,
        meeting_id: &str,
        role: &str,
        content: &str,
        metadata: Option<String>,
    ) -> Result<ChatMessage, SqlxError> {
        if meeting_id.trim().is_empty() {
            return Err(SqlxError::Protocol(
                "meeting_id cannot be empty".to_string(),
            ));
        }

        if role != "user" && role != "assistant" {
            return Err(SqlxError::Protocol(
                "role must be 'user' or 'assistant'".to_string(),
            ));
        }

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        sqlx::query(
            "INSERT INTO chat_messages (id, meeting_id, role, content, created_at, metadata)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(meeting_id)
        .bind(role)
        .bind(content)
        .bind(created_at)
        .bind(&metadata)
        .execute(pool)
        .await?;

        info!(
            "Saved chat message (role: {}) for meeting_id: {}",
            role, meeting_id
        );

        Ok(ChatMessage {
            id,
            meeting_id: meeting_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at,
            metadata,
        })
    }

    /// Delete all chat messages for a meeting
    pub async fn delete_messages(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<u64, SqlxError> {
        if meeting_id.trim().is_empty() {
            return Err(SqlxError::Protocol(
                "meeting_id cannot be empty".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM chat_messages WHERE meeting_id = ?")
            .bind(meeting_id)
            .execute(pool)
            .await?;

        let rows_affected = result.rows_affected();
        info!(
            "Deleted {} chat messages for meeting_id: {}",
            rows_affected, meeting_id
        );

        Ok(rows_affected)
    }

    /// Delete a specific chat message by id
    pub async fn delete_message(pool: &SqlitePool, message_id: &str) -> Result<bool, SqlxError> {
        if message_id.trim().is_empty() {
            return Err(SqlxError::Protocol(
                "message_id cannot be empty".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM chat_messages WHERE id = ?")
            .bind(message_id)
            .execute(pool)
            .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted chat message: {}", message_id);
        }

        Ok(deleted)
    }

    /// Count messages for a meeting
    pub async fn count_messages(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<i64, SqlxError> {
        if meeting_id.trim().is_empty() {
            return Err(SqlxError::Protocol(
                "meeting_id cannot be empty".to_string(),
            ));
        }

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM chat_messages WHERE meeting_id = ?",
        )
        .bind(meeting_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }
}
