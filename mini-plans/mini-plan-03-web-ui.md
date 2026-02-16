# Mini-Plan 03: Web UI (Next.js 16)

> **Status**: Pendente
> **Depende de**: Mini-Plan 02 (Sync Engine) ✅ completo
> **PRD Referência**: Seções 5.5, 10 do `PRD_CLOUD_SYNC_V1.md`
> **Estimativa**: ~2-3 sessões (usar Team Agents: Auth/Layout + Pages + Copilot em paralelo)
> **Design**: Usar skill `frontend-design` para CADA componente visual (ver seção 10.2 do PRD)

---

## Contexto

Supabase está configurado (MP01), o app desktop sincroniza meetings para a nuvem (MP02). Agora precisamos de uma Web UI para acessar meetings de qualquer lugar, com auth obrigatória e design de nível produto (referências: Apple, Linear, Vercel, Granola).

## Pré-requisitos

- [ ] Mini-Plan 01 e 02 completos e todos os testes passando
- [ ] Meetings sincronizadas no Supabase (pelo menos dados de teste)
- [ ] Auth providers configurados no Supabase Dashboard
- [ ] Node.js 20+ e pnpm instalados

## Escopo

### O que FAZER neste mini-plan:
1. Scaffold projeto Next.js 16 (Turbopack, Tailwind 4, Shadcn/ui)
2. Auth completa (login, callback, middleware, logout)
3. App shell (sidebar Linear-style, topbar, command palette)
4. Dashboard (meetings recentes, workspaces)
5. Meeting detail (transcript viewer, summary viewer, audio player)
6. Workspace CRUD
7. Contact management básico
8. Settings page
9. Testes E2E Playwright

### O que NÃO fazer:
- Copilot avançado com 4 níveis de contexto (Fase 4) — incluir chat panel básico como placeholder
- Gravação via web (permanece exclusiva do desktop)

---

## Steps de Implementação

### Step 1: Scaffold do Projeto

```bash
# Criar projeto Next.js 16 na raiz do monorepo
cd /Users/luiz/git/meeting-minutes
pnpm create next-app web --typescript --tailwind --eslint --app --turbopack
cd web
pnpm add @supabase/supabase-js @supabase/ssr
pnpm add framer-motion lucide-react sonner
pnpm add -D @playwright/test
npx shadcn@latest init  # Configurar com Geist font, dark mode, slate palette
```

**Estrutura resultante:**
```
web/
├── src/
│   ├── app/
│   │   ├── layout.tsx               # Root layout
│   │   ├── (marketing)/
│   │   │   └── page.tsx             # Landing page (pública)
│   │   ├── (auth)/
│   │   │   ├── login/page.tsx       # Login
│   │   │   ├── callback/route.ts    # OAuth callback
│   │   │   └── logout/route.ts      # Logout
│   │   └── (app)/
│   │       ├── layout.tsx           # App shell (sidebar + topbar)
│   │       ├── page.tsx             # Dashboard
│   │       ├── workspaces/
│   │       │   ├── page.tsx
│   │       │   └── [id]/page.tsx
│   │       ├── meetings/
│   │       │   └── [id]/page.tsx
│   │       ├── contacts/
│   │       │   ├── page.tsx
│   │       │   └── [id]/page.tsx
│   │       └── settings/page.tsx
│   ├── components/
│   │   ├── ui/                      # Shadcn components
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── TopBar.tsx
│   │   │   └── CommandPalette.tsx
│   │   ├── meetings/
│   │   │   ├── MeetingCard.tsx
│   │   │   ├── TranscriptViewer.tsx
│   │   │   ├── SummaryViewer.tsx
│   │   │   └── AudioPlayer.tsx
│   │   ├── copilot/
│   │   │   ├── CopilotPanel.tsx
│   │   │   ├── ChatMessage.tsx
│   │   │   └── ChatInput.tsx
│   │   └── workspaces/
│   │       ├── WorkspaceCard.tsx
│   │       └── WorkspaceSelector.tsx
│   ├── lib/
│   │   ├── supabase/
│   │   │   ├── client.ts            # Browser client
│   │   │   ├── server.ts            # Server client (RSC)
│   │   │   └── middleware.ts        # Auth middleware helper
│   │   ├── hooks/
│   │   │   ├── useMeetings.ts
│   │   │   ├── useWorkspaces.ts
│   │   │   ├── useContacts.ts
│   │   │   └── useAuth.ts
│   │   └── utils.ts
│   └── styles/
│       └── globals.css
├── e2e/                              # Playwright E2E tests
│   ├── setup/
│   │   ├── global-setup.ts
│   │   ├── global-teardown.ts
│   │   ├── auth.setup.ts
│   │   └── fixtures.ts
│   ├── auth.spec.ts
│   ├── dashboard.spec.ts
│   ├── meeting-detail.spec.ts
│   ├── workspace.spec.ts
│   ├── contact.spec.ts
│   └── search.spec.ts
├── middleware.ts                      # Next.js middleware (auth guard)
├── playwright.config.ts
├── next.config.ts
├── tailwind.config.ts
└── package.json
```

