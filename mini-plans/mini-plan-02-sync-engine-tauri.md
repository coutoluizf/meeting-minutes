# Mini-Plan 02: Auth + Sync Engine no Tauri App

> **Status**: Pendente
> **Depende de**: Mini-Plan 01 (Supabase Foundation) ✅ completo
> **PRD Referência**: Seções 5.4, 5.6, 5.7, 8 do `PRD_CLOUD_SYNC_V1.md`
> **Estimativa**: ~1-2 sessões (usar Team Agents: Rust + React em paralelo)

---

## Contexto

O Supabase está configurado (mini-plan 01). Agora precisamos adicionar auth e sync ao app desktop Tauri existente. O app continua funcionando 100% sem login (modo local). Login é opt-in para habilitar cloud sync.

## Pré-requisitos

- [ ] Mini-Plan 01 completo e todos os testes passando
- [ ] `.env` com credenciais Supabase válidas
- [ ] Auth providers configurados no Supabase Dashboard
- [ ] App Tauri compila e roda (`cd frontend && pnpm run tauri:dev`)

## Escopo

### O que FAZER neste mini-plan:
1. Novo módulo Rust `cloud/` (auth, client, sync, storage)
2. Nova migration SQLite local (`cloud_sync_log`)
3. Novos Tauri commands (login, logout, sync, import)
4. Novos componentes React (login banner/modal, profile menu, sync indicator, onboarding)
5. Deep link handler (`meetily://auth/callback`)
6. Testes de integração Rust + E2E desktop

### O que NÃO fazer:
- Web UI (Fase 3)
- Copilot cloud (Fase 4)
- Modificar fluxo de gravação/transcrição existente (só adicionar sync APÓS)

---

## Steps de Implementação

### Step 1: Configuração do projeto

**1.1 — Adicionar dependências Rust (`frontend/src-tauri/Cargo.toml`)**:
```toml
# Novas dependências para cloud sync
reqwest = { version = "0.12", features = ["json"] }  # HTTP client (provavelmente já existe)
serde_json = "1"  # JSON (provavelmente já existe)
base64 = "0.22"   # Para PKCE code_challenge
sha2 = "0.10"     # Para PKCE S256 hash
rand = "0.8"      # Para PKCE code_verifier generation
```

**1.2 — Registrar deep link no Tauri (`frontend/src-tauri/tauri.conf.json`)**:
```json
{
  "plugins": {
    "deep-link": {
      "desktop": {
        "schemes": ["meetily"]
      }
    }
  }
}
```

**1.3 — Adicionar plugin deep-link**:
```toml
# Cargo.toml
tauri-plugin-deep-link = "2"
```

### Step 2: Módulo Rust — cloud/

Criar em `frontend/src-tauri/src/cloud/`:

```
cloud/
├── mod.rs               # Exports do módulo
├── auth.rs              # Auth: PKCE flow, token storage, refresh
├── client.rs            # Supabase HTTP client com auth headers
├── sync_engine.rs       # Sync local ↔ cloud (bidirecional)
├── sync_queue.rs        # Fila offline (persiste em SQLite)
├── storage.rs           # Upload/download áudio (Supabase Storage)
└── commands.rs          # Tauri commands expostos ao frontend
```

**2.1 — cloud/auth.rs**

```rust
// Responsabilidades:
// - Gerar code_verifier + code_challenge (PKCE S256)
// - Construir URL de auth do Supabase
// - Receber callback via deep link (meetily://auth/callback?code=X)
// - Trocar code por tokens (access_token + refresh_token)
// - Salvar sessão no SQLite local (tabela cloud_session)
// - Refresh automático antes de cada API call
// - Logout (limpar tokens locais + revogar no Supabase)

pub struct CloudSession {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub device_id: String,
}

pub async fn start_oauth_flow(provider: &str) -> Result<String, AuthError>;  // Retorna URL
pub async fn handle_callback(code: &str, code_verifier: &str) -> Result<CloudSession, AuthError>;
pub async fn refresh_session(session: &mut CloudSession) -> Result<(), AuthError>;
pub async fn logout(session: &CloudSession) -> Result<(), AuthError>;
pub async fn get_valid_token(session: &CloudSession) -> Result<String, AuthError>;
```

**2.2 — cloud/client.rs**

