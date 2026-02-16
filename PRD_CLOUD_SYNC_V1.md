# PRD: Meetily Cloud Sync v1

> **Status**: Draft
> **Autor**: Luiz + Claude
> **Data**: 2026-02-16
> **Escopo**: Sincronização de reuniões com Supabase + Web UI

---

## 1. Problema

O Meetily armazena todas as reuniões localmente no dispositivo onde foram gravadas. Usuários que trabalham em múltiplos dispositivos (laptop + desktop) não conseguem acessar reuniões de um dispositivo no outro. Dados ficam isolados, sem possibilidade de busca unificada ou acesso remoto.

## 2. Solução

Implementar sincronização na nuvem via Supabase, permitindo que reuniões gravadas em qualquer dispositivo sejam acessíveis de qualquer outro dispositivo ou via interface web. Incluir um Copilot na nuvem para conversar sobre reuniões de qualquer lugar.

## 3. Objetivos v1

- [ ] Sincronizar meetings, transcripts, summaries e chat history para Supabase
- [ ] Autenticação de usuário (Supabase Auth)
- [ ] Web UI para visualizar e interagir com reuniões sincronizadas
- [ ] Copilot na nuvem (chat sobre reuniões via web)
- [ ] Busca semântica cross-device via pgvector
- [ ] Upload opcional de áudio para Supabase Storage

## 4. Fora de Escopo (v1)

- Multi-channel (emails, Slack, etc.) — planejado para v2
- Integrações CRM (Salesforce, HubSpot) — planejado para v2
- MCP Tools / API pública — planejado para v2
- Collaborative features (múltiplos usuários no mesmo workspace)
- Gravação via web (continua sendo exclusiva do app Tauri)

---

## 5. Autenticação

### 5.1. Contexto

O Meetily **não possui autenticação hoje**. O app Tauri é 100% local, single-user, e não requer login para funcionar. Com a introdução do Cloud Sync, autenticação se torna necessária — mas **apenas para habilitar o sync**, não para usar o app localmente.

### 5.2. Princípio Fundamental

```
┌─────────────────────────────────────────────────────────────────┐
│                        App Tauri (Desktop)                       │
│                                                                  │
│  ┌───────────────────────────────┐ ┌──────────────────────────┐ │
│  │     MODO LOCAL (padrão)       │ │   MODO CLOUD (opt-in)    │ │
│  │                               │ │                          │ │
│  │  • Sem login necessário       │ │  • Requer login          │ │
│  │  • Funciona 100% como hoje   │ │  • Habilita sync         │ │
│  │  • Dados só no dispositivo    │ │  • Dados local + nuvem   │ │
│  │  • Zero dependência de rede   │ │  • Offline-first         │ │
│  │  • Privacidade total          │ │  • Copilot cloud         │ │
│  │                               │ │                          │ │
│  │  Gravar ✓ Transcrever ✓      │ │  Tudo do local +         │ │
│  │  Resumir ✓ Chat local ✓      │ │  Sync ✓ Web access ✓     │ │
│  └───────────────────────────────┘ └──────────────────────────┘ │
│                                                                  │
│  O usuário escolhe: app funciona sem login.                      │
│  Login é opt-in para quem quer sync.                             │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                        Web UI                                    │
│                                                                  │
│  • Login OBRIGATÓRIO (não existe modo offline)                   │
│  • Sem auth = só vê landing page / marketing                     │
│  • Com auth = acessa dashboard, meetings, copilot, settings      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 5.3. Provedor de Auth: Supabase Auth

| Método de Login | Tauri App | Web UI | Notas |
|-----------------|-----------|--------|-------|
| **Magic Link (email)** | ✓ | ✓ | Mais simples, sem senha |
| **Google OAuth** | ✓ | ✓ | Login rápido, maioria dos usuários tem |
| **GitHub OAuth** | ✓ | ✓ | Público técnico / dev |
| **Apple Sign-In** | — (futuro) | — (futuro) | Requer Apple Developer account |

### 5.4. Fluxo de Auth no App Tauri (Desktop)

```
┌─────────────────────────────────────────────────────────────┐
│  App inicia (como hoje)                                      │
│  → Verifica se há sessão Supabase salva localmente           │
│                                                              │
│  SE não logado:                                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  App funciona normalmente (modo local)                  │ │
│  │  Banner sutil no topo ou settings:                      │ │
│  │  "☁️ Faça login para sincronizar entre dispositivos"    │ │
│  │  [Entrar com Google]  [Entrar com Email]                │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  SE logado:                                                  │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  App funciona com sync ativo                            │ │
│  │  Indicador de sync na topbar: ☁️ Synced / ⟳ Syncing    │ │
│  │  Menu do perfil: avatar, logout, sync settings          │ │
│  │                                                         │ │
│  │  Primeiro login? → Modal de onboarding:                 │ │
│  │  "Encontramos 42 reuniões locais. Importar para nuvem?" │ │
│  │  [Importar todas] [Selecionar] [Agora não]             │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Implementação técnica no Tauri**:

O fluxo OAuth no desktop usa o padrão **PKCE (Proof Key for Code Exchange)**, que é o recomendado para apps nativos:

```
1. Usuário clica "Entrar com Google"
2. App gera code_verifier + code_challenge (PKCE)
3. App abre o navegador padrão do SO com a URL de auth do Supabase:
   https://<project>.supabase.co/auth/v1/authorize?
     provider=google&
     redirect_to=meetily://auth/callback&
     code_challenge=<hash>&
     code_challenge_method=S256
4. Usuário autentica no browser (Google consent screen)
5. Supabase redireciona para meetily://auth/callback?code=<auth_code>
6. Tauri captura o deep link (meetily:// protocol handler)
7. App troca auth_code + code_verifier por session tokens (access + refresh)
8. Tokens salvos no SQLite local (encrypted)
9. Sync engine inicia automaticamente
```

**Deep Link no Tauri** (requer configuração):

```json
// tauri.conf.json — registrar protocol handler
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

**Gestão de tokens** (Rust, módulo `cloud/auth.rs`):

```rust
// Estrutura de sessão armazenada localmente
pub struct CloudSession {
    pub user_id: String,            // UUID do Supabase Auth
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,       // JWT, expira em 1h
    pub refresh_token: String,      // Longa duração, usado para renovar
    pub expires_at: i64,            // Timestamp de expiração
    pub device_id: String,          // UUID deste device (registrado no Supabase)
}

// Refresh automático antes de cada chamada à API
pub async fn get_valid_token(&self) -> Result<String, AuthError> {
    if self.is_token_expired() {
        self.refresh_session().await?;
    }
    Ok(self.access_token.clone())
}
```

### 5.5. Fluxo de Auth na Web UI (Next.js)

```
┌──────────────────────────────────────────────────────────────┐
│  Usuário acessa app.meetily.ai                                │
│                                                               │
│  SE não logado:                                               │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                                                          │ │
│  │             ┌─────────────────────────┐                  │ │
│  │             │      Meetily ☁️          │                  │ │
│  │             │                         │                  │ │
│  │             │  Suas reuniões,         │                  │ │
│  │             │  em qualquer lugar.     │                  │ │
│  │             │                         │                  │ │
│  │             │  [Entrar com Google]    │                  │ │
│  │             │  [Entrar com Email]     │                  │ │
│  │             │  [Entrar com GitHub]    │                  │ │
│  │             │                         │                  │ │
│  │             │  Não tem conta?         │                  │ │
│  │             │  Baixe o app desktop    │                  │ │
│  │             └─────────────────────────┘                  │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                               │
│  SE logado:                                                   │
│  → Redireciona para /dashboard                                │
│  → Middleware Next.js valida JWT em cada request              │
│  → Server Components usam Supabase server client (SSR)       │
│  → Client Components usam Supabase browser client            │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

**Implementação técnica no Next.js**:

```
src/
├── lib/supabase/
│   ├── client.ts          # createBrowserClient() — client components
│   ├── server.ts          # createServerClient() — server components / RSC
│   └── middleware.ts       # Refresh token + proteger rotas (app)/*
├── app/
│   ├── (auth)/
│   │   ├── login/page.tsx          # Página de login (pública)
│   │   ├── callback/route.ts       # OAuth callback handler (GET)
│   │   └── logout/route.ts         # Logout handler
│   ├── (marketing)/
│   │   └── page.tsx                # Landing page (pública)
│   └── (app)/
│       ├── layout.tsx              # Valida sessão, redireciona se não logado
│       └── ...                     # Todas as rotas protegidas
└── middleware.ts                    # Next.js middleware global (refresh token)
```

