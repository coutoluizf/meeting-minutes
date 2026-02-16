# Mini-Plan 04: Copilot Cloud (Edge Functions + Semantic Search)

> **Status**: Pendente
> **Depende de**: Mini-Plan 03 (Web UI) ✅ completo
> **PRD Referência**: Seções 9, 10.3 (Copilot), 11 (Fase 4) do `PRD_CLOUD_SYNC_V1.md`
> **Estimativa**: ~1-2 sessões
> **Design**: Usar skill `frontend-design` para o chat UI (ref: Cursor AI sidebar)

---

## Contexto

A Web UI está funcional (MP03), com auth, dashboard, meeting detail, workspaces e contacts. O Copilot chat tab no meeting detail é um placeholder. Agora implementamos o backend (Edge Functions) e frontend do Copilot com 4 níveis de contexto e busca semântica via pgvector.

## Pré-requisitos

- [ ] Mini-Plans 01, 02 e 03 completos e todos os testes passando
- [ ] Meetings sincronizadas no Supabase com transcripts e summaries
- [ ] Edge Function `generate-embedding` (MP01) deployed e gerando embeddings
- [ ] Chave `OPENAI_API_KEY` configurada nas Edge Functions (para embeddings)
- [ ] Chave do LLM configurada (Claude API key ou outro provider)

## Escopo

### O que FAZER neste mini-plan:
1. Edge Function `copilot-chat` (endpoint principal do copilot)
2. 4 níveis de contexto (global, workspace, contact, meeting)
3. Busca semântica via `search_meetings_by_embedding()` (pgvector)
4. Streaming de resposta (SSE — Server-Sent Events)
5. Histórico de conversas persistente (tabela `chat_messages`)
6. UI completa do Copilot (chat panel, mensagens, input)
7. Context pills (mostram de onde veio a informação)
8. Copilot full-page (`/copilot`) para chat global

### O que NÃO fazer:
- RAG avançado com re-ranking (v2)
- Voice input/output no copilot (v2)
- Copilot no app Tauri desktop (permanece local, Fase futura)
- Multi-model selection no copilot (fixo em Claude por ora)

---

## Steps de Implementação

### Step 1: Edge Function `copilot-chat`

**1.1 — Estrutura do arquivo:**

```
supabase/functions/copilot-chat/
├── index.ts              # Entry point — roteamento e auth
├── context.ts            # Busca de contexto por nível
├── embedding.ts          # Geração de embedding da query
├── llm.ts                # Chamada ao LLM (Claude)
└── stream.ts             # SSE streaming helper
```

**1.2 — index.ts** (entry point):
```typescript
// POST /functions/v1/copilot-chat
//
// Headers: Authorization: Bearer <jwt>
//
// Body:
// {
//   "message": "O que o João falou sobre preço?",
//   "context_type": "global" | "workspace" | "contact" | "meeting",
//   "context_id": "uuid" | null,
//   "conversation_id": "uuid" | null   // null = nova conversa
// }
//
// Response: SSE stream (text/event-stream)
//   data: {"type": "token", "content": "O João"}
//   data: {"type": "token", "content": " mencionou"}
//   data: {"type": "sources", "meetings": [{id, title, similarity}]}
//   data: {"type": "done", "message_id": "uuid"}
//
// Fluxo:
// 1. Validar JWT (extrair user_id)
// 2. Buscar histórico da conversa (últimas 10 msgs)
// 3. Gerar embedding da pergunta do usuário
// 4. Buscar meetings relevantes via search_meetings_by_embedding()
// 5. Aplicar filtro de contexto (workspace, contact, meeting)
// 6. Montar prompt com transcrições/resumos como contexto
// 7. Chamar LLM com streaming
// 8. Salvar mensagem do usuário + resposta em chat_messages
// 9. Retornar stream SSE
```

