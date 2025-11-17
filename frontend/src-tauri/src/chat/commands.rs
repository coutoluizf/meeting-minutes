use crate::chat::service::ChatService;
use crate::database::models::ChatMessage;
use crate::database::repositories::chat_message::ChatMessagesRepository;
use crate::state::AppState;
use log::{error as log_error, info as log_info};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

#[derive(Debug, Serialize, Deserialize)]
pub struct AskQuestionRequest {
    pub meeting_id: String,
    pub question: String,
    pub model_provider: String,
    pub model_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AskQuestionResponse {
    pub answer: String,
    pub user_message: ChatMessage,
    pub assistant_message: ChatMessage,
}

/// Get all chat messages for a meeting
#[tauri::command]
pub async fn api_get_chat_messages<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    meeting_id: String,
) -> Result<Vec<ChatMessage>, String> {
    log_info!("api_get_chat_messages called for meeting_id: {}", meeting_id);
    let pool = state.db_manager.pool();

    match ChatMessagesRepository::get_messages(pool, &meeting_id).await {
        Ok(messages) => {
            log_info!("Retrieved {} messages for meeting_id: {}", messages.len(), meeting_id);
            Ok(messages)
        }
        Err(e) => {
            log_error!("Failed to get messages for {}: {}", meeting_id, e);
            Err(format!("Failed to retrieve chat messages: {}", e))
        }
    }
}

/// Save a chat message
#[tauri::command]
pub async fn api_save_chat_message<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    meeting_id: String,
    role: String,
    content: String,
    metadata: Option<String>,
) -> Result<ChatMessage, String> {
    log_info!("api_save_chat_message called for meeting_id: {} (role: {})", meeting_id, role);
    let pool = state.db_manager.pool();

    match ChatMessagesRepository::save_message(pool, &meeting_id, &role, &content, metadata).await {
        Ok(message) => {
            log_info!("Saved message {} for meeting_id: {}", message.id, meeting_id);
            Ok(message)
        }
        Err(e) => {
            log_error!("Failed to save message for {}: {}", meeting_id, e);
            Err(format!("Failed to save chat message: {}", e))
        }
    }
}

/// Delete all chat messages for a meeting
#[tauri::command]
pub async fn api_delete_chat_messages<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    meeting_id: String,
) -> Result<u64, String> {
    log_info!("api_delete_chat_messages called for meeting_id: {}", meeting_id);
    let pool = state.db_manager.pool();

    match ChatMessagesRepository::delete_messages(pool, &meeting_id).await {
        Ok(count) => {
            log_info!("Deleted {} messages for meeting_id: {}", count, meeting_id);
            Ok(count)
        }
        Err(e) => {
            log_error!("Failed to delete messages for {}: {}", meeting_id, e);
            Err(format!("Failed to delete chat messages: {}", e))
        }
    }
}

/// Ask a question about the meeting with full context
#[tauri::command]
pub async fn api_ask_question<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    meeting_id: String,
    question: String,
    model_provider: String,
    model_name: String,
) -> Result<AskQuestionResponse, String> {
    log_info!(
        "api_ask_question called for meeting_id: {} with provider: {}",
        meeting_id,
        model_provider
    );
    let pool = state.db_manager.pool().clone();

    match ChatService::ask_question(
        app,
        pool,
        meeting_id,
        question,
        model_provider,
        model_name,
    )
    .await
    {
        Ok(response) => {
            log_info!("Question answered successfully");
            Ok(response)
        }
        Err(e) => {
            log_error!("Failed to answer question: {}", e);
            Err(format!("Failed to answer question: {}", e))
        }
    }
}