**Middleware de proteção** (Next.js):

```typescript
// middleware.ts — executa em TODA request
// 1. Refresh do token Supabase (se expirado)
// 2. Se rota é (app)/* e não tem sessão → redirect para /login
// 3. Se rota é /login e tem sessão → redirect para /dashboard
```

### 5.6. Segurança de Auth

| Aspecto | Implementação |
|---------|---------------|
| **Tokens no Tauri** | Salvos no SQLite local, coluna encrypted (ou Keychain/Credential Manager do SO) |
| **Tokens na Web** | HTTP-only cookies (configurado pelo Supabase SSR helpers) |
| **Refresh automático** | Tauri: antes de cada API call. Web: via middleware Next.js |
| **Logout** | Revoga refresh token no Supabase + limpa storage local |
| **PKCE** | Obrigatório para OAuth no Tauri (previne interceptação) |
| **RLS** | Todas as queries filtradas por `auth.uid()` no Supabase |
| **Sessão expirada** | Tauri: volta ao modo local silenciosamente. Web: redirect para /login |
| **Múltiplos devices** | Cada device tem sua própria sessão independente |

### 5.7. UX de Auth no Tauri — Componentes UI Novos

```
Componentes novos no app Tauri (React/TypeScript):
├── CloudLoginBanner.tsx        # Banner sutil "Faça login para sync"
├── CloudLoginModal.tsx         # Modal com botões OAuth + magic link
├── CloudProfileMenu.tsx        # Avatar + dropdown (sync status, logout)
├── CloudSyncIndicator.tsx      # Ícone na topbar (synced/syncing/offline)
├── CloudOnboardingModal.tsx    # "Importar reuniões locais?" (primeiro login)
└── CloudSyncSettings.tsx       # Toggle sync áudio, gerenciar devices
```

**Onde aparecem no app**:

```
┌──────────────────────────────────────────────────────┐
│  Meetily    [☁️ Synced]                    [👤 Luiz ▼]│  ← CloudSyncIndicator + CloudProfileMenu
│──────────────────────────────────────────────────────│
│  │          │                                        │
│  │ Sidebar  │     Main Content                       │
│  │          │     (sem mudanças no fluxo existente)  │
│  │          │                                        │
│  │          │                                        │
│  │          │                                        │
│  │          │                                        │
│  └──────────┘                                        │
│                                                      │
│  ┌──────────────────────────────────────────────────┐│
│  │ ☁️ Sincronize entre dispositivos [Fazer login]   ││  ← CloudLoginBanner (só se não logado)
│  └──────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────┘
```

---

## 6. Arquitetura Geral

### 6.1. Visão Geral

```
┌──────────────────────────────────────────────────────────────┐
│              Dispositivo (Tauri App - existente)              │
│                                                              │
│  ┌──────────┐  ┌───────────┐  ┌────────────┐               │
│  │ Gravação  │→ │Transcrição│→ │  Resumo AI │               │
│  │ (Audio)   │  │(Parakeet) │  │ (LLM local)│               │
│  └──────────┘  └───────────┘  └────────────┘               │
│        │              │              │                        │
│        ▼              ▼              ▼                        │
│  ┌─────────────────────────────────────────┐                │
│  │          SQLite Local (existente)        │                │
│  └────────────────────┬────────────────────┘                │
│                       │                                      │
│                 Sync Engine (NOVO)                            │
│                       │                                      │
└───────────────────────┼──────────────────────────────────────┘
                        │ HTTPS
                        ▼
┌──────────────────────────────────────────────────────────────┐
│                    Supabase (NOVO)                            │
│                                                              │
│  ┌────────┐ ┌────────────┐ ┌─────────┐ ┌────────────────┐  │
│  │  Auth   │ │ PostgreSQL │ │ Storage │ │ Edge Functions │  │
│  │(users,  │ │(meetings,  │ │ (áudio  │ │ (embedding     │  │
│  │ devices)│ │ transcripts│ │  .wav)  │ │  generation,   │  │
│  │         │ │ summaries, │ │         │ │  copilot API)  │  │
│  │         │ │ pgvector)  │ │         │ │                │  │
│  └────────┘ └────────────┘ └─────────┘ └────────────────┘  │
│                                                              │
└──────────────────────┬───────────────────────────────────────┘
                       │ HTTPS
                       ▼
┌──────────────────────────────────────────────────────────────┐
│                Web UI (NOVO) — Next.js 16                    │
│                                                              │
│  ┌──────────────┐ ┌────────────┐ ┌────────────────────────┐ │
│  │  Dashboard    │ │  Meeting   │ │  Copilot (Chat AI)    │ │
│  │  (Workspaces, │ │  Viewer    │ │  (4 níveis contexto:  │ │
│  │   Meetings)   │ │ (Transcript│ │   global, workspace,  │ │
│  │               │ │  + Summary)│ │   contact, interaction│ │
│  └──────────────┘ └────────────┘ └────────────────────────┘ │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 6.2. Stack Técnica

| Camada | Tecnologia | Versão | Justificativa |
|--------|-----------|--------|---------------|
| **Web UI** | Next.js | 16.1+ (Turbopack) | Último stable, RSC, Turbopack default |
| **Web UI** | React | 19 | Concurrent features, use cache |
| **Web UI** | Tailwind CSS | 4.x | Styling utility-first |
| **Web UI** | Framer Motion | latest | Animações sofisticadas |
| **Web UI** | Shadcn/ui | latest | Component library (já usado no Tauri app) |
| **Auth** | Supabase Auth | latest | Magic link, Google, GitHub OAuth |
| **Database** | Supabase PostgreSQL | 16+ | Relacional + JSONB |
| **Vector** | pgvector + pgvectorscale | latest | Embeddings, busca semântica, DiskANN |
| **Storage** | Supabase Storage | latest | Áudio files (S3-compatible) |
| **Functions** | Supabase Edge Functions | Deno | Embedding generation, copilot API |
| **Hosting Web** | Vercel ou Cloudflare Pages | — | Deploy automático do Next.js |
| **Desktop App** | Tauri 2.6 (existente) | — | Sync engine adicionado |

---

## 7. Modelo de Dados (Supabase PostgreSQL)

### 7.1. Schema Principal

```sql
-- ============================================================
-- EXTENSIONS
-- ============================================================
CREATE EXTENSION IF NOT EXISTS vector;           -- pgvector
CREATE EXTENSION IF NOT EXISTS pgvectorscale;    -- DiskANN indexing
CREATE EXTENSION IF NOT EXISTS pg_trgm;          -- Trigram search (fuzzy text)

-- ============================================================
-- USERS & DEVICES
-- ============================================================
-- users: Gerenciado pelo Supabase Auth (auth.users)
-- Não criamos tabela, usamos auth.users diretamente