**1.3 — context.ts** (busca de contexto por nível):
```typescript
// Os 4 níveis de contexto determinam o escopo da busca:
//
// 1. GLOBAL (context_type="global", context_id=null):
//    - Busca semântica em TODAS as meetings do usuário
//    - Ideal para: "Resuma meu dia", "O que discuti com o João?"
//
// 2. WORKSPACE (context_type="workspace", context_id=workspace_uuid):
//    - Busca semântica filtrada por workspace
//    - Ideal para: "Quais decisões tomamos no projeto Alpha?"
//
// 3. CONTACT (context_type="contact", context_id=contact_uuid):
//    - Busca em meetings que incluem o contact (via meeting_contacts)
//    - Ideal para: "O que o João mencionou sobre preços?"
//
// 4. MEETING (context_type="meeting", context_id=meeting_uuid):
//    - Contexto é a transcrição + resumo desta meeting específica
//    - Não precisa de busca semântica
//    - Ideal para: "Resuma esta reunião", "Quem falou sobre X?"
//
// Função: getRelevantContext(user_id, context_type, context_id, query_embedding)
//   → Retorna: { meetings: Meeting[], transcripts: string[], summaries: string[] }
```

**1.4 — embedding.ts** (geração de embedding da query):
```typescript
// Gera embedding da pergunta do usuário via OpenAI
//
// Input: query string (ex: "O que discutimos sobre preço?")
// Output: VECTOR(1536)
//
// Usa: OpenAI text-embedding-3-small (mesmo modelo dos meetings)
// Env: OPENAI_API_KEY
```

**1.5 — llm.ts** (chamada ao LLM):
```typescript
// Chama Claude API com streaming
//
// System prompt inclui:
// - Instruções de comportamento (responder em pt-BR, ser conciso)
// - Contexto da conversa (histórico)
// - Transcrições e resumos relevantes (do context.ts)
// - Instruções para citar fontes (título da meeting + timestamp)
//
// Model: claude-sonnet-4-5-20250929 (configurable via env)
// Max tokens: 4096
// Temperature: 0.3 (respostas factuais)
//
// Streaming: Lê chunks da API Anthropic e re-emite como SSE
```

**1.6 — stream.ts** (SSE helper):
```typescript
// Helper para construir response SSE
//
// Formatos de evento:
// - "token": chunk de texto da resposta
// - "sources": lista de meetings usadas como contexto
// - "done": sinaliza fim da resposta (inclui message_id)
// - "error": se algo falhou
```

### Step 2: System Prompt do Copilot

```typescript
// Prompt base (em português, conforme padrão do projeto):
const COPILOT_SYSTEM_PROMPT = `Você é o assistente AI do Meetily, especializado em reuniões e interações profissionais.

REGRAS:
1. Responda SEMPRE em português do Brasil.
2. Baseie suas respostas EXCLUSIVAMENTE no contexto fornecido (transcrições e resumos de reuniões).
3. Se a informação não está no contexto, diga claramente: "Não encontrei informação sobre isso nas reuniões disponíveis."
4. Ao citar informações, referencie a reunião de origem: [Título da Reunião — DD/MM/YYYY]
5. Seja conciso e direto. Prefira bullets a parágrafos longos.
6. Se o usuário pedir algo que não é sobre reuniões, responda educadamente que seu foco é em reuniões e interações profissionais.

CONTEXTO DISPONÍVEL:
{context_type}: {context_description}

REUNIÕES RELEVANTES:
{formatted_meetings_context}

HISTÓRICO DA CONVERSA:
{conversation_history}
`;
```

### Step 3: Persistência de Chat (chat_messages)

A tabela `chat_messages` já foi criada no MP01. Neste step, implementamos a lógica de leitura/escrita:

```typescript
// Salvar mensagem do usuário:
// INSERT INTO chat_messages (user_id, context_type, context_id, role, content, origin)
// VALUES (auth.uid(), 'global', null, 'user', 'O que discutimos?', 'web')

// Salvar resposta do assistente:
// INSERT INTO chat_messages (user_id, context_type, context_id, role, content, model_used, origin)
// VALUES (auth.uid(), 'global', null, 'assistant', 'Vocês discutiram...', 'claude-sonnet-4-5', 'web')

// Buscar histórico (últimas N mensagens da conversa):
// SELECT * FROM chat_messages
// WHERE user_id = auth.uid()
//   AND context_type = 'global'
//   AND context_id IS NULL  (ou = specific_id)
// ORDER BY created_at DESC
// LIMIT 20
```

**Nota sobre conversation_id**: Usamos a combinação `(user_id, context_type, context_id)` como identificador implícito da conversa. Cada contexto tem uma thread de conversa. Para implementar múltiplas threads no mesmo contexto, adicionar `conversation_id` à tabela (v2).

