# Mini-Plans — Meetily Cloud Sync v1

## Pattern de Execução

Cada mini-plan é um arquivo autocontido que pode ser executado em uma sessão nova do Claude Code.

### Como usar

1. Abrir nova sessão do Claude Code
2. Dizer: **"Execute o mini-plan-01"** (ou o número correspondente)
3. O agente lê o arquivo, executa os steps, e roda os testes E2E ao final
4. Se todos os testes passam → avançar para o próximo mini-plan

### Testes Cumulativos

Cada mini-plan inclui testes E2E que validam:
- Os fluxos implementados nesse mini-plan
- **TODOS os testes dos mini-plans anteriores** (cumulativo)

Exemplo: No mini-plan-03, os testes rodam:
- Testes do mini-plan-01 (schema, RLS)
- Testes do mini-plan-02 (auth desktop, sync)
- Testes do mini-plan-03 (web UI, auth web)

### Ordem de Execução (sequencial, obrigatória)

| # | Arquivo | Escopo | Depende de |
|---|---------|--------|------------|
| 1 | `mini-plan-01-supabase-foundation.md` | Supabase: schema, RLS, auth, storage, edge functions | Nenhum |
| 2 | `mini-plan-02-sync-engine-tauri.md` | Tauri: auth desktop, sync engine, import, UI cloud | Mini-plan 01 |
| 3 | `mini-plan-03-web-ui.md` | Next.js 16: auth web, dashboard, meetings, workspaces | Mini-plan 02 |
| 4 | `mini-plan-04-copilot.md` | Copilot: edge function, 4 níveis contexto, semantic search | Mini-plan 03 |
| 5 | `mini-plan-05-e2e-tests-deploy.md` | Testes completos, CI/CD, deploy, monitoring | Mini-plan 04 |

### Referência

- PRD completo: `PRD_CLOUD_SYNC_V1.md` (raiz do projeto)
- Instruções do projeto: `CLAUDE.md` (raiz do projeto)
- Memória persistente: `~/.claude/projects/-Users-luiz-git-meeting-minutes/memory/MEMORY.md`
