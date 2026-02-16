# Mini-Plan 05: E2E Tests Completos + CI/CD + Deploy

> **Status**: Pendente
> **Depende de**: Mini-Plan 04 (Copilot) ✅ completo
> **PRD Referência**: Seções 12 (Testes E2E), 11 (Fase 5) do `PRD_CLOUD_SYNC_V1.md`
> **Estimativa**: ~1-2 sessões (usar Team Agents: Tests + Deploy em paralelo)

---

## Contexto

Toda a funcionalidade Cloud Sync v1 está implementada: Supabase (MP01), Sync Engine (MP02), Web UI (MP03), Copilot (MP04). Cada mini-plan anterior criou testes para seu escopo. Este mini-plan consolida os testes, preenche gaps, configura CI/CD, e faz o deploy da Web UI para produção.

## Pré-requisitos

- [ ] Mini-Plans 01 a 04 completos e todos os 99 testes passando
- [ ] Supabase project de teste configurado (separado do production)
- [ ] Domínio configurado (ex: app.meetily.ai) — ou usar Vercel preview URL
- [ ] Conta Vercel (ou Cloudflare Pages) para deploy
- [ ] Secrets configurados no GitHub repo para CI

## Escopo

### O que FAZER neste mini-plan:
1. Consolidar e review todos os test suites (MP01-MP04)
2. Preencher gaps de testes (cross-suite, edge cases)
3. Configurar Supabase local para testes (Docker)
4. CI/CD completo com GitHub Actions
5. Deploy da Web UI (Vercel)
6. Monitoring básico e health checks
7. Documentação de uso e deploy

### O que NÃO fazer:
- Performance testing / load testing (v2)
- Staging environment separado (v2)
- Feature flags system (v2)
- A/B testing infrastructure (v2)

---

## Steps de Implementação

### Step 1: Supabase Local para Testes (Docker)

**1.1 — Configurar Supabase CLI:**
```bash
# Instalar Supabase CLI (se não instalado)
pnpm add -g supabase

# Inicializar config no projeto
cd /Users/luiz/git/meeting-minutes/supabase
supabase init  # Cria supabase/config.toml se não existir
```

**1.2 — Configurar `supabase/config.toml`:**
```toml
[project]
id = "meetily-local"

[db]
port = 54322
shadow_port = 54320
major_version = 15

[auth]
site_url = "http://localhost:3000"
additional_redirect_urls = ["http://localhost:3000/auth/callback"]
enable_signup = true

[auth.email]
enable_signup = true
enable_confirmations = false  # Desabilitar confirmação para testes

[storage]
enabled = true
file_size_limit = "500MB"
```

**1.3 — Script de setup para testes locais:**

Criar `supabase/test-setup.sh`:
```bash
#!/bin/bash
# Inicia Supabase local, aplica migrations, seed dados de teste
#
# Uso: ./supabase/test-setup.sh
#
# Passos:
# 1. supabase start (PostgreSQL, Auth, Storage, Edge Functions locais)
# 2. Aplicar todas as migrations (001-014)
# 3. Seed com dados de teste (2 test users, meetings, transcripts, etc.)
# 4. Exportar variáveis de ambiente para .env.test
# 5. Verificar que tudo está rodando (health check)
```

**1.4 — Seed data completo:**

Criar `supabase/seed.sql`:
```sql
-- Seed data para testes E2E
--
-- User A: test-user-a@meetily.ai (via Supabase Auth local)
-- User B: test-user-b@meetily.ai (para testar isolamento RLS)
--
-- User A tem:
--   - 2 workspaces (Projeto Alpha, Acme Corp)
--   - 5 meetings (com transcripts e summaries)
--   - 3 contacts (João Silva, Maria Costa, Pedro Lima)
--   - 1 default workspace (Inbox)
--   - 1 device (MacBook de Teste)
--
-- User B tem:
--   - 1 workspace + 1 meeting (para testar que A não acessa)
--
-- Meetings do User A incluem:
--   - Meeting com transcrição completa e resumo (para testar copilot)
--   - Meeting sem resumo (para testar empty states)
--   - Meeting com áudio sincronizado (para testar audio player)
--   - Meeting sem áudio (para testar sem player)
--   - Meeting com embedding (para testar busca semântica)
```

### Step 2: Consolidação de Test Suites

**2.1 — Review dos testes existentes (MP01-MP04):**

