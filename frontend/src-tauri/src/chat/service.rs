use crate::chat::commands::AskQuestionResponse;
use crate::database::models::ChatMessage;
use crate::database::repositories::{
    chat_message::ChatMessagesRepository,
    meeting::MeetingsRepository,
    setting::SettingsRepository,
    summary::SummaryProcessesRepository,
    transcript::TranscriptsRepository,
};
use crate::summary::llm_client::{self, LLMProvider};
use chrono::Utc;
use log::{error as log_error, info as log_info};
use reqwest::Client;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

pub struct ChatService;

impl ChatService {
    /// Ask a question about a meeting with full context
    pub async fn ask_question<R: Runtime>(
        _app: AppHandle<R>,
        pool: SqlitePool,
        meeting_id: String,
        question: String,
        model_provider: String,
        model_name: String,
    ) -> Result<AskQuestionResponse, String> {
        log_info!("🚀 Starting ask_question for meeting_id: {}", meeting_id);

        // 1. Validate provider
        let provider = LLMProvider::from_str(&model_provider)?;

        // 2. Get API key from settings
        let api_key = Self::get_api_key(&pool, &provider).await?;

        // 3. Get Ollama endpoint if applicable
        let ollama_endpoint = if provider == LLMProvider::Ollama {
            SettingsRepository::get_ollama_endpoint(&pool)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        // 4. Get meeting details
        let meeting = MeetingsRepository::get_meeting(&pool, &meeting_id)
            .await
            .map_err(|e| format!("Failed to get meeting: {}", e))?
            .ok_or_else(|| format!("Meeting not found: {}", meeting_id))?;

        // 5. Get transcripts
        let transcripts = TranscriptsRepository::get_transcripts(&pool, &meeting_id)
            .await
            .map_err(|e| format!("Failed to get transcripts: {}", e))?;

        let transcript_text = transcripts
            .iter()
            .map(|t| t.transcript.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        // 6. Get summary if it exists
        let summary_data = SummaryProcessesRepository::get_summary_data_for_meeting(&pool, &meeting_id)
            .await
            .map_err(|e| format!("Failed to get summary: {}", e))?;

        let summary_text = if let Some(summary_process) = summary_data {
            if summary_process.status.to_lowercase() == "completed" {
                if let Some(result_str) = summary_process.result {
                    match serde_json::from_str::<serde_json::Value>(&result_str) {
                        Ok(parsed) => {
                            if let Some(markdown) = parsed.get("markdown").and_then(|m| m.as_str()) {
                                Some(markdown.to_string())
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 7. Get chat history
        let chat_history = ChatMessagesRepository::get_messages(&pool, &meeting_id)
            .await
            .map_err(|e| format!("Failed to get chat history: {}", e))?;

        // 8. Build context for LLM
        let context = Self::build_context(
            &meeting.title,
            &transcript_text,
            summary_text.as_deref(),
            &chat_history,
            &question,
        );

        // 9. Get current language from settings
        let language = Self::get_language(&pool).await.unwrap_or_else(|_| "pt-BR".to_string());

        // 10. Build system prompt
        let system_prompt = Self::build_system_prompt(&language);

        log_info!("📝 Context built, sending to LLM (provider: {}, model: {})", model_provider, model_name);

        // 11. Call LLM
        let client = Client::new();
        let answer = llm_client::generate_summary(
            &client,
            &provider,
            &model_name,
            &api_key,
            &system_prompt,
            &context,
            ollama_endpoint.as_deref(),
        )
        .await?;

        log_info!("✅ Received answer from LLM");

        // 12. Save user message
        let user_message = ChatMessagesRepository::save_message(
            &pool,
            &meeting_id,
            "user",
            &question,
            None,
        )
        .await
        .map_err(|e| format!("Failed to save user message: {}", e))?;

        // 13. Save assistant message
        let assistant_message = ChatMessagesRepository::save_message(
            &pool,
            &meeting_id,
            "assistant",
            &answer,
            None,
        )
        .await
        .map_err(|e| format!("Failed to save assistant message: {}", e))?;

        log_info!("💾 Messages saved to database");

        Ok(AskQuestionResponse {
            answer,
            user_message,
            assistant_message,
        })
    }

    /// Get API key for the specified provider from settings
    async fn get_api_key(pool: &SqlitePool, provider: &LLMProvider) -> Result<String, String> {
        let settings = SettingsRepository::get_settings(pool)
            .await
            .map_err(|e| format!("Failed to get settings: {}", e))?
            .ok_or_else(|| "No settings found. Please configure API keys in settings.".to_string())?;

        let api_key = match provider {
            LLMProvider::OpenAI => settings.openai_api_key,
            LLMProvider::Claude => settings.anthropic_api_key,
            LLMProvider::Groq => settings.groq_api_key,
            LLMProvider::Ollama => Some(String::new()), // Ollama doesn't need API key
            LLMProvider::OpenRouter => settings.open_router_api_key,
        };

        api_key.ok_or_else(|| {
            format!(
                "API key not found for provider: {:?}. Please add it in settings.",
                provider
            )
        })
    }

    /// Get current language setting
    async fn get_language(pool: &SqlitePool) -> Result<String, String> {
        // Try to get language from settings table
        let language: Result<Option<String>, _> = sqlx::query_scalar(
            "SELECT language FROM settings WHERE id = 'default'"
        )
        .fetch_optional(pool)
        .await;

        match language {
            Ok(Some(lang)) => Ok(lang),
            _ => Ok("pt-BR".to_string()), // Default to Portuguese Brazil
        }
    }

    /// Build system prompt based on language
    fn build_system_prompt(language: &str) -> String {
        let current_date = Utc::now().format("%Y-%m-%d").to_string();

        match language {
            "en" | "en-US" => {
                format!(
                    "You are an AI assistant helping users understand their meeting notes. \
                    Today's date is {}. You have access to the meeting transcript, summary, \
                    and previous conversation history. Answer questions accurately based on the \
                    context provided. If information is not in the context, say so clearly. \
                    Be concise but comprehensive in your responses. Respond in English.",
                    current_date
                )
            }
            _ => {
                // Default to Portuguese Brazil
                format!(
                    "Você é um assistente de IA ajudando usuários a entender suas anotações de reunião. \
                    A data de hoje é {}. Você tem acesso à transcrição da reunião, ao resumo, \
                    e ao histórico de conversas anteriores. Responda perguntas com precisão baseado no \
                    contexto fornecido. Se a informação não estiver no contexto, deixe isso claro. \
                    Seja conciso mas abrangente em suas respostas. Responda em português do Brasil.",
                    current_date
                )
            }
        }
    }

    /// Build context for LLM with all available information
    fn build_context(
        meeting_title: &str,
        transcript: &str,
        summary: Option<&str>,
        chat_history: &[ChatMessage],
        current_question: &str,
    ) -> String {
        let mut context = String::new();

        // Meeting title
        context.push_str(&format!("# Meeting Title\n{}\n\n", meeting_title));

        // Transcript
        context.push_str("# Transcript\n");
        context.push_str(transcript);
        context.push_str("\n\n");

        // Summary if available
        if let Some(summary_text) = summary {
            context.push_str("# Summary\n");
            context.push_str(summary_text);
            context.push_str("\n\n");
        }

        // Chat history if exists
        if !chat_history.is_empty() {
            context.push_str("# Previous Conversation\n");
            for msg in chat_history {
                let role_label = if msg.role == "user" { "User" } else { "Assistant" };
                context.push_str(&format!("{}: {}\n\n", role_label, msg.content));
            }
        }

        // Current question
        context.push_str(&format!("# Current Question\n{}", current_question));

        context
    }
}
