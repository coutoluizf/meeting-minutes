# Mini-Plan 01: Supabase Foundation

> **Status**: Pendente
> **Depende de**: Nenhum (primeiro mini-plan)
> **PRD Referência**: Seções 5, 6, 7, 9 do `PRD_CLOUD_SYNC_V1.md`
> **Estimativa**: ~1 sessão

---

## Contexto

O Meetily é um app desktop (Tauri) de gravação e transcrição de reuniões, 100% local. Estamos adicionando cloud sync via Supabase para permitir acesso de múltiplos dispositivos e via web. Este mini-plan configura toda a infraestrutura Supabase.

## Pré-requisitos

- [ ] Ter uma conta no Supabase (https://supabase.com)
- [ ] Criar um projeto Supabase manualmente no Dashboard (o agente não faz isso)
- [ ] Ter as credenciais disponíveis:
  - `SUPABASE_URL` (ex: https://xxxxx.supabase.co)
  - `SUPABASE_ANON_KEY`
  - `SUPABASE_SERVICE_ROLE_KEY`
- [ ] Configurar as variáveis de ambiente no arquivo `.env` na raiz do projeto

## Escopo

### O que FAZER neste mini-plan:
1. Criar estrutura de diretório `supabase/` no projeto
2. Criar SQL migrations com schema completo (tabelas, indexes, RLS, functions, triggers)
3. Criar Edge Function `generate-embedding`
4. Configurar Supabase Storage bucket
5. Criar script de setup automatizado
6. Criar testes de validação do schema e RLS
7. Documentar configuração de Auth providers (manual no Dashboard)

### O que NÃO fazer:
- Código Rust (Fase 2)
- Código Next.js (Fase 3)
- Modificar o app Tauri existente

---

## Steps de Implementação

### Step 1: Estrutura de diretório

```
supabase/
├── migrations/
│   ├── 001_extensions.sql           # pgvector, pgvectorscale, pg_trgm
│   ├── 002_profiles.sql             # Tabela profiles (extensão auth.users)
│   ├── 003_devices.sql              # Tabela devices
│   ├── 004_workspaces.sql           # Tabela workspaces
│   ├── 005_contacts.sql             # Tabela contacts + workspace_contacts
│   ├── 006_meetings.sql             # Tabela meetings (com embedding vector)
│   ├── 007_transcripts.sql          # Tabela transcripts
│   ├── 008_summaries.sql            # Tabela summaries
│   ├── 009_chat_messages.sql        # Tabela chat_messages
│   ├── 010_meeting_contacts.sql     # Tabela meeting_contacts (N:N)
│   ├── 011_indexes.sql              # Todos os indexes (DiskANN, trigram, etc.)
│   ├── 012_rls_policies.sql         # Row Level Security para todas as tabelas
│   ├── 013_functions.sql            # Functions (updated_at trigger, search)
│   └── 014_storage.sql              # Storage bucket + policies
│
├── functions/
│   └── generate-embedding/
│       └── index.ts                 # Edge Function: gera embedding via OpenAI
│
├── tests/
│   ├── rls.test.sql                 # Testa que RLS bloqueia cross-user
│   ├── schema.test.sql              # Valida schema, constraints, indexes
│   └── functions.test.ts            # Testa Edge Function isoladamente
│
├── seed.sql                         # Dados de teste (opcional)
├── config.toml                      # Supabase CLI config (se usar CLI local)
└── .env.example                     # Template das variáveis de ambiente
```

### Step 2: SQL Migrations

Implementar cada migration conforme o schema da seção 7.1 do PRD (`PRD_CLOUD_SYNC_V1.md`).

**Detalhes críticos a não esquecer:**
- `CREATE EXTENSION IF NOT EXISTS vector;` (pgvector)
- `VECTOR(1536)` para embeddings (OpenAI text-embedding-3-small)
- `TIMESTAMPTZ` (não TEXT) para todas as datas
- `ON DELETE CASCADE` em todas as foreign keys
- `gen_random_uuid()` como default para IDs
- `RLS ENABLE` em TODAS as tabelas
- Triggers de `updated_at` em meetings, workspaces, contacts, summaries, profiles

### Step 3: RLS Policies

Para CADA tabela, criar policy:
```sql
CREATE POLICY "Users can CRUD own {table}"
    ON public.{table} FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);
```

Para tabelas de junção (workspace_contacts, meeting_contacts), verificar via parent:
```sql
CREATE POLICY "Users can manage {junction}"
    ON public.{junction} FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM public.{parent}
            WHERE id = {junction}.{parent}_id
            AND user_id = auth.uid()
        )
    );
```

### Step 4: Edge Function — generate-embedding

```typescript
// supabase/functions/generate-embedding/index.ts
//
// Trigger: Chamada via HTTP POST (invocada após meeting sync)
// Input: { meeting_id: string }
// Fluxo:
//   1. Busca transcripts da meeting (Supabase client com service_role)
//   2. Concatena texto dos transcripts
//   3. Chama OpenAI embeddings API (text-embedding-3-small)
//   4. UPDATE meetings SET embedding = <vector> WHERE id = meeting_id
//   5. Retorna success/error
//
// Variáveis de ambiente (configurar no Dashboard):
//   - OPENAI_API_KEY
//   - SUPABASE_URL (auto-disponível)
//   - SUPABASE_SERVICE_ROLE_KEY (auto-disponível)
```

### Step 5: Storage Bucket

```sql
-- Criar bucket privado para áudio
INSERT INTO storage.buckets (id, name, public)
VALUES ('meetily-audio', 'meetily-audio', false);

-- Policy: usuário só acessa seus próprios arquivos
-- Path format: {user_id}/{meeting_id}/recording.wav
CREATE POLICY "Users can manage own audio"
ON storage.objects FOR ALL
USING (
    bucket_id = 'meetily-audio'
    AND auth.uid()::text = (storage.foldername(name))[1]
)
WITH CHECK (
    bucket_id = 'meetily-audio'
    AND auth.uid()::text = (storage.foldername(name))[1]
);
```

### Step 6: Script de Setup

Criar `supabase/setup.sh`:
```bash
#!/bin/bash
# Aplica todas as migrations em ordem no Supabase remoto
# Uso: ./supabase/setup.sh
# Requer: SUPABASE_URL e SUPABASE_SERVICE_ROLE_KEY no .env
```

### Step 7: Arquivo .env.example

```
# Supabase
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=eyJhbGciOi...
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOi...

# OpenAI (para embeddings)
OPENAI_API_KEY=sk-...

# Auth redirect URLs (configurar no Supabase Dashboard)
# Web: https://app.meetily.ai/auth/callback
# Desktop: meetily://auth/callback
```

---

## Configuração Manual (instruções para o usuário)

Após executar este mini-plan, o usuário precisa configurar manualmente no Supabase Dashboard:

### Auth Providers
1. **Email Magic Link**: Authentication → Providers → Email → Enable (já vem habilitado)
2. **Google OAuth**:
   - Google Cloud Console → Criar OAuth 2.0 Client ID
   - Authorized redirect: `https://<project>.supabase.co/auth/v1/callback`
   - Supabase Dashboard → Authentication → Providers → Google → Preencher Client ID + Secret
3. **GitHub OAuth**:
   - GitHub Settings → Developer Settings → OAuth Apps → New
   - Callback URL: `https://<project>.supabase.co/auth/v1/callback`
   - Supabase Dashboard → Authentication → Providers → GitHub → Preencher Client ID + Secret

### Redirect URLs
- Authentication → URL Configuration → Redirect URLs:
  - `https://app.meetily.ai/auth/callback` (Web — ajustar domínio)
  - `http://localhost:3000/auth/callback` (Web dev)
  - `meetily://auth/callback` (Desktop deep link)

### Edge Function Secrets
- Edge Functions → generate-embedding → Secrets:
  - `OPENAI_API_KEY`: chave da OpenAI para embeddings

---

## Testes E2E — Mini-Plan 01

### Como rodar

```bash
# Rodar todos os testes deste mini-plan
cd supabase && ./run-tests.sh
```

### Test Suite: Schema Validation

```sql
-- schema.test.sql
-- Verifica que todas as tabelas existem
-- Verifica que todas as colunas existem com tipos corretos
-- Verifica que indexes foram criados
-- Verifica que triggers estão ativos
-- Verifica que extensions (pgvector, pg_trgm) estão habilitadas
```

**Casos de teste:**
- [ ] Tabela `profiles` existe com colunas corretas
- [ ] Tabela `devices` existe com FK para auth.users
- [ ] Tabela `workspaces` existe com colunas type_hint, is_default
- [ ] Tabela `contacts` existe com colunas corretas
- [ ] Tabela `workspace_contacts` existe com PK composta
- [ ] Tabela `meetings` existe com coluna `embedding VECTOR(1536)`
- [ ] Tabela `transcripts` existe com FK cascade para meetings
- [ ] Tabela `summaries` existe com FK cascade para meetings
- [ ] Tabela `chat_messages` existe com context_type e context_id
- [ ] Tabela `meeting_contacts` existe com PK composta
- [ ] Index DiskANN em meetings.embedding existe
- [ ] Index trigram em transcripts.text existe
- [ ] Trigger updated_at funciona (UPDATE meetings → updated_at muda)
- [ ] Function search_meetings_by_embedding existe e retorna resultados

### Test Suite: RLS Security

```sql
-- rls.test.sql
-- Cria 2 test users (User A, User B)
-- User A insere dados
-- Tenta acessar com User B → deve falhar
```

**Casos de teste:**
- [ ] User A cria meeting → User B NÃO consegue SELECT
- [ ] User A cria meeting → User B NÃO consegue UPDATE
- [ ] User A cria meeting → User B NÃO consegue DELETE
- [ ] User A cria transcript → User B NÃO consegue acessar
- [ ] User A cria summary → User B NÃO consegue acessar
- [ ] User A cria chat_message → User B NÃO consegue acessar
- [ ] User A cria workspace → User B NÃO consegue acessar
- [ ] User A cria contact → User B NÃO consegue acessar
- [ ] User A cria device → User B NÃO consegue acessar
- [ ] Service role key BYPASSA RLS (para Edge Functions)
- [ ] Anon key sem user NÃO acessa nenhum dado

### Test Suite: Storage Policies

- [ ] User A faz upload para `{userA_id}/meeting1/recording.wav` → sucesso
- [ ] User B tenta acessar `{userA_id}/meeting1/recording.wav` → bloqueado
- [ ] User A acessa seu próprio arquivo → sucesso
- [ ] Signed URL funciona e expira

### Test Suite: Edge Function

- [ ] POST generate-embedding com meeting_id válido → gera embedding
- [ ] Meeting sem transcripts → retorna erro gracioso
- [ ] Meeting_id inválido → retorna 404

---

## Comando de Testes Cumulativos

```bash
# Mini-plan 01 — roda apenas os testes deste mini-plan
cd supabase && ./run-tests.sh

# Saída esperada:
# ✓ Schema Validation: 14/14 passed
# ✓ RLS Security: 11/11 passed
# ✓ Storage Policies: 4/4 passed
# ✓ Edge Function: 3/3 passed
# ✓ TOTAL: 32/32 passed
```

---

## Checklist de Validação Final

Antes de avançar para o Mini-Plan 02, confirmar:

- [ ] Todas as 10 tabelas criadas no Supabase
- [ ] RLS habilitado e policies criadas para todas as tabelas
- [ ] Storage bucket `meetily-audio` criado e privado
- [ ] Edge Function `generate-embedding` deployed e respondendo
- [ ] Auth providers configurados (pelo menos Email Magic Link)
- [ ] Redirect URLs configurados (web + desktop deep link)
- [ ] Todos os 32 testes passando
- [ ] `.env` com credenciais corretas na raiz do projeto

---

*Próximo: `mini-plan-02-sync-engine-tauri.md`*
