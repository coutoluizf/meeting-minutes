// audio/transcription/engine.rs
//
// TranscriptionEngine enum and model initialization/validation logic.
//
// NOTE: Whisper engine has been removed from Meetily.
// Parakeet (ONNX-based) is now the sole transcription engine.
// This simplifies builds by eliminating GPU SDK dependencies (Vulkan, CUDA, Metal).
// Parakeet provides 15x faster transcription with better accuracy (6% vs 10% WER).

use super::provider::TranscriptionProvider;
use log::{info, warn};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};

// ============================================================================
// TRANSCRIPTION ENGINE ENUM
// ============================================================================

// Transcription engine abstraction - now using Parakeet exclusively
// NOTE: Whisper variant was removed because whisper-rs requires GPU SDKs at build time
pub enum TranscriptionEngine {
    // NOTE: Whisper variant removed - kept comment for reference
    // Whisper(Arc<crate::whisper_engine::WhisperEngine>),
    Parakeet(Arc<crate::parakeet_engine::ParakeetEngine>), // Primary and only engine
    Provider(Arc<dyn TranscriptionProvider>),  // Trait-based (for extensibility)
}

impl TranscriptionEngine {
    /// Check if the engine has a model loaded
    pub async fn is_model_loaded(&self) -> bool {
        match self {
            Self::Parakeet(engine) => engine.is_model_loaded().await,
            Self::Provider(provider) => provider.is_model_loaded().await,
        }
    }

    /// Get the current model name
    pub async fn get_current_model(&self) -> Option<String> {
        match self {
            Self::Parakeet(engine) => engine.get_current_model().await,
            Self::Provider(provider) => provider.get_current_model().await,
        }
    }

    /// Get the provider name for logging
    pub fn provider_name(&self) -> &str {
        match self {
            Self::Parakeet(_) => "Parakeet (ONNX)",
            Self::Provider(provider) => provider.provider_name(),
        }
    }
}

// ============================================================================
// MODEL VALIDATION AND INITIALIZATION
// ============================================================================

/// Validate that Parakeet transcription model is ready before starting recording
/// NOTE: Whisper support was removed - Parakeet is now the only transcription engine
pub async fn validate_transcription_model_ready<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // Check transcript configuration - but always use Parakeet regardless of saved config
    let _config = match crate::api::api::api_get_transcript_config(
        app.clone(),
        app.clone().state(),
        None,
    )
    .await
    {
        Ok(Some(config)) => {
            info!(
                "📝 Found transcript config - provider: {}, model: {}",
                config.provider, config.model
            );
            // If old config has localWhisper, we'll use parakeet instead
            if config.provider == "localWhisper" {
                warn!("⚠️ localWhisper is no longer supported, using Parakeet instead");
            }
            config
        }
        Ok(None) => {
            info!("📝 No transcript config found, defaulting to Parakeet");
            crate::api::api::TranscriptConfig {
                provider: "parakeet".to_string(),
                model: "parakeet-tdt-0.6b-v3-int8".to_string(),
                api_key: None,
            }
        }
        Err(e) => {
            warn!("⚠️ Failed to get transcript config: {}, defaulting to Parakeet", e);
            crate::api::api::TranscriptConfig {
                provider: "parakeet".to_string(),
                model: "parakeet-tdt-0.6b-v3-int8".to_string(),
                api_key: None,
            }
        }
    };

    // Always validate Parakeet (even if config says localWhisper, we use Parakeet now)
    info!("🦜 Validating Parakeet model...");

    // Ensure parakeet engine is initialized first
    if let Err(init_error) = crate::parakeet_engine::commands::parakeet_init().await {
        warn!("❌ Failed to initialize Parakeet engine: {}", init_error);
        return Err(format!(
            "Failed to initialize Parakeet speech recognition: {}",
            init_error
        ));
    }

    // Use the validation command that includes auto-discovery and loading
    match crate::parakeet_engine::commands::parakeet_validate_model_ready_with_config(app).await {
        Ok(model_name) => {
            info!("✅ Parakeet model validation successful: {} is ready", model_name);
            Ok(())
        }
        Err(e) => {
            warn!("❌ Parakeet model validation failed: {}", e);
            Err(e)
        }
    }
}

/// Get or initialize the Parakeet transcription engine
/// NOTE: Whisper support was removed - this function now only returns Parakeet engine
pub async fn get_or_init_transcription_engine<R: Runtime>(
    _app: &AppHandle<R>,
) -> Result<TranscriptionEngine, String> {
    // Always use Parakeet engine (Whisper support was removed)
    info!("🦜 Initializing Parakeet transcription engine");

    // Get Parakeet engine
    let engine = {
        let guard = crate::parakeet_engine::commands::PARAKEET_ENGINE
            .lock()
            .unwrap();
        guard.as_ref().cloned()
    };

    match engine {
        Some(engine) => {
            // Check if model is loaded
            if engine.is_model_loaded().await {
                let model_name = engine.get_current_model().await
                    .unwrap_or_else(|| "unknown".to_string());
                info!("✅ Parakeet model '{}' already loaded", model_name);
                Ok(TranscriptionEngine::Parakeet(engine))
            } else {
                Err("Parakeet engine initialized but no model loaded. Please download a Parakeet model from settings.".to_string())
            }
        }
        None => {
            Err("Parakeet engine not initialized. This should not happen after validation.".to_string())
        }
    }
}

// =============================================================================
// NOTE: Whisper engine support was removed from Meetily
// =============================================================================
// The get_or_init_whisper function has been removed because:
// 1. whisper-rs requires GPU SDKs (Vulkan, CUDA, Metal) at build time
// 2. This caused Windows CI builds to fail (Vulkan SDK installation issues)
// 3. Parakeet provides faster (15x) and more accurate (6% vs 10% WER) transcription
// 4. Parakeet uses ONNX Runtime which handles GPU detection at runtime
//
// If you need to restore Whisper support in the future:
// 1. Re-enable whisper-rs in Cargo.toml
// 2. Uncomment whisper_engine module in lib.rs
// 3. Restore the TranscriptionEngine::Whisper variant
// 4. Restore the get_or_init_whisper function from git history
// =============================================================================
