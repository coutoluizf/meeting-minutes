# Meetily Cloud Sync v1 — Visão Geral

> **Referência rápida para cada sessão de execução dos mini-plans.**
> PRD completo: [`PRD_CLOUD_SYNC_V1.md`](../PRD_CLOUD_SYNC_V1.md)
> Instruções do projeto: [`CLAUDE.md`](../CLAUDE.md)

---

## O Projeto

O **Meetily** é um app desktop (Tauri 2.6 + Next.js 14) de gravação e transcrição de reuniões, 100% local. O **Cloud Sync v1** adiciona sincronização na nuvem via Supabase, permitindo:

- Acessar reuniões de qualquer dispositivo (laptop ↔ desktop)
- Web UI para visualizar meetings de qualquer lugar
- Copilot na nuvem com busca semântica (pgvector)
- Importação seletiva de meetings locais existentes

**Princípio fundamental**: O app desktop continua funcionando 100% sem login. Login é opt-in para quem quer sync. Na Web UI, login é obrigatório.

---

## Arquitetura

```
┌─────────────────────────────────────────────────────────────┐
│              App Tauri (Desktop) — existente                 │
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │  Next.js UI │  │ Rust Backend │  │  Parakeet (STT)   │  │
│  │  (React/TS) │  │ (Audio+IPC)  │  │  (local, GPU)     │  │
│  └─────────────┘  └──────────────┘  └───────────────────┘  │
│                          │                                   │
│                   cloud/ module (NOVO)                       │
│                   ├── auth.rs                                │
│                   ├── client.rs                              │
│                   ├── sync_engine.rs                         │
│                   ├── sync_queue.rs                          │
│                   └── storage.rs                             │
└──────────────────────────┬──────────────────────────────────┘
                           │ HTTPS (bidirecional)
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Supabase (NOVO)                           │
│                                                             │
│  ┌────────┐ ┌────────────┐ ┌─────────┐ ┌────────────────┐  │
│  │  Auth  │ │ PostgreSQL │ │ Storage │ │ Edge Functions │  │
│  │        │ │ + pgvector │ │ (áudio) │ │ (embedding,    │  │
│  │        │ │ + DiskANN  │ │         │ │  copilot-chat) │  │
│  └────────┘ └────────────┘ └─────────┘ └────────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                           │ HTTPS
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                Web UI (NOVO) — Next.js 16                    │
│                                                             │
│  ┌────────────┐ ┌──────────────┐ ┌────────────────────────┐ │
│  │ Dashboard  │ │ Meeting      │ │ Copilot (4 contextos:  │ │
│  │ Workspaces │ │ Viewer       │ │ global, workspace,     │ │
│  │ Contacts   │ │ Transcript   │ │ contact, meeting)      │ │
│  │ Settings   │ │ + Summary    │ │ SSE streaming          │ │
│  └────────────┘ └──────────────┘ └────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Stack Técnica

| Camada | Tecnologia | Notas |
|--------|-----------|-------|
| **Desktop App** | Tauri 2.6 (Rust + Next.js 14) | Existente, novo módulo `cloud/` |
| **Web UI** | Next.js 16.1 (Turbopack) + React 19 | Novo, diretório `web/` |
| **Styling** | Tailwind 4 + Shadcn/ui + Framer Motion | Geist Sans/Mono |
| **Auth** | Supabase Auth (Magic Link, Google, GitHub) | PKCE no desktop, SSR na web |
| **Database** | Supabase PostgreSQL 16+ | RLS em todas as tabelas |
| **Vector** | pgvector + pgvectorscale (DiskANN) | 1536d, OpenAI embedding |
| **Storage** | Supabase Storage | Bucket privado `meetily-audio` |
| **Edge Functions** | Supabase (Deno) | `generate-embedding`, `copilot-chat` |
| **LLM (Copilot)** | Claude (Anthropic API) | Streaming SSE |
| **Embeddings** | OpenAI text-embedding-3-small | $0.02/1M tokens |
| **Deploy** | Vercel (Web UI) | Auto-deploy via GitHub Actions |
| **Testes** | Playwright (web), cargo test (Rust), pgTAP (SQL) | 118 testes E2E |

---

## Schema do Banco (Supabase PostgreSQL)

| Tabela | Propósito | Relações |
|--------|-----------|----------|
| `profiles` | Extensão de auth.users (display_name, avatar) | 1:1 com auth.users |
| `devices` | Dispositivos registrados do usuário | N:1 com auth.users |
| `workspaces` | Organização flexível (projeto, cliente, pessoal) | N:1 com auth.users |
| `contacts` | Entidade global, cross-workspace | N:1 com auth.users |
| `workspace_contacts` | Relação N:N workspaces ↔ contacts | Junction table |
| `meetings` | Entidade principal — reunião gravada | N:1 com workspaces, tem `embedding VECTOR(1536)` |
| `transcripts` | Segmentos de transcrição com timestamps | N:1 com meetings |
| `summaries` | Resumos gerados por AI (markdown + JSON) | N:1 com meetings |
| `chat_messages` | Conversas do Copilot (4 contextos) | Polimórfico via `context_type` + `context_id` |
| `meeting_contacts` | Relação N:N meetings ↔ contacts (participantes) | Junction table |

**Segurança (RLS)**: RLS habilitado em todas as tabelas. Cada usuário só acessa seus próprios dados. Service role key bypassa RLS (Edge Functions).

**Acesso a dados**: Clientes (Web UI e Tauri) acessam Supabase **diretamente** via `supabase-js` + `anon_key` + RLS. **Sem API intermediária** para dados do usuário. O Supabase + RLS é o backend.

**Sync**: Last-write-wins via `sync_version` (incrementa a cada update). Dedup via `local_meeting_id` + `device_id`.

**Supabase Project**: `meetpix` (Organization: meetpix PRO) · Region: Americas · Credenciais em `.env.local`

---

## Design System

> Usar skill `frontend-design` para CADA componente visual da Web UI.

| Referência | O que extrair |
|------------|---------------|
| **Apple** | Espaçamento generoso, tipografia perfeita, micro-interações |
| **Linear** | Sidebar navigation, dark mode, ⌘K command palette |
| **Vercel** | Geist font, dashboard cards, sync status indicators |
| **Granola** | Split notepad, AI content em cinza, timestamps clicáveis |

**Cores**: Background `#09090b` (dark) / `#fafafa` (light), accent configurável por workspace.
**Tipografia**: Geist Sans (body), Geist Mono (code/timestamps).
**Dark mode**: Default. Transição suave (sem flash).