### Step 2: Supabase Client Setup

**2.1 — lib/supabase/client.ts** (browser, client components):
```typescript
import { createBrowserClient } from '@supabase/ssr'
export const createClient = () =>
  createBrowserClient(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!
  )
```

**2.2 — lib/supabase/server.ts** (server, RSC):
```typescript
import { createServerClient } from '@supabase/ssr'
import { cookies } from 'next/headers'
// Cria client com cookies do request
```

**2.3 — middleware.ts** (proteção de rotas):
```typescript
// Para TODA request:
// 1. Refresh token se expirado
// 2. Se rota (app)/* e sem sessão → redirect /login
// 3. Se rota /login e com sessão → redirect /
```

### Step 3: Auth Pages

**3.1 — Login page** (`(auth)/login/page.tsx`):
- Design: ref Vercel login (centered card, fundo escuro, minimal)
- Botões: "Continuar com Google", "Continuar com GitHub"
- Input email + botão "Enviar magic link"
- Link "Baixe o app desktop" no footer
- **Usar skill `frontend-design`** com referência Vercel

**3.2 — OAuth callback** (`(auth)/callback/route.ts`):
```typescript
// GET handler
// 1. Extrai code da URL
// 2. Troca por sessão via supabase.auth.exchangeCodeForSession(code)
// 3. Redirect para /
```

**3.3 — Logout** (`(auth)/logout/route.ts`):
```typescript
// POST handler
// 1. supabase.auth.signOut()
// 2. Redirect para /login
```

### Step 4: App Shell (Layout)

**4.1 — Sidebar** (`components/layout/Sidebar.tsx`):
- Ref: Linear sidebar
- Colapsável (240px → 64px), toggle com ⌘B
- Seções: Inbox (default workspace), Workspaces (expandível), Contacts
- Active state com accent color do workspace
- Hover states elegantes
- Footer: settings link, user avatar small
- **Usar skill `frontend-design`** com referência Linear

**4.2 — TopBar** (`components/layout/TopBar.tsx`):
- Breadcrumb (Workspace > Meeting title)
- Search trigger (⌘K)
- Sync status badge (se veio de desktop)
- User avatar + menu

**4.3 — Command Palette** (`components/layout/CommandPalette.tsx`):
- Ref: Linear ⌘K
- Fuzzy search em meetings, workspaces, contacts
- Ações contextuais (criar workspace, ir para settings)
- Keyboard navigation (↑↓ Enter Esc)
- **Usar skill `frontend-design`** com referência Linear

### Step 5: Dashboard (`(app)/page.tsx`)

- Ref: Vercel dashboard
- Meeting cards recentes (título, data, duração, device badge, workspace tag)
- Filtro por workspace (dropdown ou tabs)
- Busca rápida (input no topo)
- Empty state se não há meetings ("Sincronize meetings do app desktop")
- **Usar skill `frontend-design`** com referência Vercel

### Step 6: Meeting Detail (`(app)/meetings/[id]/page.tsx`)

- Ref: Granola (split view, AI content em cinza)
- Tabs: Transcript / Summary / Chat
- **Transcript tab**: segmentos com timestamps clicáveis, scroll sync
- **Summary tab**: Markdown renderizado, AI content em cor diferenciada, editável
- **Chat tab**: Placeholder básico (copilot completo no MP04)
- Sidebar direita: participantes, metadata (duração, device, modelo usado)
- Audio player minimal (waveform, play/pause, speed) — se áudio sincronizado
- **Usar skill `frontend-design`** com referência Granola

### Step 7: Workspace Pages

**7.1 — Workspace list** (`(app)/workspaces/page.tsx`):
- Grid de workspace cards (nome, cor, ícone, meeting count)
- Botão "Novo workspace"
- Modal de criação (nome, tipo, cor, ícone)

**7.2 — Workspace detail** (`(app)/workspaces/[id]/page.tsx`):
- Lista de meetings do workspace
- Filtros (data, duração, com/sem resumo)
- Drag & drop para mover meetings entre workspaces (opcional)
- **Usar skill `frontend-design`** com referência Linear

### Step 8: Contact Pages

**8.1 — Contact list** (`(app)/contacts/page.tsx`):
- Lista com avatar, nome, empresa, email
- Busca por nome/empresa
- Botão "Novo contact"