-- Perfil público do usuário (extensão do auth.users)
CREATE TABLE public.profiles (
    id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
    display_name TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Dispositivos registrados do usuário
CREATE TABLE public.devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,                        -- "MacBook Pro do Luiz", "Desktop"
    platform TEXT NOT NULL,                    -- "macos", "windows", "linux"
    app_version TEXT,                          -- "0.1.1"
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- WORKSPACES (organização flexível)
-- ============================================================
CREATE TABLE public.workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,                                  -- Emoji ou icon name
    color TEXT,                                 -- Hex color
    type_hint TEXT DEFAULT 'custom',            -- 'project' | 'client' | 'personal' | 'custom'
    is_default BOOLEAN DEFAULT FALSE,           -- Workspace padrão (inbox)
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- CONTACTS (entidade global, cross-workspace)
-- ============================================================
CREATE TABLE public.contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    email TEXT,
    company TEXT,
    phone TEXT,
    avatar_url TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Relação N:N entre contacts e workspaces
CREATE TABLE public.workspace_contacts (
    workspace_id UUID NOT NULL REFERENCES public.workspaces(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES public.contacts(id) ON DELETE CASCADE,
    role TEXT,                                  -- "stakeholder", "decisor", "técnico"
    PRIMARY KEY (workspace_id, contact_id)
);

-- ============================================================
-- MEETINGS (entidade principal v1)
-- ============================================================
CREATE TABLE public.meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    workspace_id UUID REFERENCES public.workspaces(id) ON DELETE SET NULL,
    device_id UUID REFERENCES public.devices(id) ON DELETE SET NULL,

    -- Identidade
    title TEXT NOT NULL,
    local_meeting_id TEXT,                     -- ID original do SQLite local (para dedup)

    -- Áudio
    audio_url TEXT,                             -- URL no Supabase Storage (opcional)
    audio_duration_secs REAL,
    audio_synced BOOLEAN DEFAULT FALSE,

    -- Metadados
    recorded_at TIMESTAMPTZ NOT NULL,          -- Quando a reunião aconteceu
    created_at TIMESTAMPTZ DEFAULT NOW(),      -- Quando o registro foi criado
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Busca semântica
    embedding VECTOR(1536),                    -- OpenAI text-embedding-3-small

    -- Sync control
    sync_version BIGINT DEFAULT 1,             -- Incrementa a cada update (last-write-wins)
    origin_device_id UUID REFERENCES public.devices(id)
);

-- ============================================================
-- TRANSCRIPTS (segmentos de transcrição)
-- ============================================================
CREATE TABLE public.transcripts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL REFERENCES public.meetings(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,

    -- Conteúdo
    text TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,

    -- Sincronização com áudio
    audio_start_time REAL,                     -- Segundos desde início da gravação
    audio_end_time REAL,
    duration REAL,

    -- Metadados
    speaker TEXT,                              -- Futuro: diarização
    confidence REAL,                           -- Confiança da transcrição

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- SUMMARIES (resumos gerados por AI)
-- ============================================================
CREATE TABLE public.summaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL REFERENCES public.meetings(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,

    -- Conteúdo
    markdown TEXT NOT NULL,                    -- Resumo completo em Markdown
    summary_json JSONB,                        -- Estrutura do resumo (sections, blocks)

    -- Metadados de geração
    model_provider TEXT NOT NULL,              -- "ollama", "claude", "groq", "openai", "openrouter"
    model_name TEXT NOT NULL,                  -- "gpt-4o", "claude-3-5-sonnet", etc.
    template_id TEXT,                          -- Template usado
    processing_time_secs REAL,
    chunk_count INTEGER DEFAULT 1,

    -- Versioning (usuário pode editar)
    version INTEGER DEFAULT 1,
    is_edited BOOLEAN DEFAULT FALSE,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- CHAT MESSAGES (Copilot conversations)
-- ============================================================
CREATE TABLE public.chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,

    -- Contexto polimórfico (sobre o que é essa conversa?)
    context_type TEXT NOT NULL,                 -- 'meeting' | 'workspace' | 'contact' | 'global'
    context_id UUID,                           -- ID da entidade (meeting_id, workspace_id, contact_id, ou NULL para global)

    -- Conteúdo
    role TEXT NOT NULL,                         -- 'user' | 'assistant'
    content TEXT NOT NULL,
    model_used TEXT,                            -- Qual LLM respondeu

    -- Metadados
    origin TEXT DEFAULT 'web',                 -- 'web' | 'desktop' | 'api'
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- MEETING <-> CONTACTS (N:N — quem participou)
-- ============================================================
CREATE TABLE public.meeting_contacts (
    meeting_id UUID NOT NULL REFERENCES public.meetings(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES public.contacts(id) ON DELETE CASCADE,
    role TEXT,                                  -- "presenter", "participant", "observer"
    PRIMARY KEY (meeting_id, contact_id)
);

-- ============================================================
-- INDEXES
-- ============================================================

-- Busca semântica com DiskANN (pgvectorscale)
CREATE INDEX idx_meetings_embedding ON public.meetings
    USING diskann (embedding vector_cosine_ops);

-- Busca por usuário (todas as queries filtram por user)
CREATE INDEX idx_meetings_user ON public.meetings(user_id);
CREATE INDEX idx_meetings_workspace ON public.meetings(workspace_id);
CREATE INDEX idx_transcripts_meeting ON public.transcripts(meeting_id);
CREATE INDEX idx_summaries_meeting ON public.summaries(meeting_id);
CREATE INDEX idx_chat_messages_context ON public.chat_messages(context_type, context_id);
CREATE INDEX idx_contacts_user ON public.contacts(user_id);
CREATE INDEX idx_devices_user ON public.devices(user_id);

-- Full-text search (complementa vector search)
CREATE INDEX idx_transcripts_text_trgm ON public.transcripts
    USING gin (text gin_trgm_ops);

-- ============================================================
-- ROW LEVEL SECURITY (cada usuário só vê seus dados)
-- ============================================================

ALTER TABLE public.profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.workspaces ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.contacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.workspace_contacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.meetings ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.transcripts ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.summaries ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.chat_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.meeting_contacts ENABLE ROW LEVEL SECURITY;

-- Policy padrão: usuário só acessa seus próprios dados
-- (Exemplo para meetings, replicar para todas as tabelas)
CREATE POLICY "Users can CRUD own meetings"
    ON public.meetings FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can CRUD own workspaces"
    ON public.workspaces FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can CRUD own contacts"
    ON public.contacts FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can CRUD own transcripts"
    ON public.transcripts FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can CRUD own summaries"
    ON public.summaries FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can CRUD own chat messages"
    ON public.chat_messages FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can CRUD own devices"
    ON public.devices FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can CRUD own profiles"
    ON public.profiles FOR ALL
    USING (auth.uid() = id)
    WITH CHECK (auth.uid() = id);

-- Policies para tabelas de junção (precisa verificar via parent)
CREATE POLICY "Users can manage workspace_contacts"
    ON public.workspace_contacts FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM public.workspaces
            WHERE id = workspace_contacts.workspace_id
            AND user_id = auth.uid()
        )
    );

CREATE POLICY "Users can manage meeting_contacts"
    ON public.meeting_contacts FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM public.meetings
            WHERE id = meeting_contacts.meeting_id
            AND user_id = auth.uid()
        )
    );

-- ============================================================
-- FUNCTIONS (helpers)
-- ============================================================

-- Atualiza updated_at automaticamente
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER meetings_updated_at
    BEFORE UPDATE ON public.meetings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER workspaces_updated_at
    BEFORE UPDATE ON public.workspaces
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER contacts_updated_at
    BEFORE UPDATE ON public.contacts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER summaries_updated_at
    BEFORE UPDATE ON public.summaries
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER profiles_updated_at
    BEFORE UPDATE ON public.profiles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Busca semântica por similaridade