```rust
// Responsabilidades:
// - Wrapper HTTP sobre reqwest com auth headers automáticos
// - Endpoints: meetings, transcripts, summaries, chat_messages
// - UPSERT com sync_version para last-write-wins
// - Paginação para buscar meetings do cloud

pub struct SupabaseClient {
    url: String,
    anon_key: String,
    session: Arc<RwLock<Option<CloudSession>>>,
}

pub async fn upsert_meeting(&self, meeting: &MeetingSync) -> Result<(), SyncError>;
pub async fn upsert_transcripts(&self, transcripts: Vec<TranscriptSync>) -> Result<(), SyncError>;
pub async fn upsert_summary(&self, summary: &SummarySync) -> Result<(), SyncError>;
pub async fn fetch_meetings_since(&self, since: DateTime) -> Result<Vec<MeetingSync>, SyncError>;
pub async fn register_device(&self, device: &DeviceInfo) -> Result<String, SyncError>;
```

**2.3 — cloud/sync_engine.rs**

```rust
// Responsabilidades:
// - Sync local → cloud: detecta meetings novas/atualizadas, envia ao Supabase
// - Sync cloud → local: busca meetings de outros devices, salva localmente
// - Conflito: compara sync_version, maior ganha (last-write-wins)
// - Trigger: automático após gravação parar + polling a cada 5 min

pub struct SyncEngine {
    client: SupabaseClient,
    db: DatabaseManager,
    queue: SyncQueue,
}

pub async fn sync_all(&self) -> Result<SyncResult, SyncError>;
pub async fn sync_meeting(&self, local_meeting_id: &str) -> Result<(), SyncError>;
pub async fn import_local_meetings(&self, meeting_ids: Vec<String>, include_audio: bool) -> Result<ImportResult, SyncError>;
pub async fn get_sync_status(&self) -> SyncStatus;  // {pending: N, synced: N, errors: N}
```

**2.4 — cloud/sync_queue.rs**

```rust
// Responsabilidades:
// - Fila persistente em SQLite para sync offline
// - Quando offline: enqueue operations
// - Quando online: dequeue e executar em ordem
// - Retry com backoff para erros temporários

pub struct SyncQueue { db: DatabaseManager }

pub async fn enqueue(&self, op: SyncOperation) -> Result<(), QueueError>;
pub async fn process_queue(&self, client: &SupabaseClient) -> Result<usize, QueueError>;
pub async fn pending_count(&self) -> usize;
```

**2.5 — cloud/storage.rs**

```rust
// Responsabilidades:
// - Upload de áudio para Supabase Storage (background, chunked)
// - Download de áudio (se necessário)
// - Signed URLs para playback
// - Path format: {user_id}/{meeting_id}/recording.wav

pub async fn upload_audio(&self, meeting_id: &str, file_path: &Path) -> Result<String, StorageError>;
pub async fn get_signed_url(&self, meeting_id: &str) -> Result<String, StorageError>;
```

**2.6 — cloud/commands.rs**

```rust
// Tauri commands expostos ao frontend React

#[tauri::command]
async fn cloud_login(provider: String) -> Result<(), String>;
// Abre browser com URL de auth (PKCE)

#[tauri::command]
async fn cloud_logout() -> Result<(), String>;

#[tauri::command]
async fn cloud_get_session() -> Result<Option<CloudSessionInfo>, String>;
// Retorna info do user logado (sem tokens sensíveis)

#[tauri::command]
async fn cloud_sync_now() -> Result<SyncResult, String>;
// Força sync imediato

#[tauri::command]
async fn cloud_sync_status() -> Result<SyncStatus, String>;
// {is_syncing, last_sync, pending_count, error_count}

#[tauri::command]
async fn cloud_import_local_meetings(
    meeting_ids: Vec<String>,
    include_audio: bool,
    workspace_id: Option<String>,
) -> Result<ImportResult, String>;

#[tauri::command]
async fn cloud_get_local_meetings_for_import() -> Result<Vec<LocalMeetingInfo>, String>;
// Lista meetings locais que ainda não foram sincronizadas

#[tauri::command]
async fn cloud_set_audio_sync(enabled: bool) -> Result<(), String>;
```

### Step 3: Migration SQLite local

Adicionar em `frontend/src-tauri/migrations/`:

```sql
-- Nova tabela para controle de sync
CREATE TABLE IF NOT EXISTS cloud_sync_log (
    local_meeting_id TEXT PRIMARY KEY,
    cloud_meeting_id TEXT,
    last_synced_at TEXT,
    sync_version INTEGER DEFAULT 0,
    audio_synced INTEGER DEFAULT 0,
    status TEXT DEFAULT 'pending'  -- 'pending' | 'synced' | 'error'
);

-- Tabela para sessão cloud (singleton)
CREATE TABLE IF NOT EXISTS cloud_session (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton
    user_id TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT,
    avatar_url TEXT,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    device_id TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Tabela para fila de sync offline
CREATE TABLE IF NOT EXISTS cloud_sync_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation TEXT NOT NULL,       -- 'upsert_meeting' | 'upsert_transcript' | 'upload_audio' | ...
    payload TEXT NOT NULL,         -- JSON
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    created_at TEXT DEFAULT (datetime('now')),
    status TEXT DEFAULT 'pending'  -- 'pending' | 'processing' | 'completed' | 'failed'
);
```

### Step 4: Registrar comandos no Tauri

Em `frontend/src-tauri/src/lib.rs`, adicionar:

```rust
// Adicionar ao invoke_handler:
.invoke_handler(tauri::generate_handler![
    // ... comandos existentes ...

    // Cloud sync (NOVOS)
    cloud::commands::cloud_login,
    cloud::commands::cloud_logout,
    cloud::commands::cloud_get_session,
    cloud::commands::cloud_sync_now,
    cloud::commands::cloud_sync_status,
    cloud::commands::cloud_import_local_meetings,
    cloud::commands::cloud_get_local_meetings_for_import,
    cloud::commands::cloud_set_audio_sync,
])
```

### Step 5: Deep Link Handler

Configurar no setup do Tauri app para capturar `meetily://auth/callback`:

```rust
// Em lib.rs ou setup
app.handle().plugin(tauri_plugin_deep_link::init())?;

// Listener para deep links
app.listen("deep-link://new-url", |event| {
    // Parse URL: meetily://auth/callback?code=XXX
    // Chamar cloud::auth::handle_callback(code, stored_code_verifier)
    // Emitir evento para frontend: "cloud-auth-success" ou "cloud-auth-error"
});
```

### Step 6: Componentes React (UI Cloud)

Criar em `frontend/src/components/Cloud/`:

```
Cloud/
├── CloudLoginBanner.tsx        # Banner sutil na parte inferior do app
├── CloudLoginModal.tsx         # Modal com botões de login (Google, Email, GitHub)
├── CloudProfileMenu.tsx        # Avatar + dropdown na topbar (logado)
├── CloudSyncIndicator.tsx      # Ícone na topbar: ☁️ Synced / ⟳ Syncing / ⚠ Error
├── CloudOnboardingModal.tsx    # "Importar reuniões locais?" (primeiro login)
├── CloudSyncSettings.tsx       # Toggle sync áudio, gerenciar devices, re-import
└── CloudImportProgress.tsx     # Barra de progresso durante importação
```

**6.1 — CloudLoginBanner.tsx**
- Aparece só se NÃO logado
- Posição: parte inferior do app, acima da barra de status
- Texto: "☁️ Sincronize entre dispositivos"
- Botão: "Fazer login"
- Pode ser dispensado ("x" para fechar, mas volta na próxima sessão)

**6.2 — CloudLoginModal.tsx**
- Abre ao clicar em "Fazer login" (banner ou settings)
- Botões: "Continuar com Google", "Continuar com GitHub", "Continuar com Email"
- Email: input para magic link
- Ao clicar: chama `invoke('cloud_login', { provider })` que abre browser

**6.3 — CloudProfileMenu.tsx**
- Aparece na topbar só se logado
- Mostra avatar + nome
- Dropdown: "Sync status", "Devices", "Settings", "Logout"

**6.4 — CloudSyncIndicator.tsx**
- Ícone na topbar (ao lado do profile menu)
- Estados: ☁️ Synced (verde), ⟳ Syncing (animando), ⚠ Error (vermelho), 📴 Offline (cinza)
- Tooltip com detalhes: "Last sync: 2 min ago · 3 pending"