### Step 4: Frontend — Copilot Panel Component

**4.1 — CopilotPanel.tsx** (`web/src/components/copilot/CopilotPanel.tsx`):
- Ref: Cursor AI sidebar
- Panel lateral (320px), toggle com ⌘/
- Header: context pill (mostra contexto ativo), botão "Nova conversa"
- Body: lista de mensagens com scroll automático
- Footer: input de texto + botão enviar
- **Usar skill `frontend-design`** com referência Cursor AI

**4.2 — ChatMessage.tsx** (`web/src/components/copilot/ChatMessage.tsx`):
- Mensagens do usuário: alinhadas à direita, fundo accent
- Mensagens do assistente: alinhadas à esquerda, fundo muted
- Suporte a Markdown (renderizar com react-markdown)
- Source citations: badges clicáveis que navegam para a meeting
- Streaming: texto aparece incrementalmente (typewriter effect)
- Timestamp sutil abaixo de cada mensagem

**4.3 — ChatInput.tsx** (`web/src/components/copilot/ChatInput.tsx`):
- Input de texto (textarea auto-resize)
- Enter para enviar, Shift+Enter para nova linha
- Botão de enviar (ícone arrow-up)
- Disabled durante streaming (com indicador "pensando...")
- Suggestion chips acima do input: "Resuma meu dia", "Ações pendentes", "O que discutimos semana passada?"

**4.4 — ContextPill.tsx** (`web/src/components/copilot/ContextPill.tsx`):
- Pill/badge que mostra contexto ativo
- Ícones por tipo: 🌐 Global, 📁 Workspace, 👤 Contact, 📝 Meeting
- Clicável: abre dropdown para mudar contexto
- Quando em meeting detail, auto-seleciona contexto da meeting

**4.5 — SourceCard.tsx** (`web/src/components/copilot/SourceCard.tsx`):
- Card compacto que aparece nas respostas do assistente
- Mostra: título da meeting, data, similarity score (badge sutil)
- Clicável: navega para meeting detail
- Aparece após o bloco "Sources" do SSE

### Step 5: Frontend — Copilot Hook

**5.1 — useCopilot.ts** (`web/src/lib/hooks/useCopilot.ts`):
```typescript
// Hook principal para interagir com o copilot
//
// Interface:
// const {
//   messages,           // ChatMessage[]
//   isStreaming,        // boolean
//   sources,            // MeetingSource[]
//   sendMessage,        // (text: string) => void
//   clearConversation,  // () => void
//   contextType,        // 'global' | 'workspace' | 'contact' | 'meeting'
//   setContext,          // (type, id?) => void
// } = useCopilot({ defaultContext: 'global' })
//
// Implementação:
// 1. Fetch histórico ao montar (GET chat_messages via Supabase client)
// 2. sendMessage: POST /functions/v1/copilot-chat (SSE)
// 3. Parsear SSE stream:
//    - "token" → append ao último message
//    - "sources" → setSource
//    - "done" → mark streaming complete
//    - "error" → show error toast
// 4. Otimistic UI: mensagem do usuário aparece imediatamente
```

### Step 6: Integração nas Páginas Existentes

**6.1 — Meeting Detail** (`(app)/meetings/[id]/page.tsx`):
- Tab "Chat" agora usa `CopilotPanel` real (substituir placeholder do MP03)
- Context auto-setado para `context_type="meeting"`, `context_id=meeting.id`
- Transcrição e resumo são passados como contexto direto (sem busca semântica)

**6.2 — App Shell** (`(app)/layout.tsx`):
- Copilot panel lateral (toggle com ⌘/)
- Context default: "global"
- Muda contexto ao navegar: workspace page → workspace context, contact page → contact context
- Panel persiste entre navegações (não reseta)

**6.3 — Copilot Full-Page** (`(app)/copilot/page.tsx`):
- Chat full-width (sem sidebar de meetings)
- Contexto global por default
- Dropdown para selecionar contexto (workspace, contact)
- Histórico completo com scroll infinito
- **Usar skill `frontend-design`** para layout de chat full-page

**6.4 — Dashboard** (`(app)/page.tsx`):
- Copilot panel lateral já presente no app shell
- Quick suggestions no dashboard: "Resuma meu dia", "Ações pendentes"

### Step 7: Deploy da Edge Function