CREATE OR REPLACE FUNCTION search_meetings_by_embedding(
    query_embedding VECTOR(1536),
    match_user_id UUID,
    match_workspace_id UUID DEFAULT NULL,
    match_threshold FLOAT DEFAULT 0.7,
    match_count INT DEFAULT 10
)
RETURNS TABLE (
    id UUID,
    title TEXT,
    recorded_at TIMESTAMPTZ,
    similarity FLOAT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        m.id,
        m.title,
        m.recorded_at,
        1 - (m.embedding <=> query_embedding) AS similarity
    FROM public.meetings m
    WHERE m.user_id = match_user_id
        AND (match_workspace_id IS NULL OR m.workspace_id = match_workspace_id)
        AND m.embedding IS NOT NULL
        AND 1 - (m.embedding <=> query_embedding) > match_threshold
    ORDER BY m.embedding <=> query_embedding
    LIMIT match_count;
END;
$$;
```

### 7.2. Supabase Storage Buckets

```
meetily-audio/
  └── {user_id}/
      └── {meeting_id}/
          └── recording.wav        -- Áudio original (upload opcional)
```

- **Bucket policy**: Privado, acesso via signed URLs (expiram em 1h)
- **Limite**: 500MB por arquivo (suficiente para reuniões de várias horas)

---

## 8. Sync Engine (Tauri App → Supabase)

### 8.1. Fluxo de Sincronização

```
                    ┌─────────────────┐
                    │ Meeting gravada │
                    │ (salva no SQLite│
                    │  local como hj) │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  Sync Engine    │
                    │  (background)   │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
     ┌──────────────┐ ┌──────────┐ ┌──────────────┐
     │ Upload meeting│ │ Upload   │ │ Upload áudio │
     │ metadata +    │ │ summary  │ │ (opcional,   │
     │ transcripts   │ │ (se      │ │  background) │
     │               │ │  existir)│ │              │
     └──────┬───────┘ └────┬─────┘ └──────┬───────┘
            │              │              │
            ▼              ▼              ▼
     ┌─────────────────────────────────────────┐
     │         Supabase (via SDK)              │
     │  → INSERT/UPSERT com sync_version      │
     │  → Edge Function gera embedding         │
     └─────────────────────────────────────────┘
```

### 8.2. Regras de Sync

| Regra | Descrição |
|-------|-----------|
| **Direção** | Bidirecional: local → cloud e cloud → local |
| **Conflito** | Last-write-wins baseado em `sync_version` (incrementa a cada update) |
| **Dedup** | `local_meeting_id` + `device_id` evita duplicatas |
| **Áudio** | Upload opcional (usuário decide nas settings). Metadata sempre sincroniza |
| **Frequência** | Sync imediato após gravação parar. Polling periódico (a cada 5 min) para updates de outros devices |
| **Offline** | App funciona 100% offline. Queue de sync executa quando online |
| **Primeira sync** | Ao logar pela primeira vez, oferece importar meetings locais existentes para a nuvem |

### 8.3. Mudanças no App Tauri (Rust)

Novo módulo: `frontend/src-tauri/src/cloud/`

```
cloud/
├── mod.rs               -- Exports
├── auth.rs              -- Supabase Auth (login, token refresh)
├── client.rs            -- Supabase client HTTP wrapper
├── sync_engine.rs       -- Lógica de sincronização bidirecional
├── sync_queue.rs        -- Fila offline (persiste em SQLite local)
├── storage.rs           -- Upload/download de áudio (Supabase Storage)
├── embedding.rs         -- Chamada à Edge Function para gerar embedding
└── commands.rs          -- Tauri commands (login, logout, sync_now, sync_status)
```

Novos Tauri commands:

```rust
// Auth
#[tauri::command] async fn cloud_login(provider: String) -> Result<AuthSession, String>;
#[tauri::command] async fn cloud_logout() -> Result<(), String>;
#[tauri::command] async fn cloud_get_session() -> Result<Option<AuthSession>, String>;

// Sync
#[tauri::command] async fn cloud_sync_now() -> Result<SyncResult, String>;
#[tauri::command] async fn cloud_sync_status() -> Result<SyncStatus, String>;
#[tauri::command] async fn cloud_import_local_meetings() -> Result<ImportResult, String>;

// Settings
#[tauri::command] async fn cloud_set_audio_sync(enabled: bool) -> Result<(), String>;
```

### 8.4. Tabela local adicional (SQLite)

```sql
-- Controle de sync no SQLite local
CREATE TABLE cloud_sync_log (
    local_meeting_id TEXT PRIMARY KEY,
    cloud_meeting_id TEXT,            -- UUID no Supabase
    last_synced_at TEXT,
    sync_version INTEGER DEFAULT 0,
    audio_synced BOOLEAN DEFAULT 0,
    status TEXT DEFAULT 'pending'     -- 'pending' | 'synced' | 'error'
);
```

### 8.5. Importação de Meetings Locais Existentes

#### Contexto

O app Tauri **não possui autenticação hoje** e já pode ter dezenas ou centenas de reuniões gravadas localmente em cada dispositivo. Quando o usuário faz login pela primeira vez (habilitando sync), precisa de um fluxo claro para importar seletivamente essas reuniões para a nuvem.

#### Quando acontece

| Trigger | Comportamento |
|---------|---------------|
| **Primeiro login no device** | Modal de onboarding aparece automaticamente |
| **A qualquer momento** | Disponível em Settings → Cloud → "Importar reuniões locais" |
| **Novas meetings locais** | Meetings gravadas APÓS login sincronizam automaticamente |

#### Fluxo de Importação — Onboarding (primeiro login)

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│  ☁️  Bem-vindo ao Meetily Cloud!                             │
│                                                              │
│  Encontramos 47 reuniões locais neste dispositivo.           │
│  Selecione as que deseja sincronizar com a nuvem.            │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  ☑  Selecionar todas (47)          Ordenar: Data ▼    │  │
│  │──────────────────────────────────────────────────────  │  │
│  │                                                        │  │
│  │  ☑  Team Standup                                       │  │
│  │     14/02/2026 · 45min · 🎤 Transcrição · 📝 Resumo   │  │
│  │     🔊 Áudio: 23 MB                                    │  │
│  │                                                        │  │
│  │  ☑  Client Review — Acme Corp                          │  │
│  │     13/02/2026 · 1h20 · 🎤 Transcrição · 📝 Resumo    │  │
│  │     🔊 Áudio: 58 MB                                    │  │
│  │                                                        │  │
│  │  ☐  Teste de microfone                                 │  │
│  │     10/02/2026 · 2min · 🎤 Transcrição                 │  │
│  │     🔊 Áudio: 1.2 MB                                   │  │
│  │                                                        │  │
│  │  ☑  Sprint Retrospective                               │  │
│  │     09/02/2026 · 55min · 🎤 Transcrição · 📝 Resumo   │  │
│  │     🔊 Áudio: 31 MB                                    │  │
│  │                                                        │  │
│  │  ... (scroll)                                          │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  Destino: [Workspace padrão ▼]  (pode reorganizar depois)   │
│                                                              │
│  ☑ Incluir arquivos de áudio (113 MB total)                  │
│    ℹ️ Sem áudio, só transcrições e resumos são sincronizados  │
│                                                              │
│  ┌──────────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Importar 45 de 47│  │  Agora não   │  │  Nunca       │   │
│  │    selecionadas   │  │ (lembra dps) │  │  perguntar   │   │
│  └──────────────────┘  └──────────────┘  └──────────────┘   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### Fluxo de Importação — Via Settings (a qualquer momento)

```
Settings → Cloud → Importar reuniões locais

┌────────────────────────────────────────────────────────────┐
│  Reuniões locais não sincronizadas: 2                       │
│  (Reuniões já sincronizadas não aparecem aqui)             │
│                                                            │
│  ☑  Teste de microfone                                     │
│     10/02/2026 · 2min                                      │
│                                                            │
│  ☑  Brainstorm rápido                                      │
│     08/02/2026 · 15min                                     │
│                                                            │
│  Destino: [Workspace padrão ▼]                             │
│  ☐ Incluir áudio                                           │
│                                                            │
│  [Importar selecionadas]                                   │
└────────────────────────────────────────────────────────────┘
```

#### Progress Tracking (durante importação)

```
┌────────────────────────────────────────────────────────────┐
│  ☁️ Importando reuniões...                                  │
│                                                            │
│  ████████████░░░░░░░░  12 de 45 reuniões                   │
│                                                            │
│  ✓ Team Standup — sincronizado                              │
│  ✓ Client Review — sincronizado                             │
│  ⟳ Sprint Retrospective — enviando áudio (58%)             │
│  ○ Planning Meeting — aguardando                            │
│  ○ ...                                                      │
│                                                            │
│  Transcrições: 45/45 ✓    Resumos: 38/45 ✓                │
│  Áudio: 3/45 (uploading)  Embeddings: processando...       │
│                                                            │
│  ⚡ Importação roda em background. Pode fechar este modal.  │
│                                                            │
│  [Minimizar]  [Cancelar restantes]                         │
└────────────────────────────────────────────────────────────┘
```

#### Cenário Multi-Device

```
Dispositivo A (Desktop):                Dispositivo B (Laptop):
┌──────────────────────┐               ┌──────────────────────┐
│ 47 reuniões locais   │               │ 32 reuniões locais   │
│ (gravadas jan-fev)   │               │ (gravadas nov-fev)   │
└──────────┬───────────┘               └──────────┬───────────┘
           │                                      │
           │ Login + importa 45                    │ Login + importa 30
           │                                      │
           ▼                                      ▼
┌───────────────────────────────────────────────────────────┐
│                    Supabase Cloud                         │
│                                                           │
│  75 reuniões totais (45 do Desktop + 30 do Laptop)       │
│  Sem duplicatas — IDs locais diferentes, devices ≠       │
│  Cada meeting tem badge do device de origem               │
│                                                           │
│  Web UI / Qualquer device:                                │
│  ┌─────────────────────────────────────────────────────┐ │
│  │  Recent Meetings                                    │ │
│  │  ┌──────────────────────────────────────┐          │ │
│  │  │ Team Standup · 14/02 · 🖥️ Desktop  │          │ │
│  │  │ Client Call  · 13/02 · 💻 Laptop   │          │ │
│  │  │ Sprint Retro · 09/02 · 🖥️ Desktop  │          │ │
│  │  └──────────────────────────────────────┘          │ │
│  └─────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

#### Regras de Importação

| Regra | Descrição |
|-------|-----------|
| **Seletividade** | Usuário escolhe individualmente quais meetings importar |
| **Áudio opcional** | Checkbox global "incluir áudio" — transcrições e resumos sempre vão |
| **Workspace destino** | Todas as importadas vão para um workspace (default). Pode reorganizar depois |
| **Dedup** | `local_meeting_id` + `device_id` evita importar a mesma meeting duas vezes |
| **Background** | Importação roda em background — app continua funcional |
| **Resumível** | Se interrompida (app fechou), retoma de onde parou no próximo sync |
| **Re-importável** | Via Settings, pode importar meetings que não foram selecionadas antes |
| **Meetings futuras** | Após login, novas meetings sincronizam automaticamente (sem modal) |

#### Dados importados por meeting

```
Para cada meeting selecionada, o import faz:
1. INSERT meeting metadata → tabela meetings (Supabase)
2. INSERT todos os transcript segments → tabela transcripts
3. INSERT summary (se existir) → tabela summaries
4. INSERT chat messages (se existirem) → tabela chat_messages
5. UPLOAD áudio (se selecionado) → Supabase Storage
6. UPDATE cloud_sync_log local → status = 'synced'
7. TRIGGER Edge Function → gera embedding da transcrição
```

---

## 9. Supabase Edge Functions

### 9.1. generate-embedding

Gera embedding quando uma meeting é criada/atualizada.

```
POST /functions/v1/generate-embedding

Trigger: Database webhook (INSERT/UPDATE em meetings)

Fluxo:
1. Recebe meeting_id
2. Busca transcript completo da meeting
3. Chama OpenAI text-embedding-3-small (ou alternativa)
4. Salva embedding na coluna meetings.embedding
```

### 9.2. copilot-chat

Endpoint do Copilot para Web UI e API.

```
POST /functions/v1/copilot-chat

Body:
{
  "message": "O que o João falou sobre preço na última reunião?",
  "context_type": "global",         // "meeting" | "workspace" | "contact" | "global"
  "context_id": null,               // UUID ou null para global
  "conversation_id": "uuid"         // Para manter histórico
}

Fluxo:
1. Gera embedding da pergunta do usuário
2. Busca meetings relevantes via search_meetings_by_embedding()
3. Se context_type != "global", filtra pelo contexto
4. Monta prompt com transcrições/resumos relevantes como contexto
5. Chama LLM (Claude ou configurável)
6. Salva mensagem e resposta em chat_messages
7. Retorna resposta
```

---

## 10. Web UI (Next.js 16)

### 10.1. Stack

```
web/                               -- Novo diretório no monorepo
├── src/
│   ├── app/                       -- Next.js 16 App Router
│   │   ├── layout.tsx             -- Root layout + providers
│   │   ├── page.tsx               -- Landing/marketing page
│   │   ├── (auth)/
│   │   │   ├── login/page.tsx     -- Login (magic link, Google, GitHub)
│   │   │   └── callback/page.tsx  -- OAuth callback
│   │   └── (app)/                 -- Área autenticada
│   │       ├── layout.tsx         -- App shell (sidebar + topbar)
│   │       ├── page.tsx           -- Dashboard (overview)
│   │       ├── workspaces/
│   │       │   ├── page.tsx       -- Lista de workspaces
│   │       │   └── [id]/
│   │       │       └── page.tsx   -- Workspace detail (meetings list)
│   │       ├── meetings/
│   │       │   └── [id]/
│   │       │       └── page.tsx   -- Meeting detail (transcript + summary + chat)
│   │       ├── contacts/
│   │       │   ├── page.tsx       -- Lista de contacts
│   │       │   └── [id]/
│   │       │       └── page.tsx   -- Contact detail (interaction timeline)
│   │       ├── copilot/
│   │       │   └── page.tsx       -- Copilot full-page (chat global)
│   │       └── settings/
│   │           └── page.tsx       -- Settings (devices, sync, LLM config)
│   ├── components/
│   │   ├── ui/                    -- Shadcn/ui (mesmo design system do Tauri app)
│   │   ├── layout/                -- Sidebar, TopBar, CommandPalette
│   │   ├── meetings/              -- MeetingCard, TranscriptViewer, SummaryViewer
│   │   ├── copilot/               -- ChatPanel, ChatMessage, ChatInput
│   │   └── workspaces/            -- WorkspaceCard, WorkspaceSelector
│   ├── lib/
│   │   ├── supabase/
│   │   │   ├── client.ts          -- Supabase browser client
│   │   │   ├── server.ts          -- Supabase server client (RSC)
│   │   │   └── middleware.ts      -- Auth middleware
│   │   ├── hooks/                 -- React hooks (useMeetings, useWorkspaces, etc.)
│   │   └── utils.ts
│   └── styles/
│       └── globals.css            -- Tailwind 4
├── next.config.ts
├── tailwind.config.ts
├── package.json
└── tsconfig.json
```

### 10.2. Design System

> **IMPORTANTE**: Utilizar o skill `frontend-design` (frontend design specialist) para implementar
> cada página e componente da Web UI. O objetivo é entregar uma interface de nível produto,
> distintiva e sofisticada — não um template genérico de SaaS.

#### Referências de Design (por ordem de prioridade)

| Referência | O que extrair | URL |
|------------|---------------|-----|
| **Apple** | Espaçamento generoso, tipografia perfeita, micro-interações polidas, senso de premium | apple.com |
| **Linear** | Sidebar-driven navigation, dark mode elegante, command palette (⌘K), UI de issue/project como referência para meetings/workspaces | linear.app |
| **Vercel** | Tipografia Geist, design system neutro/moderno, dashboard cards, deploy status como referência para sync status | vercel.com |
| **Granola** | Split-screen notepad, distinção visual entre conteúdo do usuário (preto) e conteúdo AI (cinza), links clicáveis para timestamp do áudio, zero learning curve | granola.ai |

#### Princípios de Design

1. **Content-first**: A interface desaparece. O foco é no conteúdo da reunião (transcrição, resumo, chat). Sem decoração desnecessária.
2. **Densidade informacional controlada**: Como o Linear — muita informação disponível, mas hierarquia visual clara que guia o olho.
3. **Transições com propósito**: Animações Framer Motion que comunicam mudança de estado (entrou na meeting, sync completou), não decorativas.
4. **Distinção humano vs AI**: Seguir o padrão do Granola — conteúdo gerado por AI tem tratamento visual diferente (cor, tipografia, ou badge sutil) do conteúdo editado pelo usuário.
5. **Progressividade**: Interface simples no primeiro uso, features avançadas se revelam conforme o usuário explora (command palette, keyboard shortcuts, bulk actions).

#### Especificações Visuais

- **Tipografia**: Geist Sans (body) / Geist Mono (code, timestamps, metadata)
- **Cores**:
  - Background: `#09090b` (dark) / `#fafafa` (light)
  - Accent: Configurável por workspace (default: blue-500)
  - AI content: `text-muted-foreground` (cinza sutil, como Granola)
  - Success/Sync: green-500 · Error: red-500 · Warning: amber-500
- **Dark mode**: Default. Light mode disponível. Transição suave (sem flash).
- **Bordas**: `border-border/50` — sutis, quase imperceptíveis. Separação por espaçamento > bordas.
- **Sombras**: Mínimas. Só em modais e dropdowns. Cards sem sombra (só borda sutil).
- **Border radius**: `rounded-lg` (8px) padrão. `rounded-xl` (12px) para cards maiores.
- **Spacing scale**: 4px base (Tailwind default). Generoso — nunca apertado.

#### Layout Structure

```
┌────────────────────────────────────────────────────────────────┐
│  Sidebar (240px, colapsável para 64px)                         │
│  ┌──────┐ ┌──────────────────────────────────────────────────┐ │
│  │      │ │  Top Bar (48px)                                  │ │
│  │ Nav  │ │  ┌─────────────────┐              ┌────────────┐ │ │
│  │      │ │  │ Breadcrumb/Title│              │ ⌘K  👤 ☁️  │ │ │
│  │      │ │  └─────────────────┘              └────────────┘ │ │
│  │      │ ├──────────────────────────────────────────────────┤ │
│  │      │ │                                                  │ │
│  │      │ │  Main Content (flex, max-w-4xl centered)         │ │
│  │      │ │                                                  │ │
│  │      │ │                                    ┌───────────┐ │ │
│  │      │ │                                    │ Copilot   │ │ │
│  │      │ │                                    │ Panel     │ │ │
│  │      │ │                                    │ (320px,   │ │ │
│  │      │ │                                    │ toggle)   │ │ │
│  │      │ │                                    └───────────┘ │ │
│  └──────┘ └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

#### Keyboard-First UX

| Shortcut | Ação |
|----------|------|
| `⌘K` | Command palette (busca global, navegação, ações) |
| `⌘/` | Toggle Copilot panel |
| `⌘B` | Toggle sidebar |
| `⌘⇧P` | Busca de meetings |
| `⌘1-9` | Switch entre workspaces |
| `J/K` | Navegar entre meetings na lista |
| `Enter` | Abrir meeting selecionada |
| `Esc` | Voltar / fechar panel |

#### Componentes Chave (usar `frontend-design` skill para cada um)

| Componente | Referência Principal | Notas |
|-----------|---------------------|-------|
| **Sidebar** | Linear | Workspaces como "teams", meetings como "issues", colapsável |
| **Meeting Card** | Vercel dashboard cards | Título, duração, participantes, sync badge, hover preview |
| **Transcript Viewer** | Granola | Scroll synced com áudio, timestamps clicáveis, speaker labels |
| **Summary Viewer** | Apple Notes × Granola | Markdown renderizado, AI content em cinza, editável inline |
| **Copilot Chat** | Cursor AI sidebar | Streaming response, context pills, suggestion chips |
| **Command Palette** | Linear ⌘K | Fuzzy search, ações contextuais, keyboard navigation |
| **Login Page** | Vercel login | Minimal, centered card, social buttons, magic link |
| **Sync Indicator** | Vercel deploy status | Dot + label (Synced/Syncing/Offline), animação sutil |
| **Audio Player** | Apple Music mini-player | Waveform, play/pause, speed control, timestamp link |
| **Empty States** | Linear | Ilustrações sutis, copy amigável, CTA claro |

### 10.3. Páginas Principais

#### Dashboard (`/`)
```
┌─────────────────────────────────────────────────────────┐
│  ┌──────────┐                         ┌───────────────┐ │
│  │ Sidebar  │   Recent Meetings       │  Copilot      │ │
│  │          │   ┌─────────────────┐   │  Panel        │ │
│  │ □ Inbox  │   │ Team Standup    │   │               │ │
│  │          │   │ Hoje · 45min    │   │  "Resuma meu  │ │
│  │ Workspaces│  │ 3 participants  │   │   dia..."     │ │
│  │ ├ Projeto│   └─────────────────┘   │               │ │
│  │ ├ Acme   │   ┌─────────────────┐   │  [AI Response]│ │
│  │ └ Pessoal│   │ Client Review   │   │               │ │
│  │          │   │ Ontem · 1h20    │   │               │ │
│  │ Contacts │   │ 5 participants  │   │               │ │
│  │          │   └─────────────────┘   │               │ │
│  └──────────┘                         └───────────────┘ │
└─────────────────────────────────────────────────────────┘
```

#### Meeting Detail (`/meetings/[id]`)
```
┌─────────────────────────────────────────────────────────┐
│  ← Back    Team Standup    ▼ Workspace: Projeto Alpha   │
│─────────────────────────────────────────────────────────│
│                                                         │
│  [Transcript]  [Summary]  [Chat]     ▶ Play Audio       │
│  ─────────────────────────────────                      │
│                                                         │
│  # Resumo Executivo                  ┌───────────────┐  │
│  A reunião abordou o progresso...    │ Participants  │  │
│                                      │ ○ João Silva  │  │
│  ## Decisões Principais              │ ○ Maria Costa │  │
│  - Aprovado novo design...           │ ○ Pedro Lima  │  │
│                                      └───────────────┘  │
│  ## Action Items                     ┌───────────────┐  │
│  | Owner | Task        | Due |       │ Metadata      │  │
│  |-------|-------------|-----|       │ 45min · Desktop│  │
│  | João  | Revisar... | Seg |       │ Parakeet v2   │  │
│  | Maria | Preparar...| Qua |       │ Claude 3.5    │  │
│                                      └───────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 11. Fases de Implementação

### Fase 1: Fundação (Supabase + Auth) — ~1 semana

```
1.1  Criar projeto Supabase
1.2  Aplicar schema SQL (seção 7.1) — todas as tabelas, indexes, RLS
1.3  Configurar Storage bucket (meetily-audio, privado)
1.4  Configurar Auth providers:
     • Email Magic Link (habilitado por padrão)
     • Google OAuth (Client ID + Secret no Supabase Dashboard)
     • GitHub OAuth (Client ID + Secret)
     • Configurar redirect URLs:
       - https://app.meetily.ai/auth/callback (Web)
       - meetily://auth/callback (Tauri deep link)
1.5  Testar auth flow via Supabase Dashboard / curl
1.6  Deploy Edge Function: generate-embedding
```

**Entregável**: Infraestrutura cloud pronta. Auth testável (criar conta, login, ver JWT).

### Fase 2: Auth + Sync Engine no Tauri App — ~2 semanas

```
Auth no Desktop (semana 1):
2.1  Registrar deep link protocol "meetily://" no tauri.conf.json
2.2  Criar módulo cloud/auth.rs (PKCE flow, token storage, refresh)
2.3  Criar cloud/client.rs (Supabase HTTP client com auth headers)
2.4  Criar UI: CloudLoginBanner.tsx (banner sutil "Faça login para sync")
2.5  Criar UI: CloudLoginModal.tsx (botões Google, Email, GitHub)
2.6  Criar UI: CloudProfileMenu.tsx (avatar + dropdown na topbar)
2.7  Criar UI: CloudSyncIndicator.tsx (ícone synced/syncing/offline)
2.8  Primeiro login: registrar device no Supabase (tabela devices)

Sync Engine (semana 2):
2.9  Criar cloud/sync_engine.rs (local → cloud, bidirecional)
2.10 Criar cloud/sync_queue.rs (fila offline, persiste em SQLite)
2.11 Criar cloud/storage.rs (upload/download de áudio)
2.12 Criar UI: CloudOnboardingModal.tsx ("Importar 42 reuniões locais?")
2.13 Criar UI: CloudSyncSettings.tsx (toggle sync áudio, gerenciar devices)
2.14 Migração local → SQLite: tabela cloud_sync_log
2.15 Testar: gravar no Device A, ver no Supabase Dashboard
```

**Entregável**: App desktop com login opcional. Ao logar, meetings sincronizam para Supabase.

### Fase 3: Web UI + Auth Web — ~2-3 semanas

> **Instrução de implementação**: Usar o skill `frontend-design` (frontend design specialist)
> para implementar cada página e componente visual. Fornecer as referências de design
> (Apple, Linear, Vercel, Granola) detalhadas na seção 10.2 como contexto para cada invocação.
> O objetivo é resultado de nível produto — não template genérico.

```
Auth Web (início da fase):
3.1  Scaffold Next.js 16 project (Turbopack, Tailwind 4, Shadcn/ui, Geist font)
3.2  Configurar @supabase/ssr (createBrowserClient, createServerClient)
3.3  Criar middleware.ts (refresh token, proteção de rotas)
3.4  [frontend-design] Login page — ref: Vercel login (centered card, social buttons, magic link)
3.5  Criar (auth)/callback/route.ts — OAuth callback handler
3.6  Criar (auth)/logout/route.ts — logout + clear cookies
3.7  Testar: login via web, ver sessão, acessar rota protegida

App Web — Layout & Navegação:
3.8  [frontend-design] App shell — ref: Linear (sidebar colapsável, topbar, breadcrumbs)
3.9  [frontend-design] Command palette (⌘K) — ref: Linear (fuzzy search, ações contextuais)

App Web — Páginas:
3.10 [frontend-design] Dashboard — ref: Vercel dashboard (meeting cards, sync status, overview)
3.11 [frontend-design] Workspace page — ref: Linear projects (meeting list, filters, bulk actions)
3.12 [frontend-design] Meeting detail — ref: Granola (transcript viewer, AI summary em cinza, audio player)
3.13 [frontend-design] Copilot chat — ref: Cursor AI sidebar (streaming, context pills, suggestions)
3.14 [frontend-design] Contact page — ref: Linear (interaction timeline, cross-workspace)
3.15 [frontend-design] Settings page — ref: Vercel settings (tabs, forms, danger zone)

App Web — Polish:
3.16 Dark/light mode com transição suave (sem flash)
3.17 Responsive (sidebar collapses, mobile-friendly)
3.18 Keyboard shortcuts (⌘K, ⌘/, ⌘B, J/K navigation)
3.19 Empty states com ilustrações sutis e CTAs claros
3.20 Loading states (skeletons, não spinners)
```

**Entregável**: Web app funcional com design de nível produto. Login obrigatório. Mesma conta que o desktop.

### Fase 4: Copilot Avançado — ~1 semana

```
4.1  Deploy Edge Function: copilot-chat
4.2  Implementar 4 níveis de contexto (global, workspace, contact, meeting)
4.3  Busca semântica via pgvector
4.4  Streaming de resposta (SSE)
4.5  Histórico de conversas persistente
```

**Entregável**: Copilot funcional na web com busca semântica.

### Fase 5: Testes E2E + Deploy — ~1-2 semanas

```
Testes E2E (ver seção 12 para detalhamento completo):
5.1  Configurar Playwright (web/e2e/ e frontend/e2e/)
5.2  Configurar Supabase test project (ou supabase local via Docker)
5.3  Criar fixtures e seed data
5.4  Implementar test suites:
     • Auth E2E (desktop + web)
     • Sync E2E (local → cloud → web)
     • Import E2E (meetings existentes)
     • Web UI E2E (dashboard, meeting detail, copilot, search)
     • RLS Security tests (cross-user isolation)
5.5  Rust integration tests (cargo test) para cloud/ module
5.6  CI/CD: GitHub Actions workflow para E2E em cada PR

Deploy:
5.7  Deploy Web UI (Vercel ou Cloudflare Pages)
5.8  Domínio customizado + SSL
5.9  Monitoring básico (Supabase Dashboard + logs)
5.10 Documentação de uso
```

**Entregável**: Produto v1 completo, público.

---

## 12. Testes End-to-End (E2E)

> **Decisão**: Skip de testes unitários. Foco direto em testes E2E que validam fluxos completos
> com dados reais (SQLite local e Supabase remoto).

### 12.1. Estado Atual

O projeto **não possui nenhum teste** — nem unitário, nem E2E, nem integração. Nenhum framework de teste está configurado. As pastas de teste precisam ser criadas do zero em cada componente.

### 12.2. Estrutura de Pastas de Teste

```
meeting-minutes/
├── frontend/
│   ├── e2e/                                  # E2E tests do app Tauri (desktop)
│   │   ├── setup/
│   │   │   ├── global-setup.ts               # Seed SQLite local com dados de teste
│   │   │   ├── global-teardown.ts            # Cleanup
│   │   │   └── fixtures.ts                   # Test fixtures (meetings, transcripts)
│   │   ├── auth.spec.ts                      # Login/logout flow no desktop
│   │   ├── sync.spec.ts                      # Sync local → Supabase
│   │   ├── import.spec.ts                    # Importação de meetings existentes
│   │   ├── recording.spec.ts                 # Gravar → transcrever → sync
│   │   └── offline.spec.ts                   # Offline queue → reconnect → sync
│   │
│   ├── src-tauri/
│   │   └── tests/                            # Rust integration tests
│   │       ├── cloud_auth_test.rs            # PKCE flow, token refresh
│   │       ├── cloud_sync_test.rs            # Sync engine com Supabase real
│   │       ├── cloud_import_test.rs          # Import de meetings locais
│   │       ├── cloud_storage_test.rs         # Upload/download de áudio
│   │       └── helpers/
│   │           ├── mod.rs
│   │           ├── test_db.rs                # SQLite de teste com seed data
│   │           └── test_supabase.rs          # Supabase test client (service_role key)
│   │
│   └── package.json                          # + playwright, @playwright/test
│
├── web/                                       # Web UI (Next.js 16)
│   ├── e2e/                                   # E2E tests da Web UI
│   │   ├── setup/
│   │   │   ├── global-setup.ts               # Auth com Supabase (cria test user)
│   │   │   ├── global-teardown.ts            # Cleanup test data
│   │   │   ├── fixtures.ts                   # Seed meetings no Supabase
│   │   │   └── auth.setup.ts                 # Salva auth state pra reusar
│   │   ├── auth.spec.ts                      # Login, logout, redirect flows
│   │   ├── dashboard.spec.ts                 # Dashboard carrega meetings
│   │   ├── meeting-detail.spec.ts            # Transcript, summary, audio player
│   │   ├── workspace.spec.ts                 # CRUD workspace, organizar meetings
│   │   ├── copilot.spec.ts                   # Chat com contexto, busca semântica
│   │   ├── contact.spec.ts                   # CRUD contacts, timeline
│   │   ├── search.spec.ts                    # Command palette, busca global
│   │   └── sync-visibility.spec.ts           # Meeting do desktop visível na web
│   │
│   ├── playwright.config.ts                   # Playwright configuration
│   └── package.json                           # + @playwright/test
│
└── supabase/                                  # (Novo) Supabase project config
    └── tests/                                 # Testes de infraestrutura
        ├── rls.test.sql                       # Verifica que RLS bloqueia cross-user
        ├── schema.test.sql                    # Valida schema, constraints, indexes
        └── functions.test.ts                  # Testa Edge Functions isoladamente
```

### 12.3. Frameworks de Teste

| Componente | Framework | Justificativa |
|-----------|-----------|---------------|
| **Web UI E2E** | Playwright | Suporte nativo Next.js, multi-browser, fast, auto-wait |
| **Tauri Desktop E2E** | Playwright + Tauri driver | Tauri 2 tem suporte experimental a WebDriver |
| **Rust integration** | `cargo test` (built-in) | Testes de integração com `#[tokio::test]` |
| **Supabase RLS** | pgTAP ou SQL direto | Valida policies de segurança no banco |
| **Edge Functions** | Deno test | Runtime nativo das Edge Functions |

### 12.4. Test Suites — Detalhamento

#### Suite 1: Auth E2E (Desktop + Web)

```
auth.spec.ts (Web):
  ✓ Redireciona para /login se não autenticado
  ✓ Login com magic link (email) funciona
  ✓ Login com Google OAuth funciona
  ✓ Após login, redireciona para /dashboard
  ✓ Logout limpa sessão e redireciona para /login
  ✓ Token expirado → refresh automático → sessão mantida
  ✓ Rota protegida retorna 401 sem token (API)

auth.spec.ts (Desktop):
  ✓ App inicia sem login (modo local funcional)
  ✓ Banner "Faça login para sync" aparece
  ✓ Login abre browser com URL correta (PKCE)
  ✓ Deep link meetily://auth/callback é capturado
  ✓ Após login, CloudSyncIndicator aparece
  ✓ Logout volta ao modo local
```

#### Suite 2: Sync E2E

```
sync.spec.ts (Desktop):
  ✓ Meeting gravada localmente aparece no Supabase em < 30s
  ✓ Transcript segments são sincronizados com timestamps
  ✓ Summary é sincronizado quando gerado
  ✓ Chat messages são sincronizadas
  ✓ Áudio é uploaded quando toggle ativo
  ✓ Áudio NÃO é uploaded quando toggle inativo
  ✓ Sync funciona após período offline (queue)
  ✓ sync_version incrementa a cada update
  ✓ Dedup: mesma meeting não duplica ao re-sync

sync-visibility.spec.ts (Web):
  ✓ Meeting sincronizada do desktop aparece no dashboard web
  ✓ Transcript é visualizável na web
  ✓ Summary é visualizável e editável na web
  ✓ Edição na web sincroniza de volta (bidirecional)
  ✓ Badge de device de origem é exibido
```

#### Suite 3: Import E2E

```
import.spec.ts (Desktop):
  ✓ Primeiro login mostra modal de importação
  ✓ Lista correta de meetings locais é exibida
  ✓ Seleção individual funciona (check/uncheck)
  ✓ "Selecionar todas" / "Nenhuma" funciona
  ✓ Toggle de áudio calcula tamanho total
  ✓ Import roda em background com progress
  ✓ Meetings importadas aparecem no Supabase
  ✓ Meetings não selecionadas permanecem só locais
  ✓ "Agora não" → pode importar depois via Settings
  ✓ Segundo login no mesmo device não mostra modal novamente
```

#### Suite 4: Web UI E2E

```
dashboard.spec.ts:
  ✓ Dashboard carrega em < 2s (LCP)
  ✓ Meetings recentes aparecem ordenadas por data
  ✓ Meeting card mostra título, data, duração, device badge
  ✓ Click em meeting card navega para detail

meeting-detail.spec.ts:
  ✓ Transcript renderiza com timestamps
  ✓ Summary renderiza em markdown
  ✓ Tabs (Transcript / Summary / Chat) funcionam
  ✓ Audio player carrega e reproduz (se áudio sincronizado)
  ✓ Participantes são listados

workspace.spec.ts:
  ✓ Criar workspace com nome, cor, ícone
  ✓ Mover meeting para workspace
  ✓ Filtrar meetings por workspace
  ✓ Deletar workspace (meetings voltam para default)

copilot.spec.ts:
  ✓ Enviar mensagem e receber resposta (streaming)
  ✓ Contexto de meeting: resposta referencia transcrição correta
  ✓ Contexto global: busca cross-workspace funciona
  ✓ Histórico de conversa persiste entre page loads

contact.spec.ts:
  ✓ Criar contact com nome, email, empresa
  ✓ Associar contact a meeting
  ✓ Timeline mostra todas as interactions do contact

search.spec.ts:
  ✓ Command palette (⌘K) abre e fecha
  ✓ Busca por título de meeting retorna resultados
  ✓ Busca semântica retorna meetings relevantes
  ✓ Navegação via keyboard (J/K/Enter) funciona
```

#### Suite 5: Supabase Security (RLS)

```
rls.test.sql:
  ✓ User A não consegue SELECT meetings de User B
  ✓ User A não consegue UPDATE meetings de User B
  ✓ User A não consegue DELETE meetings de User B
  ✓ User A não acessa transcripts de meetings de User B
  ✓ User A não acessa summaries de User B
  ✓ User A não acessa chat_messages de User B
  ✓ User A não acessa devices de User B
  ✓ Service role key bypassa RLS (para Edge Functions)
  ✓ Anon key não acessa nenhum dado
```

### 12.5. Test Data (Fixtures)

```typescript
// fixtures.ts — dados de teste reutilizáveis

export const TEST_USER = {
  email: 'test@meetily.ai',
  password: 'test-password-e2e',
};

export const TEST_MEETINGS = [
  {
    title: 'E2E Test - Team Standup',
    recorded_at: '2026-02-15T10:00:00Z',
    duration_secs: 2700, // 45min
    transcript_segments: [
      { text: 'Bom dia pessoal, vamos começar o standup.', audio_start_time: 0.0, audio_end_time: 3.5 },
      { text: 'Ontem eu terminei a feature de login.', audio_start_time: 4.0, audio_end_time: 7.2 },
      // ... mais segmentos
    ],
    summary_markdown: '# Team Standup\n\n## Resumo\nReunião diária da equipe...',
  },
  {
    title: 'E2E Test - Client Review',
    recorded_at: '2026-02-14T14:00:00Z',
    duration_secs: 4800, // 1h20
    transcript_segments: [ /* ... */ ],
    summary_markdown: null, // Sem resumo ainda
  },
];

export const TEST_WORKSPACE = {
  name: 'E2E Test Workspace',
  type_hint: 'project',
  color: '#3b82f6',
};

export const TEST_CONTACT = {
  name: 'João Silva',
  email: 'joao@example.com',
  company: 'Acme Corp',
};
```

### 12.6. Ambiente de Teste

```
Opção A (recomendada): Supabase project dedicado para testes
  - Projeto separado: meetily-test (free tier)
  - Mesmo schema, dados resetados antes de cada run
  - Variáveis de ambiente: SUPABASE_TEST_URL, SUPABASE_TEST_ANON_KEY

Opção B: Supabase local (Docker)
  - supabase start (CLI oficial)
  - PostgreSQL local com mesmo schema
  - Mais rápido, sem dependência de rede
  - Ideal para CI/CD
```

```bash
# .env.test
SUPABASE_URL=https://xxxxx.supabase.co    # ou http://localhost:54321
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIs...
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIs...  # Só para seed/cleanup
TEST_USER_EMAIL=test@meetily.ai
TEST_USER_PASSWORD=test-password-e2e
```

### 12.7. CI/CD Integration

```yaml
# .github/workflows/e2e.yml
name: E2E Tests
on: [push, pull_request]

jobs:
  web-e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - run: cd web && pnpm install --frozen-lockfile
      - run: cd web && pnpm exec playwright install --with-deps
      - run: cd web && pnpm run build
      - run: cd web && pnpm exec playwright test
        env:
          SUPABASE_URL: ${{ secrets.SUPABASE_TEST_URL }}
          SUPABASE_ANON_KEY: ${{ secrets.SUPABASE_TEST_ANON_KEY }}

  rust-integration:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cd frontend/src-tauri && cargo test --test '*' -- --test-threads=1
        env:
          SUPABASE_URL: ${{ secrets.SUPABASE_TEST_URL }}
          SUPABASE_SERVICE_ROLE_KEY: ${{ secrets.SUPABASE_TEST_SERVICE_KEY }}
```

### 12.8. Regras de Teste

| Regra | Descrição |
|-------|-----------|
| **Sem unit tests** | Foco exclusivo em E2E e integration. Se funciona end-to-end, funciona. |
| **Dados reais** | Testes usam dados realistas (transcrições de verdade, não "lorem ipsum") |
| **Isolamento** | Cada test suite cria e limpa seus dados (não compartilha estado entre suites) |
| **Determinístico** | Testes não dependem de ordem de execução |
| **CI obrigatório** | Testes rodam em todo PR. Falhou = não faz merge |
| **Parallelism** | Playwright roda testes em paralelo (workers) |
| **Screenshots on fail** | Playwright captura screenshot e trace em falhas para debug |

---

## 13. Considerações Técnicas

### 13.1. Embedding Model

| Opção | Dimensões | Custo | Qualidade |
|-------|-----------|-------|-----------|
| OpenAI text-embedding-3-small | 1536 | $0.02/1M tokens | Excelente |
| OpenAI text-embedding-3-large | 3072 | $0.13/1M tokens | Superior |
| Ollama (local, nomic-embed) | 768 | Grátis | Boa |

**Recomendação**: `text-embedding-3-small` (1536d) — melhor custo-benefício. Uma reunião de 1h ≈ 10k tokens ≈ $0.0002. Milhares de reuniões por centavos.

### 13.2. Custos Estimados (Supabase)

| Recurso | Free Tier | Pro ($25/mês) |
|---------|-----------|---------------|
| Database | 500MB | 8GB |
| Storage | 1GB | 100GB |
| Edge Functions | 500k invocações | 2M invocações |
| Auth | 50k MAU | 100k MAU |
| Realtime | 200 conexões | 500 conexões |

**Para uso pessoal**: Free tier é suficiente por muito tempo.
**Para ~100 usuários**: Pro tier cobre com folga.

### 13.3. Segurança

- **RLS ativado** em todas as tabelas — usuário nunca acessa dados de outro
- **Signed URLs** para áudio — expiram em 1h, não são guessable
- **API keys do LLM** ficam apenas no dispositivo local, nunca vão para nuvem
- **JWT tokens** gerenciados pelo Supabase Auth (refresh automático)
- **Edge Functions** rodam em Deno (sandboxed)

### 13.4. Performance

- **pgvectorscale (DiskANN)** — index em SSD, escala até 50M vetores
- **Trigram index** — busca fuzzy rápida em texto de transcrição
- **Connection pooling** — Supabase inclui PgBouncer
- **Next.js RSC** — Server Components reduzem JavaScript no cliente
- **Turbopack** — Build/dev 10x mais rápido que Webpack

---

## 14. Riscos e Mitigações

| Risco | Impacto | Mitigação |
|-------|---------|-----------|
| Sync conflitos complexos | Médio | Last-write-wins simples; v2 pode adicionar CRDT |
| Custo de embeddings em escala | Baixo | text-embedding-3-small é barato; cache embeddings |
| Latência do Copilot | Médio | Streaming (SSE); cache de contexto frequente |
| Tamanho do áudio | Médio | Upload opcional; compressão antes de upload |
| Offline longo + muitas edições | Baixo | Queue persistente; sync_version resolve conflitos |
| Migração de dados existentes | Médio | Script de importação com preview antes de confirmar |

---

## 15. Métricas de Sucesso

- [ ] Meeting gravada no Device A aparece no Device B em < 30 segundos
- [ ] Web UI carrega dashboard em < 2 segundos (LCP)
- [ ] Copilot responde em < 5 segundos (incluindo busca semântica)
- [ ] Busca semântica retorna resultados relevantes (>70% satisfaction)
- [ ] Zero data leaks entre usuários (RLS funcional)
- [ ] App funciona 100% offline (sync ao reconectar)

---

## 16. Referência Futura (v2+)

> Não implementar agora. Apenas documentado para guiar decisões de schema.

### Multi-Channel (v2)
```sql
-- Nova tabela para emails (herda conceito de interaction)
CREATE TABLE public.emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    workspace_id UUID REFERENCES public.workspaces(id),
    subject TEXT,
    from_address TEXT,
    to_addresses TEXT[],
    body TEXT,
    thread_id TEXT,
    embedding VECTOR(1536),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Integrações CRM (v2)
```
Edge Function: crm-webhook
  → Escuta INSERT em meetings
  → Push para HubSpot/Salesforce API
  → Cria Contact + Log Activity
```

### MCP Server (v2)
```
MCP Tools expostas:
  - search_meetings(query, workspace?)
  - get_contact_history(name)
  - create_note(workspace, content)
  - list_action_items(workspace?, contact?)
```

---

*Documento gerado em 2026-02-16. Próximo passo: implementação da Fase 1.*