| Mini-Plan | Suite | Testes | Status |
|-----------|-------|--------|--------|
| MP01 | Schema Validation | 14 | Revisar |
| MP01 | RLS Security | 11 | Revisar |
| MP01 | Storage Policies | 4 | Revisar |
| MP01 | Edge Function (embedding) | 3 | Revisar |
| MP02 | Rust integration (cloud/) | 14 | Revisar |
| MP03 | Web UI (Playwright) | 26 | Revisar |
| MP04 | Copilot (Playwright) | 16 | Revisar |
| MP04 | Copilot Edge Function | 11 | Revisar |
| **Total** | | **99** | |

**2.2 — Testes de gap (cross-suite) a adicionar:**

```typescript
// cross-suite.spec.ts — Testes que validam fluxos entre mini-plans
test('meeting gravada no desktop (mock) aparece na web UI')
test('edição de summary na web sincroniza (verificar Supabase)')
test('meeting importada (mock import) tem embedding gerado')
test('copilot responde sobre meeting recém-sincronizada')
test('workspace deletado na web remove meetings do dashboard')
test('contact associado a meeting aparece no meeting detail')
test('busca semântica retorna meeting com embedding similar')
test('meeting sem embedding não aparece em busca semântica')
```

**2.3 — Edge cases a adicionar:**

```typescript
// edge-cases.spec.ts
test('meeting com transcrição vazia mostra empty state')
test('meeting com título muito longo é truncado corretamente')
test('workspace com 100+ meetings pagina corretamente')
test('copilot com mensagem muito longa (>4000 chars) é aceita')
test('login com email inválido mostra erro adequado')
test('upload de áudio grande (>100MB) mostra progresso')
test('sessão expirada redireciona para login com return URL')
test('deep link meetily://auth/callback com erro mostra mensagem')
```

### Step 3: Playwright Configuration Unificada

**3.1 — Web Playwright config** (`web/playwright.config.ts`):
```typescript
// Configuração principal do Playwright para a Web UI
//
// - baseURL: http://localhost:3000 (Next.js dev server)
// - webServer: { command: 'pnpm dev', port: 3000 }
// - projects:
//   - setup: auth.setup.ts (cria test user, salva auth state)
//   - chromium: testes em Chrome
//   - firefox: testes em Firefox (optional, CI only)
//   - mobile: testes em viewport mobile (375x812)
// - retries: 2 em CI, 0 em local
// - reporter: ['html', 'json'] em CI, ['list'] em local
// - globalSetup: e2e/setup/global-setup.ts
// - globalTeardown: e2e/setup/global-teardown.ts
// - use:
//   - screenshot: 'only-on-failure'
//   - trace: 'on-first-retry'
//   - video: 'on-first-retry'
```

**3.2 — Auth setup** (`web/e2e/setup/auth.setup.ts`):
```typescript
// Cria test user no Supabase e salva auth state
//
// 1. Criar user via Supabase Admin API (service_role key)
// 2. Login via page (navigate to /login, fill email, submit)
// 3. Salvar storage state (cookies + localStorage) para reuso
// 4. Todos os testes usam esse auth state (não fazem login individual)
```

**3.3 — Global setup/teardown:**
```typescript
// global-setup.ts:
// 1. Verificar que Supabase está rodando (health check)
// 2. Verificar que Next.js está rodando
// 3. Seed dados de teste via Supabase service_role API
// 4. Aguardar embeddings serem gerados (polling)

// global-teardown.ts:
// 1. Deletar todos os dados de teste (via service_role)
// 2. Deletar test users
```

### Step 4: Rust Integration Tests — Revisão

**4.1 — Verificar test helpers** (`frontend/src-tauri/tests/helpers/`):
```rust
// test_db.rs — Helper para SQLite de teste
// - Cria banco SQLite em /tmp/ com schema migrado
// - Seed com meetings, transcripts, summaries de teste
// - Cleanup automático (Drop trait)

// test_supabase.rs — Helper para Supabase de teste
// - Client com service_role key (bypassa RLS)
// - Funções: create_test_user, seed_meetings, cleanup
// - Configura via env vars: SUPABASE_TEST_URL, SUPABASE_TEST_SERVICE_KEY
```

**4.2 — Verificar que testes de cloud usam Supabase local:**
```rust
// Os testes do MP02 devem apontar para Supabase local:
// SUPABASE_URL=http://localhost:54321
// SUPABASE_ANON_KEY=<local anon key>
// SUPABASE_SERVICE_ROLE_KEY=<local service role key>
```

### Step 5: CI/CD — GitHub Actions

**5.1 — Workflow principal** (`.github/workflows/e2e.yml`):