**8.2 — Contact detail** (`(app)/contacts/[id]/page.tsx`):
- Info card (nome, email, empresa, telefone)
- Timeline de interactions (meetings em que participou)
- Cross-workspace (mostra meetings de todos os workspaces)

### Step 9: Settings (`(app)/settings/page.tsx`)

- Tabs: Account, Devices, Sync, Appearance
- **Account**: email, display name, avatar, logout, delete account
- **Devices**: lista de devices registrados, último sync de cada
- **Sync**: toggle sync áudio, re-import meetings
- **Appearance**: dark/light mode, accent color

### Step 10: Design Polish

- Dark/light mode com transição suave (CSS transition, sem flash)
- Responsive: sidebar collapses em mobile, layout empilha
- Loading states com skeletons (não spinners)
- Empty states com ilustrações sutis e CTAs
- Keyboard shortcuts (⌘K, ⌘B, ⌘/, J/K, Enter, Esc)
- Toast notifications (sonner) para ações

---

## Referências de Design (resumo para o skill `frontend-design`)

Ao invocar o skill `frontend-design` para cada componente, fornecer:

```
Referências de design:
- Apple: Espaçamento generoso, tipografia perfeita, micro-interações polidas
- Linear: Sidebar navigation, dark mode, ⌘K command palette, issue-like cards
- Vercel: Geist font, dashboard cards, deploy/sync status indicators
- Granola: Split notepad, AI content em cinza, timestamps clicáveis

Cores: Background #09090b (dark) / #fafafa (light), accent configurável
Tipografia: Geist Sans (body), Geist Mono (code/timestamps)
Bordas: border-border/50, separação por espaçamento > bordas
Sombras: mínimas, só modais e dropdowns
```

---

## Testes E2E — Mini-Plan 03

### Playwright Tests

Criar em `web/e2e/`:

```typescript
// auth.spec.ts
test('redirects to login if not authenticated')
test('login with magic link sends email')
test('OAuth callback sets session correctly')
test('logout clears session and redirects')
test('expired token gets refreshed automatically')
test('protected routes require auth')

// dashboard.spec.ts
test('dashboard loads within 2 seconds')
test('meetings appear sorted by date')
test('meeting card shows title, date, duration, device badge')
test('clicking meeting card navigates to detail')
test('workspace filter works')
test('empty state shows when no meetings')

// meeting-detail.spec.ts
test('transcript tab renders segments with timestamps')
test('summary tab renders markdown correctly')
test('AI content is visually differentiated')
test('tabs switch correctly')
test('audio player appears when audio is synced')
test('metadata sidebar shows correct info')

// workspace.spec.ts
test('create workspace with name and color')
test('move meeting to workspace')
test('filter meetings by workspace')
test('delete workspace moves meetings to default')

// contact.spec.ts
test('create contact with name and email')
test('associate contact with meeting')
test('contact timeline shows interactions')
test('cross-workspace interactions appear')

// search.spec.ts
test('⌘K opens command palette')
test('typing filters meetings by title')
test('keyboard navigation works (↑↓ Enter Esc)')
test('selecting result navigates to page')
```

### Comando de Testes Cumulativos

```bash
# Mini-plan 03 — roda testes deste mini-plan + todos os anteriores

echo "=== Mini-Plan 01: Supabase Foundation ==="
cd supabase && ./run-tests.sh

echo "=== Mini-Plan 02: Sync Engine ==="
cd frontend/src-tauri && cargo test --test '*cloud*' -- --test-threads=1

echo "=== Mini-Plan 03: Web UI ==="
cd web && pnpm exec playwright test

echo "=== RESULTADO CUMULATIVO ==="
# Esperado: 32 (MP01) + 14 (MP02) + 26 (MP03) = 72 testes passando
```

---

## Checklist de Validação Final

Antes de avançar para o Mini-Plan 04, confirmar:

- [ ] Next.js 16 project compila e roda (`pnpm dev`)
- [ ] Login via Google funciona na web
- [ ] Login via magic link funciona na web
- [ ] Rotas protegidas redirecionam para /login
- [ ] Dashboard mostra meetings sincronizadas do desktop
- [ ] Meeting detail renderiza transcript e summary
- [ ] Workspace CRUD funciona
- [ ] Contact CRUD funciona
- [ ] Command palette (⌘K) busca e navega
- [ ] Dark mode / light mode toggle funciona
- [ ] Design segue referências (Apple/Linear/Vercel/Granola)
- [ ] Responsive: funciona em mobile
- [ ] Todos os 72 testes cumulativos passando (32 + 14 + 26)

---

*Anterior: `mini-plan-02-sync-engine-tauri.md`*
*Próximo: `mini-plan-04-copilot.md`*