**6.5 — CloudOnboardingModal.tsx**
- Aparece automaticamente no PRIMEIRO login de cada device
- Lista meetings locais com checkboxes (título, data, duração, tem áudio?, tem resumo?)
- Toggle "Incluir áudio" com tamanho total calculado
- Dropdown workspace destino
- Botões: "Importar selecionadas", "Agora não", "Nunca perguntar"
- Chama `invoke('cloud_get_local_meetings_for_import')` para listar
- Chama `invoke('cloud_import_local_meetings', ...)` para importar

**6.6 — CloudImportProgress.tsx**
- Barra de progresso: "Importando 12 de 45 reuniões"
- Lista com status por meeting (✓ synced, ⟳ syncing, ○ pending)
- Contadores: Transcrições, Resumos, Áudio
- Botão "Minimizar" (continua em background)

### Step 7: Context/State para Cloud

Criar `frontend/src/contexts/CloudContext.tsx`:

```typescript
// Gerencia estado de auth + sync no React
// - isLoggedIn, user (email, avatar), syncStatus
// - Escuta eventos Tauri: "cloud-auth-success", "cloud-sync-update"
// - Provê para todos os componentes Cloud
```

### Step 8: Integrar no app existente

- Adicionar `CloudSyncIndicator` e `CloudProfileMenu` na topbar (ou `CloudLoginBanner` se não logado)
- Adicionar `CloudSyncSettings` na página de Settings
- Registrar listener para deep links no layout principal
- Após `stop_recording`, chamar `cloud_sync_now()` se logado

---

## Testes E2E — Mini-Plan 02

### Rust Integration Tests

Criar em `frontend/src-tauri/tests/`:

```rust
// cloud_auth_test.rs
#[tokio::test] async fn test_pkce_code_generation()        // code_verifier + challenge válidos
#[tokio::test] async fn test_token_refresh()                // refresh_token → novo access_token
#[tokio::test] async fn test_session_persistence()          // salvar/carregar sessão do SQLite

// cloud_sync_test.rs
#[tokio::test] async fn test_sync_meeting_to_cloud()        // local meeting → Supabase
#[tokio::test] async fn test_sync_transcripts()             // transcripts sincronizam
#[tokio::test] async fn test_sync_summary()                 // summary sincroniza
#[tokio::test] async fn test_sync_version_conflict()        // last-write-wins funciona
#[tokio::test] async fn test_dedup_same_meeting()           // mesma meeting não duplica

// cloud_import_test.rs
#[tokio::test] async fn test_list_local_meetings()          // lista meetings não sincronizadas
#[tokio::test] async fn test_import_selective()             // importa só as selecionadas
#[tokio::test] async fn test_import_with_audio()            // upload áudio durante import
#[tokio::test] async fn test_import_without_audio()         // só metadata + transcripts

// cloud_storage_test.rs
#[tokio::test] async fn test_upload_audio()                 // upload para Supabase Storage
#[tokio::test] async fn test_signed_url()                   // signed URL funciona
```

### Comando de Testes Cumulativos

```bash
# Mini-plan 02 — roda testes deste mini-plan + mini-plan 01
echo "=== Mini-Plan 01: Supabase Foundation ==="
cd supabase && ./run-tests.sh

echo "=== Mini-Plan 02: Sync Engine ==="
cd frontend/src-tauri && cargo test --test '*cloud*' -- --test-threads=1

echo "=== RESULTADO CUMULATIVO ==="
# Esperado: 32 (MP01) + 14 (MP02) = 46 testes passando
```

---

## Checklist de Validação Final

Antes de avançar para o Mini-Plan 03, confirmar:

- [ ] Módulo `cloud/` compila sem erros (`cargo check`)
- [ ] Deep link `meetily://` é capturado pelo app
- [ ] Login via Google abre browser e retorna sessão
- [ ] Sessão persiste entre restarts do app
- [ ] Meeting gravada localmente sincroniza para Supabase
- [ ] Transcripts e summaries sincronizam junto
- [ ] Import de meetings locais funciona (modal de seleção)
- [ ] Áudio upload funciona (quando toggle ativo)
- [ ] Sync indicator mostra status correto
- [ ] App funciona 100% sem login (modo local inalterado)
- [ ] Todos os 46 testes cumulativos passando (32 MP01 + 14 MP02)

---

*Anterior: `mini-plan-01-supabase-foundation.md`*
*Próximo: `mini-plan-03-web-ui.md`*
