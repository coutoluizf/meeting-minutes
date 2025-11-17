# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
