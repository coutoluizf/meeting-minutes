# Prompt de Execução — Mini-Plans Cloud Sync v1

> **Como usar**: Copie o prompt abaixo e cole numa sessão nova do Claude Code.
> Ajuste o número do mini-plan conforme necessário (01, 02, 03, 04, 05).

---

## Prompt para Mini-Plan 01

```
Você está no projeto Meetily (meeting-minutes). Vamos implementar o Cloud Sync v1.

## Contexto do projeto

Leia estes arquivos ANTES de começar qualquer implementação (nesta ordem):

1. `CLAUDE.md` — Instruções globais do projeto, convenções, arquitetura existente
2. `mini-plans/OVERVIEW.md` — Visão geral do Cloud Sync v1 (arquitetura, stack, schema, design)
3. `PRD_CLOUD_SYNC_V1.md` — PRD completo (referência detalhada quando precisar)

## Branch de trabalho

Toda a implementação do Cloud Sync v1 deve acontecer numa branch separada:

1. Certifique-se de estar na branch `main` e atualizado (`git pull`)
2. Crie a branch: `git checkout -b feature/cloud-sync-v1`
3. Todos os commits desta sessão (e das futuras) vão nessa branch
4. NÃO faça push automático — eu decido quando fazer push

## Tarefa

Execute o **mini-plan-01** (Supabase Foundation):

1. Leia `mini-plans/mini-plan-01-supabase-foundation.md` por completo
2. Execute cada step na ordem descrita no arquivo
3. Siga as convenções do `CLAUDE.md` (especialmente: sempre adicionar comentários no código)
4. Ao final, rode os testes descritos no mini-plan
5. Se todos os testes passarem, faça um commit com as mudanças
6. Se algum teste falhar, corrija e re-rode até todos passarem

## Regras importantes

- Leia TODOS os arquivos de contexto antes de escrever qualquer código
- Siga EXATAMENTE a estrutura de diretórios e nomes de arquivos do mini-plan
- Não pule steps — execute sequencialmente
- Não modifique o app Tauri existente (isso é mini-plan 02)
- Não crie a Web UI (isso é mini-plan 03)
- Se precisar de algo que requer configuração manual (ex: Supabase Dashboard), documente o que precisa ser feito e me pergunte
- Commits devem seguir conventional commits (ex: `feat:`, `fix:`, `docs:`)
```

---

## Prompt para Mini-Plans subsequentes (02, 03, 04, 05)

Substitua o número do mini-plan e ajuste:

```
Você está no projeto Meetily (meeting-minutes), branch `feature/cloud-sync-v1`.

## Contexto do projeto

Leia estes arquivos ANTES de começar qualquer implementação (nesta ordem):

1. `CLAUDE.md` — Instruções globais do projeto
2. `mini-plans/OVERVIEW.md` — Visão geral do Cloud Sync v1
3. `PRD_CLOUD_SYNC_V1.md` — PRD completo (referência quando necessário)

## Branch

Confirme que está na branch `feature/cloud-sync-v1`. Se não estiver, faça checkout.

## Tarefa

Execute o **mini-plan-0X**:

1. Leia `mini-plans/mini-plan-0X-<nome>.md` por completo
2. Execute cada step na ordem descrita
3. Ao final, rode os testes CUMULATIVOS (deste mini-plan + todos os anteriores)
4. Se todos passarem, faça commit
5. Se algum falhar, corrija e re-rode

## Regras

- Leia TODOS os arquivos de contexto antes de escrever código
- Siga a estrutura de diretórios do mini-plan
- Execute steps sequencialmente
- Testes são CUMULATIVOS — todos os mini-plans anteriores devem continuar passando
- NÃO faça push automático
```