---

## Mapa dos Mini-Plans

| # | Mini-Plan | Escopo | Testes | Status |
|---|-----------|--------|--------|--------|
| 1 | [Supabase Foundation](mini-plan-01-supabase-foundation.md) | Schema SQL, RLS, Storage, Edge Function embedding, Auth providers | 32 | ⬜ Pendente |
| 2 | [Sync Engine (Tauri)](mini-plan-02-sync-engine-tauri.md) | Módulo `cloud/` Rust, auth desktop (PKCE), sync bidirecional, import, UI cloud | 14 | ⬜ Pendente |
| 3 | [Web UI (Next.js 16)](mini-plan-03-web-ui.md) | Scaffold, auth web, app shell, dashboard, meeting detail, workspaces, contacts, settings | 26 | ⬜ Pendente |
| 4 | [Copilot Cloud](mini-plan-04-copilot.md) | Edge Function copilot-chat, 4 contextos, SSE streaming, busca semântica, chat UI | 27 | ⬜ Pendente |
| 5 | [E2E Tests + Deploy](mini-plan-05-e2e-tests-deploy.md) | Cross-suite tests, CI/CD GitHub Actions, Vercel deploy, monitoring, docs | 19 | ⬜ Pendente |
| | **Total** | | **118** | |

### Dependências

```
MP01 → MP02 → MP03 → MP04 → MP05
```

Sequencial obrigatório. Cada mini-plan depende do anterior estar completo com todos os testes passando.

### Testes Cumulativos

Cada mini-plan roda seus testes + TODOS os anteriores:

```
MP01: 32 testes
MP02: 32 + 14 = 46 testes
MP03: 32 + 14 + 26 = 72 testes
MP04: 32 + 14 + 26 + 27 = 99 testes
MP05: 32 + 14 + 26 + 27 + 19 = 118 testes
```

---

## Como Executar

1. Abrir nova sessão do Claude Code
2. Dizer: **"Execute o mini-plan-01"** (ou o número correspondente)
3. O agente lê o arquivo, executa os steps, e roda os testes cumulativos
4. Se todos os testes passam → avançar para o próximo mini-plan

---

## Fora de Escopo (v1)

- Multi-channel (emails, Slack) — v2
- Integrações CRM (Salesforce, HubSpot) — v2
- MCP Server / API pública — v2
- Collaborative features (multi-user no mesmo workspace)
- Gravação via web (exclusiva do desktop)
- Copilot no app desktop (permanece local)

---

## Referências

- **PRD Completo**: [`PRD_CLOUD_SYNC_V1.md`](../PRD_CLOUD_SYNC_V1.md) (~1750 linhas)
- **Instruções do Projeto**: [`CLAUDE.md`](../CLAUDE.md)
- **Memória Persistente**: `~/.claude/projects/-Users-luiz-git-meeting-minutes/memory/MEMORY.md`

---

*Última atualização: 2026-02-16*