```bash
# Deploy copilot-chat
cd supabase
supabase functions deploy copilot-chat

# Configurar secrets (se não feito antes)
supabase secrets set OPENAI_API_KEY=sk-...
supabase secrets set ANTHROPIC_API_KEY=sk-ant-...
supabase secrets set COPILOT_MODEL=claude-sonnet-4-5-20250929
```

### Step 8: CORS e Rate Limiting

```typescript
// copilot-chat/index.ts — CORS headers
//
// Permitir chamadas do domínio web:
// Access-Control-Allow-Origin: https://app.meetily.ai (ou * em dev)
// Access-Control-Allow-Headers: authorization, content-type
// Access-Control-Allow-Methods: POST, OPTIONS
//
// Rate limiting (básico):
// - Max 20 mensagens por minuto por usuário
// - Implementar via contagem em memória (ou Redis no futuro)
// - Retornar 429 Too Many Requests se exceder
```

---

## Testes E2E — Mini-Plan 04

### Playwright Tests (Web)

Criar em `web/e2e/copilot.spec.ts`:

```typescript
// copilot.spec.ts
test('copilot panel opens with ⌘/')
test('copilot panel closes with ⌘/ or Esc')
test('send message and receive streaming response')
test('response contains source citations')
test('source citation links navigate to meeting detail')
test('conversation history persists between page loads')
test('context pill shows correct context type')
test('meeting detail chat uses meeting context')
test('global context searches across all meetings')
test('workspace context filters by workspace')
test('suggestion chips send pre-defined messages')
test('new conversation clears history')
test('streaming indicator shows during response')
test('error state shows when edge function fails')
test('copilot full-page (/copilot) renders correctly')
test('markdown in responses renders properly')
```

### Edge Function Tests

Criar em `supabase/tests/copilot.test.ts`:

```typescript
// copilot.test.ts (Deno test)
test('POST with valid JWT returns SSE stream')
test('POST without JWT returns 401')
test('context_type=meeting uses correct meeting context')
test('context_type=global performs semantic search')
test('context_type=workspace filters results')
test('context_type=contact filters by meeting_contacts')
test('empty query returns error')
test('conversation history is saved to chat_messages')
test('rate limiting returns 429 after threshold')
test('SSE stream includes sources event')
test('SSE stream includes done event with message_id')
```

### Comando de Testes Cumulativos

```bash
# Mini-plan 04 — roda testes deste mini-plan + todos os anteriores

echo "=== Mini-Plan 01: Supabase Foundation ==="
cd supabase && ./run-tests.sh

echo "=== Mini-Plan 02: Sync Engine ==="
cd frontend/src-tauri && cargo test --test '*cloud*' -- --test-threads=1

echo "=== Mini-Plan 03: Web UI ==="
cd web && pnpm exec playwright test --grep-invert '@copilot'

echo "=== Mini-Plan 04: Copilot ==="
cd web && pnpm exec playwright test e2e/copilot.spec.ts
cd supabase && deno test tests/copilot.test.ts

echo "=== RESULTADO CUMULATIVO ==="
# Esperado: 32 (MP01) + 14 (MP02) + 26 (MP03) + 27 (MP04) = 99 testes passando
```

---

## Checklist de Validação Final

Antes de avançar para o Mini-Plan 05, confirmar:

- [ ] Edge Function `copilot-chat` deployed e respondendo
- [ ] Chat no meeting detail funciona (contexto meeting)
- [ ] Chat global funciona (busca semântica cross-workspace)
- [ ] Chat por workspace funciona (filtro correto)
- [ ] Chat por contact funciona (filtro por meeting_contacts)
- [ ] Streaming (SSE) funciona — resposta aparece incrementalmente
- [ ] Source citations aparecem e são clicáveis
- [ ] Histórico de conversa persiste (chat_messages no Supabase)
- [ ] Context pill mostra contexto correto e permite trocar
- [ ] Copilot full-page (/copilot) renderiza corretamente
- [ ] Suggestion chips funcionam
- [ ] Rate limiting retorna 429 quando excedido
- [ ] Design segue referência Cursor AI sidebar
- [ ] Todos os 99 testes cumulativos passando (32 + 14 + 26 + 27)

---

*Anterior: `mini-plan-03-web-ui.md`*
*Próximo: `mini-plan-05-e2e-tests-deploy.md`*