```yaml
# .github/workflows/e2e.yml
name: E2E Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  SUPABASE_URL: http://localhost:54321
  SUPABASE_ANON_KEY: ${{ secrets.SUPABASE_LOCAL_ANON_KEY }}
  SUPABASE_SERVICE_ROLE_KEY: ${{ secrets.SUPABASE_LOCAL_SERVICE_KEY }}
  OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}

jobs:
  # Job 1: Supabase schema + RLS tests
  supabase-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: supabase/setup-cli@v1
      - run: supabase start
        working-directory: ./supabase
      - run: ./run-tests.sh
        working-directory: ./supabase

  # Job 2: Rust integration tests
  rust-integration:
    runs-on: ubuntu-latest
    needs: [supabase-tests]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: supabase/setup-cli@v1
      - run: supabase start
        working-directory: ./supabase
      - run: cargo test --test '*cloud*' -- --test-threads=1
        working-directory: ./frontend/src-tauri

  # Job 3: Web E2E tests
  web-e2e:
    runs-on: ubuntu-latest
    needs: [supabase-tests]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - uses: supabase/setup-cli@v1
      - run: supabase start
        working-directory: ./supabase
      - run: pnpm install --frozen-lockfile
        working-directory: ./web
      - run: pnpm exec playwright install --with-deps
        working-directory: ./web
      - run: pnpm run build
        working-directory: ./web
      - run: pnpm exec playwright test
        working-directory: ./web
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-report
          path: web/playwright-report/
          retention-days: 7

  # Job 4: Edge Function tests
  edge-function-tests:
    runs-on: ubuntu-latest
    needs: [supabase-tests]
    steps:
      - uses: actions/checkout@v4
      - uses: denoland/setup-deno@v2
      - uses: supabase/setup-cli@v1
      - run: supabase start
        working-directory: ./supabase
      - run: supabase functions serve &
        working-directory: ./supabase
      - run: deno test tests/
        working-directory: ./supabase
```

**5.2 — Workflow de deploy** (`.github/workflows/deploy.yml`):

```yaml
# .github/workflows/deploy.yml
name: Deploy Web UI

on:
  push:
    branches: [main]
    paths:
      - 'web/**'
      - 'supabase/migrations/**'

jobs:
  deploy:
    runs-on: ubuntu-latest
    # Só roda se E2E tests passaram
    needs: []  # Referencia o workflow e2e.yml via required status checks
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: pnpm install --frozen-lockfile
        working-directory: ./web
      - run: pnpm run build
        working-directory: ./web
        env:
          NEXT_PUBLIC_SUPABASE_URL: ${{ secrets.SUPABASE_PROD_URL }}
          NEXT_PUBLIC_SUPABASE_ANON_KEY: ${{ secrets.SUPABASE_PROD_ANON_KEY }}
      # Deploy com Vercel CLI
      - run: pnpm add -g vercel
      - run: vercel deploy --prod --token=${{ secrets.VERCEL_TOKEN }}
        working-directory: ./web
```

### Step 6: Deploy da Web UI

**6.1 — Configurar Vercel:**
```bash
# Instalar Vercel CLI
pnpm add -g vercel

# Login e link ao projeto
cd web
vercel login
vercel link  # Conectar ao projeto Vercel

# Configurar environment variables no Vercel:
# NEXT_PUBLIC_SUPABASE_URL = https://xxxxx.supabase.co
# NEXT_PUBLIC_SUPABASE_ANON_KEY = eyJhbGciOi...
```

**6.2 — next.config.ts para produção:**
```typescript
// web/next.config.ts
//
// Configurações críticas para deploy:
// - output: 'standalone' (para Vercel/Docker)
// - images: { domains: ['xxxxx.supabase.co'] }  // Avatar URLs
// - experimental: { turbopack: true }  // Turbopack em dev
// - headers: CORS headers para Edge Functions
```

**6.3 — Domínio customizado (instruções para o usuário):**
```
Vercel Dashboard:
1. Settings → Domains → Add domain: app.meetily.ai
2. Configurar DNS no registrar:
   - CNAME: app → cname.vercel-dns.com
3. SSL é automático (Vercel gerencia Let's Encrypt)

Supabase Dashboard:
1. Authentication → URL Configuration → Site URL: https://app.meetily.ai
2. Authentication → URL Configuration → Redirect URLs:
   - https://app.meetily.ai/auth/callback
   - http://localhost:3000/auth/callback (dev)
   - meetily://auth/callback (desktop)
```

### Step 7: Health Checks e Monitoring

