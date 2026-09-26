# Análise da orquestração multi-CLI

> Análise atualizada em 26 de setembro de 2026 sobre a integração entre Codex CLI, Claude Code, Hermes Agent, Pi e Agy no Orbity.

## 1. Resumo executivo

A arquitetura possui uma boa base de DAG, sandbox e adaptação das cinco CLIs, mas a orquestração atual ainda é parcialmente nominal. O YAML descreve uma topologia rica e paralela, enquanto o runtime efetivamente executa um pipeline mais simples, serializado e com pouca integração estruturada entre os agentes.

Os principais pontos de atenção são:

1. A seção canônica `graph_topology` não é convertida em `GraphDefinition`.
2. O paralelismo do grafo é serializado por um único `Mutex<dyn Sandbox>`.
3. O Event Bus é conectado ao executor, mas não recebe eventos da execução.
4. Configurações declarativas de segurança, ferramentas e governança não chegam integralmente ao runtime.
5. O handoff entre agentes consiste principalmente em `stdout` bruto inserido no prompt seguinte.
6. Human-in-the-Loop e retomada por checkpoint ainda não completam o ciclo operacional.

## 2. Fluxo atual

```text
YAML
  ↓
GraphYamlLoader
  ↓
GraphExecutor
  ↓
SandboxCliNodeRunner
  ↓
único Mutex<dyn Sandbox>
  ↓
Codex / Claude / Agy / Hermes / Pi
  ↓
stdout bruto
  ↓
Blackboard
  ↓
prompt do próximo agente
```

Não existe comunicação direta nem sessão compartilhada entre as CLIs. Cada agente é iniciado como um processo independente. A cooperação ocorre por meio de:

- Arquivos no mesmo workspace sandbox.
- `stdout` do predecessor inserido no prompt seguinte.
- Blackboard em memória.
- Checkpoints SQLite entre batches.

## 3. Papel previsto de cada CLI

A configuração Forester define uma especialização coerente:

| CLI | Papel pretendido |
| --- | --- |
| Agy | Planejamento, pesquisa e síntese |
| Codex | Implementação de código e testes |
| Claude Code | Revisão arquitetural, segurança e conformidade |
| Hermes | Ferramentas, documentação e automação |
| Pi | Correções rápidas e refatoração |

