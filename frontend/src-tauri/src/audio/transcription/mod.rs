// audio/transcription/mod.rs
//
// Transcription module: Provider abstraction, engine management, and worker pool.
//
// NOTE: Whisper provider was removed - using Parakeet only
// Whisper (whisper-rs) required GPU SDKs at build time, which caused Windows CI issues.

pub mod provider;
// NOTE: whisper_provider module disabled - using parakeet_provider only
// pub mod whisper_provider;
pub mod parakeet_provider;
pub mod engine;
pub mod worker;

// Re-export commonly used types
pub use provider::{TranscriptionError, TranscriptionProvider, TranscriptResult};
// NOTE: WhisperProvider removed - using ParakeetProvider only
// pub use whisper_provider::WhisperProvider;
pub use parakeet_provider::ParakeetProvider;
pub use engine::{
    TranscriptionEngine,
    validate_transcription_model_ready,
    get_or_init_transcription_engine,
    // NOTE: get_or_init_whisper removed - using Parakeet only
};
pub use worker::{
    start_transcription_task,
    reset_speech_detected_flag,
    TranscriptUpdate
};