**7.1 — Health check endpoint** (`web/src/app/api/health/route.ts`):
```typescript
// GET /api/health
//
// Verifica:
// 1. Next.js está respondendo (✓ se chegou aqui)
// 2. Supabase está acessível (SELECT 1 via client)
// 3. Auth está funcional (verificar session endpoint)
//
// Response:
// { status: "healthy", timestamp: "...", checks: { supabase: true, auth: true } }
// ou
// { status: "degraded", timestamp: "...", checks: { supabase: true, auth: false } }
// ou
// { status: "unhealthy", timestamp: "...", checks: { supabase: false, auth: false } }
```

**7.2 — Monitoring básico:**
```
Supabase Dashboard:
- Database → Reports: Queries/sec, cache hit rate
- Auth → Logs: Login attempts, errors
- Edge Functions → Logs: Invocations, errors, latency
- Storage → Usage: Storage consumed

Vercel Dashboard:
- Analytics: Page views, LCP, FID, CLS
- Functions → Logs: API route errors
- Deployments: Build status, deploy history

GitHub:
- Actions → Workflows: E2E test status, deploy status
- Required status checks: "E2E Tests" deve passar para merge
```

### Step 8: Documentação

**8.1 — Atualizar CLAUDE.md** com informações do cloud sync:
```
Adicionar ao CLAUDE.md:
- Seção "Cloud Sync" com visão geral da arquitetura
- Comandos para rodar testes E2E
- Como configurar Supabase local para desenvolvimento
- Variáveis de ambiente necessárias
- Deploy workflow
```

**8.2 — Criar `web/README.md`** (instruções específicas da Web UI):
```markdown
# Meetily Web UI

## Setup Local
1. Instalar dependências: pnpm install
2. Configurar .env.local com Supabase URLs
3. Rodar: pnpm dev
4. Acessar: http://localhost:3000

## Testes E2E
1. Iniciar Supabase local: cd supabase && supabase start
2. Rodar testes: pnpm exec playwright test
3. Ver relatório: pnpm exec playwright show-report

## Deploy
- Automático via GitHub Actions ao push em main
- Manual: vercel deploy --prod
```

**8.3 — Criar `supabase/README.md`** (instruções do Supabase):
```markdown
# Meetily Supabase

## Setup Local
1. Instalar CLI: pnpm add -g supabase
2. Iniciar: supabase start
3. Aplicar migrations: supabase db reset
4. Seed dados: supabase db reset (aplica seed.sql automaticamente)

## Migrations
- Criar nova: supabase migration new <nome>
- Aplicar: supabase db push (remoto) ou supabase db reset (local)

## Edge Functions
- Servir localmente: supabase functions serve
- Deploy: supabase functions deploy <nome>
- Logs: supabase functions logs <nome>

## Testes
- Schema + RLS: ./run-tests.sh
- Edge Functions: deno test tests/
```

---

## Testes E2E — Mini-Plan 05

### Novos Testes (cross-suite + edge cases)

```typescript
// cross-suite.spec.ts (8 testes)
test('meeting gravada no desktop (mock) aparece na web UI')
test('edição de summary na web sincroniza (verificar Supabase)')
test('meeting importada (mock import) tem embedding gerado')
test('copilot responde sobre meeting recém-sincronizada')
test('workspace deletado na web remove meetings do dashboard')
test('contact associado a meeting aparece no meeting detail')
test('busca semântica retorna meeting com embedding similar')
test('meeting sem embedding não aparece em busca semântica')

// edge-cases.spec.ts (8 testes)
test('meeting com transcrição vazia mostra empty state')
test('meeting com título muito longo é truncado corretamente')
test('workspace com 100+ meetings pagina corretamente')
test('copilot com mensagem muito longa (>4000 chars) é aceita')
test('login com email inválido mostra erro adequado')
test('upload de áudio grande (>100MB) mostra progresso')
test('sessão expirada redireciona para login com return URL')
test('deep link meetily://auth/callback com erro mostra mensagem')

// health.spec.ts (3 testes)
test('GET /api/health returns healthy status')
test('health check detects Supabase down')
test('health check returns correct response format')
```

### Comando de Testes Cumulativos (FINAL)

