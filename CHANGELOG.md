# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

## [2025-12-16]

### Changed - Transcription Engine Simplification

#### Removed Whisper (whisper-rs) - Now Using Parakeet Only

**Why this change was made:**
- Whisper (whisper-rs) required GPU SDKs (Vulkan, CUDA, Metal) at build time
- Windows CI builds were failing due to Vulkan SDK installation issues (~50+ minute downloads, broken Chocolatey packages)
- Parakeet provides significantly better performance:
  - **15x faster** transcription (RTFx 3386 vs 216)
  - **Better accuracy** (6.05% WER vs 10-12% WER)
  - **Better for noisy audio** (21.58% WER vs 29.80%)
  - **Smaller model size** (~670MB vs ~3GB for Whisper Large)

**Technical changes:**
- Removed `whisper-rs` dependency from `Cargo.toml`
- Disabled `whisper_engine` module in `lib.rs` (code kept for future reference)
- Updated `TranscriptionEngine` enum to only include Parakeet variant
- Removed Vulkan SDK and Windows build dependencies from GitHub Actions workflow
- Updated UI (`TranscriptSettings.tsx`) to show only Parakeet option

**Benefits:**
- Simpler builds: No GPU SDKs required at build time
- Faster CI: Removed ~10 minutes of Windows dependency installation
- Cross-platform: ONNX Runtime handles GPU detection at runtime
- Better user experience: Parakeet downloads model once (~670MB), works immediately

**Migration notes:**
- Users with `localWhisper` config will automatically use Parakeet
- Whisper models can be deleted from the models folder
- No user action required - Parakeet model will auto-download on first use

### Technical Details
- **Transcription Engine**: Parakeet (NVIDIA NeMo) via ONNX Runtime
- **Model**: parakeet-tdt-0.6b-v3-int8 (~670MB)
- **GPU Support**: Automatic via ONNX Runtime (CoreML on macOS, DirectML/CUDA on Windows)
- **Languages**: English and Portuguese (tested and working)

## [2025-11-17]

### Added
- Tab-based interface for Meeting Notes (Chat and Summary tabs)
- Chat tab as default view (core business focus)
- Full-height layout for both Chat and Summary tabs

### Changed
- Reorganized SummaryPanel with tab navigation system
- Moved MeetingChat to dedicated full-screen tab
- Increased chat message font size from `text-sm` to `text-base` for better readability

### Fixed
- Tauri API import error in MeetingChat component (changed from `@tauri-apps/api/tauri` to `@tauri-apps/api/core`)
- Rust compilation errors in chat/service.rs (missing repository functions)
- Unused import warnings in chat_message.rs and notifications/commands.rs
- Unused variable warning in api.rs

## [Previous Releases]

### Added

#### Documentation
- 439 new lines in CLAUDE.md with comprehensive documentation covering:
  - Complete Generate Summary System documentation
  - Audio Processing Pipeline details
  - Tauri architecture and communication patterns
  - Whisper model management
  - Development commands and workflows

#### Internationalization (i18n)
- i18n infrastructure with next-i18next configuration
- Translation files for English and Portuguese (pt-BR)
- New LanguageSelector component for runtime language switching
- I18nProvider context for global language state management
- Translations for all major UI components:
  - About component
  - ModelSettingsModal
  - RecordingControls
  - RecordingSettings
  - SettingTabs
  - SummaryGeneratorButtonGroup
  - SummaryModelSettings
  - TranscriptSettings

#### Backend
- Database migration: Added language field support (20251113000000_add_language_field.sql)
- API endpoints for language management
- Settings repository methods for language persistence
- Language-aware prompt generation system

#### Summary System
- Language-aware prompts for AI summary generation
- Support for generating summaries in Portuguese (pt-BR) and English
- Dynamic prompt selection based on user language preference
- Enhanced prompt templates with multilingual support

### Changed
- App layout now includes I18nProvider wrapper
- Summary processor upgraded with language parameter support
- LLM client enhanced with language-aware prompt generation
- Summary service integrated with user language preferences

### Technical Details
- **Frontend**: Next.js with next-i18next, React context for i18n state
- **Backend**: SQLite schema update, Rust API endpoints for settings
- **Language Support**: English (en) and Portuguese (pt-BR)
- **Database**: New `language` field in settings table with default 'en'