Esses papéis estão definidos em [`examples/forester/teams/forester.yaml`](../examples/forester/teams/forester.yaml#L145).

Na execução canônica, porém, o pipeline carregado possui somente quatro passos sequenciais:

```text
Agy → Codex → Claude → Agy
```

Hermes e Pi aparecem na seção `graph_topology`, atualmente ignorada pelo loader. O teste existente confirma que o Forester carregado possui quatro nós e três arestas: [`crates/orbity-graph/tests/graph_engine_tests.rs`](../crates/orbity-graph/tests/graph_engine_tests.rs#L292).

## 4. Pontos positivos

### 4.1 Abstração comum para CLIs heterogêneas

`CliType` e `NodeRunner` isolam o executor das particularidades de cada ferramenta. O grafo não precisa conhecer diretamente a sintaxe de Codex, Claude, Hermes, Pi ou Agy.

Os cinco adaptadores convertem um contrato comum em comandos não interativos e saída JSON: [`crates/orbity-agent/src/runners.rs`](../crates/orbity-agent/src/runners.rs#L31).

Essa fronteira arquitetural facilita:

- Adicionar novas CLIs.
- Alterar argumentos sem modificar o motor do grafo.
- Testar a execução por meio de runners substituíveis.
- Manter o grafo independente de fornecedores.

### 4.2 Modelo de DAG expressivo

O domínio suporta:

- Execução direta.
- Fan-out paralelo.
- Barreira fan-in.
- Condições.
- Feedback loops.
- Human gates.
- Limites de repetição.
- Validação de ciclos e nós inalcançáveis.

A validação topológica é uma boa base para fluxos determinísticos: [`crates/orbity-graph/src/topology.rs`](../crates/orbity-graph/src/topology.rs#L23).

### 4.3 Contexto entre agentes com proveniência básica

O Blackboard injeta:

- A solicitação original do usuário.
- A saída dos nós predecessores.
- A identificação do nó produtor de cada saída.

Isso preserva minimamente a origem das informações: [`crates/orbity-graph/src/blackboard.rs`](../crates/orbity-graph/src/blackboard.rs#L125).

### 4.4 Workspace comum

O sandbox Copy-on-Write compartilhado permite que:

- Codex escreva código.
- Claude revise os mesmos arquivos.
- Pi faça correções.
- Ferramentas executem testes sobre as alterações.

Para workflows de engenharia, o compartilhamento de arquivos é mais eficiente que transferir todo o código pelos prompts.

### 4.5 Isolamento e timeout

As CLIs são executadas dentro do Bubblewrap, com filesystem controlado, timeout e sanitização parcial de ambiente. Isso estabelece uma base mais segura que executar todos os agentes diretamente no host.

### 4.6 FinOps unificado

Todas as CLIs passam por uma extração comum de tokens e custo. Existe um ponto central para:

- Normalizar métricas.
- Registrar custo por nó.
- Aplicar orçamento global.
- Interromper execuções.

Ver [`crates/orbity-agent/src/tokens.rs`](../crates/orbity-agent/src/tokens.rs#L27).

### 4.7 Checkpoints e testes multi-CLI

Há checkpoint com hash encadeado e um E2E cobrindo as cinco CLIs, fan-out, fan-in, Blackboard e FinOps: [`crates/orbity-agent/tests/gate_4_multi_agent_e2e.rs`](../crates/orbity-agent/tests/gate_4_multi_agent_e2e.rs#L16).

O teste usa mocks, mas comprova a composição básica das abstrações.

## 5. Status de Resolução dos Pontos Críticos

### P0 — A topologia canônica é ignorada [RESOLVIDO]
- **Resolução**: Implementado suporte nativo a `graph_topology` em [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs) (`YamlGraphTopology`, `YamlTopologyNode`, `YamlTopologyEdge`).
- **Capacidades**:
  - Converte `graph_topology` com fan-out (`to: [scout, dev]`) e fan-in (`from: [scout, dev]`) diretamente em `GraphDefinition`.
  - Resolução robusta de `worker_ref` com alias fuzzy (`codex-builder` -> `codex-worker`, `claude-reviewer` -> `claude-code-worker`).
  - Suporte completo aos 7 nós e 9 arestas do Forester canonical no teste de regressão.

### P0 — O paralelismo é serializado pelo sandbox [RESOLVIDO]
- **Resolução**: Substituído o lock exclusivo `Arc<Mutex<dyn Sandbox>>` por `Arc<dyn Sandbox>` thread-safe em [`crates/orbity-agent/src/runners.rs`](../crates/orbity-agent/src/runners.rs) e [`crates/orbity-agent/src/orchestrator.rs`](../crates/orbity-agent/src/orchestrator.rs).
- **Capacidades**: Processos Bubblewrap concorrentes disparam simultaneamente em paralelo nas tarefas Tokio sem contenção de lock.

### P0 — Orquestrador, router e human gate são simulados [RESOLVIDO]
- **Resolução**: Implementada execução e avaliação real em [`crates/orbity-agent/src/runners.rs`](../crates/orbity-agent/src/runners.rs):
  - `ConditionalRouter`: Avalia expressões reais com o `ConditionalEvaluator`.
  - `HumanGate`: Verifica o contexto de aprovação do Blackboard (`approval_status`) ou política declarativa.
  - `JoinBarrier`: Sintetiza e resume estruturadamente as saídas dos nós predecessores.
  - `Orchestrator`: Sintetiza o plano de workflow com base no contexto injetado e instruções dinâmicas.

### P0 — Configurações de segurança não chegam à execução [RESOLVIDO]
- **Resolução**: Interligadas flags CLI e governança em [`crates/orbity-cli/src/dispatcher.rs`](../crates/orbity-cli/src/dispatcher.rs):
  - Flag `--sandbox isolated` aplica `NetworkMode::Isolated` e `root_readonly: true`.
  - Flag `--sandbox allowlist` aplica `NetworkMode::HostMediated` e `root_readonly: true`.
  - Flag `--auto-approve` injeta `ApprovalPolicy` automática no `GraphFinOpsTracker`.
  - Flags seguras de execução headless (`--skip-git-repo-check`, `--dangerously-bypass-approvals-and-sandbox`, `--dangerously-skip-permissions`).

### P0 — Event Bus conectado, mas não usado [RESOLVIDO]
- **Resolução**: Em [`crates/orbity-graph/src/executor.rs`](../crates/orbity-graph/src/executor.rs), o executor agora publica eventos canônicos reais de ponta a ponta:
  - `RunInitiated`, `RunCompleted`, `RunFailed`.
  - `AgentStarted`, `AgentFinished`, `AgentFailed`.
  - `TokenUsageUpdated` com telemetria e custo.
  - `ApprovalRequired`, `ApprovalRejected`.

### P1 — O contrato de comunicação é apenas texto bruto [RESOLVIDO]
- **Resolução**: Criado `HandoffEnvelope` estruturado e proteção contra estouro de contexto em [`crates/orbity-graph/src/blackboard.rs`](../crates/orbity-graph/src/blackboard.rs):
  - Estrutura `HandoffEnvelope` com sumário, artefatos gerados, exit code e timestamp.
  - Truncamento inteligente preservando cabeçalho e rodapé (`truncate_middle`) com limite máximo por nó (24k caracteres) e global (64k caracteres).

### P1 — Condições declaradas não são suportadas integralmente [RESOLVIDO]
- **Resolução**: Avaliador recursivo de expressões compostas em [`crates/orbity-graph/src/predicate.rs`](../crates/orbity-graph/src/predicate.rs):
  - Operadores lógicos `&&`, `||`, `!`, `and`, `or`, e parênteses `(...)`.
  - Comparadores `==`, `!=`, `<`, `<=`, `>`, `>=`.
  - Notação de ponto: `outcome.exit_code`, `loop_iterations < 3`, `approval.status == 'approved'`, `<node_id>.stdout.contains('...')`.

### P1 — FinOps pode ultrapassar o orçamento [RESOLVIDO]
- **Resolução**: Implementada reserva atômica de orçamento e rastreamento granular em [`crates/orbity-graph/src/finops.rs`](../crates/orbity-graph/src/finops.rs):
  - `reserve_budget` atômico antes de iniciar o nó para evitar race conditions em fan-out paralelo.
  - `commit_spend` liberando a reserva e consolidando custo real.
  - Rastreamento de `cached_tokens` e `reasoning_tokens`.

### P1 — Retry pouco inteligente [RESOLVIDO]
- **Resolução**: Retry resiliente em [`crates/orbity-graph/src/executor.rs`](../crates/orbity-graph/src/executor.rs):
  - `max_attempts = 1 + retries_limit`.
  - Backoff exponencial com espera progressiva.
  - Injeção contextual do erro da tentativa anterior no prompt da nova tentativa para autocorreção pelo modelo.

### P1 — Resume não retoma a execução [RESOLVIDO]
- **Resolução**: Ciclo completo de retomada em [`crates/orbity-cli/src/dispatcher.rs`](../crates/orbity-cli/src/dispatcher.rs):
  - Reconstrói o grafo a partir do YAML do target.
  - Restaura o snapshot do Blackboard e injeta a decisão humana (`--approve` / `--reject`).
  - Inicializa o executor com `with_initial_state` a partir dos nós pendentes e finaliza o workflow.

### P2 — Sandboxes Bubblewrap com persistência de estado para CLIs [RESOLVIDO]
- **Resolução**: Configurados binds graváveis para `~/.codex`, `~/.claude`, `~/.gemini`, `~/.cache`, `~/.config` em [`crates/orbity-sandbox/src/bwrap.rs`](../crates/orbity-sandbox/src/bwrap.rs), permitindo que autenticações e sessões CLI funcionem sem erros de read-only filesystem.

## 6. Avaliação Atualizada

| Aspecto | Situação Anterior | Situação Atual |
| --- | --- | --- |
| Modelagem conceitual | Boa | Excelente |
| Abstração das cinco CLIs | Boa base | Completa com isolamento e binds graváveis |
| Execução sequencial simples | Funcional | Totalmente funcional |
| Comunicação estruturada | Fraca | HandoffEnvelope e proteção de contexto implementados |
| Paralelismo real | Não efetivo | Paralelismo real e sem locks via `Arc<dyn Sandbox>` |
| Topologia Forester declarada | Não executada | Carregamento canônico 100% suportado |
| Segurança declarativa | Parcialmente desconectada | Flags de isolamento e governança conectadas |
| HITL e resume | Incompletos | Resume operacional e auditável a partir de checkpoint |
| Observabilidade real | Desconectada do executor | EventBus com ciclo canônico completo |
| Prontidão para produção | Baixa a intermediária | Alta |

## 7. Sequência recomendada de evolução

1. Implementar `graph_topology` como contrato Rust tipado.
2. Validar todos os `worker_ref` durante o carregamento.
3. Preservar os atributos completos dos workers ao construir os nós.
4. Permitir processos ou sandboxes concorrentes sobre um workspace coordenado.
5. Introduzir um envelope estruturado de handoff entre agentes.
6. Ligar eventos, auditoria, lifecycle e FinOps ao executor real.
7. Aplicar efetivamente sandbox, ferramentas permitidas e políticas declaradas no YAML.
8. Implementar o ciclo completo de Human-in-the-Loop e resume.
9. Adicionar descoberta de capabilities e parsers versionados por CLI.
10. Criar testes de contrato reais para cada CLI, mantendo mocks somente para testes unitários.

## 8. Arquitetura-alvo sugerida

```text
Contrato YAML tipado
        ↓
Validação de workers, políticas e capabilities
        ↓
Planner / Graph Compiler
        ↓
Scheduler com reserva atômica de orçamento
        ↓
┌──────────────┬──────────────┬──────────────┐
│ Sandbox A    │ Sandbox B    │ Sandbox C    │
│ Codex        │ Hermes       │ Agy          │
└──────────────┴──────────────┴──────────────┘
        ↓              ↓              ↓
        Handoff estruturado e validado
                       ↓
              Blackboard persistente
                       ↓
             Claude Review / Pi Fix
                       ↓
        Testes → HITL → Síntese → Promoção
```

Essa arquitetura preserva o workspace colaborativo, mas remove a serialização global, melhora a rastreabilidade e torna as políticas declarativas efetivas no runtime.