```bash
#!/bin/bash
# run-all-tests.sh — Executa TODOS os testes do projeto Cloud Sync v1
# Localização: raiz do projeto

set -e  # Falha se qualquer teste falhar

echo "======================================"
echo "  Meetily Cloud Sync v1 — Full E2E    "
echo "======================================"

echo ""
echo "=== Mini-Plan 01: Supabase Foundation ==="
cd supabase && ./run-tests.sh
echo "✓ MP01: 32 testes passaram"

echo ""
echo "=== Mini-Plan 02: Sync Engine ==="
cd ../frontend/src-tauri && cargo test --test '*cloud*' -- --test-threads=1
echo "✓ MP02: 14 testes passaram"

echo ""
echo "=== Mini-Plan 03: Web UI ==="
cd ../../web && pnpm exec playwright test --grep-invert '@copilot'
echo "✓ MP03: 26 testes passaram"

echo ""
echo "=== Mini-Plan 04: Copilot ==="
pnpm exec playwright test e2e/copilot.spec.ts
cd ../supabase && deno test tests/copilot.test.ts
echo "✓ MP04: 27 testes passaram"

echo ""
echo "=== Mini-Plan 05: Cross-Suite + Edge Cases ==="
cd ../web && pnpm exec playwright test e2e/cross-suite.spec.ts e2e/edge-cases.spec.ts e2e/health.spec.ts
echo "✓ MP05: 19 testes passaram"

echo ""
echo "======================================"
echo "  RESULTADO FINAL                     "
echo "  32 + 14 + 26 + 27 + 19 = 118 testes"
echo "  ✓ TODOS PASSARAM                    "
echo "======================================"
```

---

## Checklist de Validação Final (Produto v1 Completo)

### Infraestrutura
- [ ] Supabase local (Docker) funciona para desenvolvimento
- [ ] Supabase production project configurado
- [ ] Todas as 14 migrations aplicadas
- [ ] Edge Functions deployed (generate-embedding + copilot-chat)
- [ ] Storage bucket configurado e funcional

### Auth
- [ ] Login via Google OAuth funciona (web + desktop)
- [ ] Login via Magic Link funciona (web + desktop)
- [ ] Logout limpa sessão corretamente
- [ ] Token refresh automático funciona
- [ ] RLS bloqueia acesso cross-user (testado)

### Sync
- [ ] Meeting gravada no desktop sincroniza para Supabase
- [ ] Meeting visível na Web UI após sync
- [ ] Sync bidirecional funciona (edição web → desktop)
- [ ] Importação de meetings locais existentes funciona
- [ ] Áudio upload (opcional) funciona
- [ ] Sync offline → queue → reconnect funciona

### Web UI
- [ ] Dashboard mostra meetings ordenadas por data
- [ ] Meeting detail renderiza transcript, summary, audio
- [ ] Workspace CRUD funciona
- [ ] Contact CRUD funciona
- [ ] Command palette (⌘K) busca e navega
- [ ] Dark mode / light mode toggle funciona
- [ ] Responsive em mobile
- [ ] Design segue referências (Apple/Linear/Vercel/Granola)

### Copilot
- [ ] Chat no meeting detail funciona (contexto meeting)
- [ ] Chat global funciona (busca semântica)
- [ ] Streaming SSE funciona
- [ ] Source citations aparecem e são clicáveis
- [ ] Histórico persiste

### Deploy & CI/CD
- [ ] Web UI deployed no Vercel (ou Cloudflare Pages)
- [ ] Domínio customizado configurado com SSL
- [ ] Health check endpoint respondendo
- [ ] GitHub Actions E2E workflow configurado
- [ ] E2E tests rodam em todo PR
- [ ] Deploy automático em push para main

### Documentação
- [ ] CLAUDE.md atualizado com seção Cloud Sync
- [ ] web/README.md criado
- [ ] supabase/README.md criado
- [ ] .env.example atualizado com todas as variáveis

### Testes
- [ ] **118 testes cumulativos passando**
  - 32 (MP01: Supabase Foundation)
  - 14 (MP02: Sync Engine)
  - 26 (MP03: Web UI)
  - 27 (MP04: Copilot)
  - 19 (MP05: Cross-Suite + Edge Cases + Health)

---

## Produto v1 Entregue!

Após completar este mini-plan, o Meetily Cloud Sync v1 está funcional:

```
✅ Gravação local (como antes) — sem regressão
✅ Login opcional no desktop — habilita sync
✅ Sync automático entre dispositivos
✅ Importação de meetings existentes
✅ Web UI sofisticada (Apple/Linear/Vercel/Granola quality)
✅ Copilot com busca semântica em todas as meetings
✅ 4 níveis de contexto (global, workspace, contact, meeting)
✅ CI/CD com testes E2E obrigatórios
✅ Deploy automatizado
✅ 118 testes passando
```

**Próximo passo (v2)**: Multi-channel (emails), integrações CRM, MCP Server.

---

*Anterior: `mini-plan-04-copilot.md`*
*Fim da sequência de mini-plans.*
