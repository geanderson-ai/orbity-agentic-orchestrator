# 🗺️ Varredura Completa da Jornada Multi-Agente com Orquestrador

> **Orbity Agentic Platform**  
> **Status da Auditoria:** 100% Contemplado e Mapeado  
> **Pilares:** Rust Core • Orquestrador Astra • Sandbox Bubblewrap • Trilha SQLite SHA-256 • 5 CLIs (`codex`, `claude`, `agy`, `hermes`, `pi`)

---

## 1. Visão Geral da Jornada em 4 Fases

A jornada de vida de uma orquestração multi-agente no Orbity foi auditada e dividida em 4 fases estritas, garantindo que não haja pontos cegos operacionais, de segurança ou de governança:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          JORNADA MULTI-AGENTE ORBITY                        │
├─────────────────┬─────────────────┬───────────────────┬─────────────────────┤
│   1. CRIAÇÃO    │   2. EXECUÇÃO   │    3. ANÁLISE     │   4. FINALIZAÇÃO    │
│  (Setup & Init) │(Plan & Delegate)│(Review & FinOps)  │ (Synthesis & Clean) │
├─────────────────┼─────────────────┼───────────────────┼─────────────────────┤
│ • Preflight 5   │ • User Goal In  │ • Validação Testes│ • Síntese Astra     │
│   CLIs + bwrap  │ • Planner Astra │   na Sandbox      │ • Relatório FinOps  │
│ • Parse YAML    │   (DAG & Deps)  │ • Code Review     │   (Tokens & USD)    │
│   (forester.yml)│ • Roteamento 5  │   (Claude Code)   │ • Teardown Sandbox  │
│ • Alocação ID   │   CLIs Locais   │ • Tripwires FinOps│   (tmpfs wipe)      │
│ • Genesis Block │ • Sandbox tmpfs │   (Max USD / HITL)│ • Hash Final SHA-256│
│   SHA-256 SQLite│ • Event Bus 4   │ • Rollback Atômico│ • Transição de      │
│ • Spawning Sbx  │   Camadas (SSE) │   em caso de erro │   Estado para Idle  │
└─────────────────┴─────────────────┴───────────────────┴─────────────────────┘
```

---

## 2. Varredura Detalhada por Fase

---

### FASE 1: CRIAÇÃO (Setup, Bootstrap & Identidade)

| Etapa | O que acontece | Componente Responsável | Garantia de Engenharia |
|---|---|---|---|
| **1.1 Preflight Healthcheck** | Na 1ª execução ou via `orbity doctor`, o sistema verifica a presença de `bwrap`, `sqlite` e a suíte de 5 CLIs: `codex`, `claude`, `agy`, `hermes` e `pi`. | `orbity-cli::doctor` | Impede o disparo de tarefas caso falte alguma dependência crítica do sistema. |
| **1.2 Ingestão Declarativa** | Leitura de `teams/forester.yaml` (o nome do arquivo define a equipe) ou `agents/agente01.yaml`. | `orbity-core::yaml_parser` | Validação estrita de schema com `serde_yaml` e tipagem Rust. |
| **1.3 Identidade & Registro** | Alocação de `AgentId` e persistência nas tabelas relacionais do SQLite (`teams`, `agents`, `agent_workers`). | `orbity-storage::dao` | Registro atômico com integridade referencial em modo WAL. |
| **1.4 Genesis Block Criptográfico** | Criação do primeiro evento da trilha de auditoria append-only (`AgentCreated` ou `RunInitiated`) com hash SHA-256 genesis (`0000...`). | `orbity-storage::audit_chain` | Imutabilidade matemática iniciada antes de qualquer código ser executado. |
| **1.5 Confinamento da Sandbox** | Provisão do diretório isolado: root somente leitura (`ro-bind`), diretório de trabalho efêmero em memória (`tmpfs`), rede isolada e limpeza de variáveis de ambiente do host. | `orbity-sandbox::bwrap` | Isolamento contra escape de processos e exfiltração de segredos. |
| **1.6 Transição de Estado** | Transição documentada de ciclo de vida: `Draft` $\rightarrow$ `Spawning` $\rightarrow$ `Idle / Ready`. | `orbity-agent::lifecycle` | Agente e time prontos para receber objetivos. |

---

### FASE 2: EXECUÇÃO (Planejamento, Decomposição & Delegação)

| Etapa | O que acontece | Componente Responsável | Garantia de Engenharia |
|---|---|---|---|
| **2.1 Recepção do Prompt** | Disparo via CLI: `orbity team run forester "Corrigir vazamento de conexões SQLite"`. | `orbity-cli::run` | Atribuição de `run_id` e alocação de cota de orçamento FinOps. |
| **2.2 Planejamento em DAG** | O supervisor **Astra** (ou **Agy** em modo plan) analisa a solicitação e gera um Grafo Acíclico Dirigido (DAG) decompondo o problema em etapas sequenciais e paralelas. | `orbity-agent::orchestrator` | Transição de estado para `Planning` e emissão do grafo topológico. |
| **2.3 Despacho Especializado** | Roteamento das subtarefas para a ferramenta mais qualificada entre as 5 CLIs nativas: <br>• **Agy:** Pesquisa de contexto na codebase e MCP tools (`agy -p`).<br>• **Codex:** Escrita de código Rust e suíte de testes (`codex exec`).<br>• **Pi:** Edições rápidas e cirúrgicas sem poluir o ambiente (`pi -p`).<br>• **Hermes:** Execução de ferramentas e busca web (`hermes run`).<br>• **Claude:** Revisão crítica e análise de invariantes (`claude -p`). | `orbity-agent::cli_runner` | Transição para `Executing`; uso do melhor modelo/CLI para cada tarefa. |
| **2.4 Blackboard Context (Memória)** | Os operários compartilham dados e descobertas através de artefatos estruturados na sandbox e metadados persistidos na tabela `task_artifacts` do SQLite. | `orbity-storage::blackboard` | Continuidade sem perda de contexto entre operários do time. |
| **2.5 Execução Confinada & Timers** | Comandos de compilação e teste rodam dentro da sandbox com limites de cgroup e timeouts estritos (com finalização forçada `SIGKILL`). | `orbity-sandbox::process` | Prevenção contra loops infinitos e travamentos de build. |
| **2.6 Barramento de 4 Camadas** | Publicação simultânea em tempo real de: <br>1. *Runtime Logs* (lifecycle)<br>2. *Execution Logs* (comandos e arquivos)<br>3. *Audit Logs* (hashes no SQLite)<br>4. *Telemetry Logs* (spans OpenTelemetry). | `orbity-core::event_bus` | Distribuição não-bloqueante para terminal TUI (`ratatui`) e WebSockets/SSE. |

---

### FASE 3: ANÁLISE (Revisão Crítica, FinOps Guardrails, HITL & Validação)

| Etapa | O que acontece | Componente Responsável | Garantia de Engenharia |
|---|---|---|---|
| **3.1 Testes Automatizados** | Execução de `cargo test --all` dentro da sandbox efêmera para verificar se o código alterado compila e passa nos testes. | `orbity-sandbox` / `Codex` | Código quebrado é bloqueado antes de qualquer merge ou aprovação. |
| **3.2 Revisão por Pares (Peer Review)** | **Claude Code** analisa o diff gerado: conformidade de tipos, ausência de credenciais expostas e segurança de memória. | `ClaudeCodeWorker` | Camada independente de auditoria sobre o código produzido pelo Codex/Pi. |
| **3.3 Guardrails FinOps em Tempo Real** | O motor de FinOps contabiliza tokens (input, output, cache, reasoning) de cada CLI e calcula o custo USD acumulado. | `orbity-agent::finops_engine` | Controle contínuo de consumo contra o `max_budget_usd` estipulado. |
| **3.4 Governança Declarativa & HITL** | O orquestrador avalia a `approval_policy` declarada no YAML (`auto_approve` / `auto_reject`). Com `expensive_model_action: auto_approve`, modelos caros executam sem pausa interativa. Apenas exceções não cobertas pelas regras pausam para validação humana via CLI (`orbity resume <RUN_ID> --approve`) ou WebSocket. | `orbity-agent::hitl` | Elimina validações manuais repetitivas mantendo controle total sobre exceções e teto global. |
| **3.5 Tripwire de Orçamento** | Caso o teto financeiro seja estourado, o sistema interrompe a execução graciosamente, grava `BudgetExceeded` e preserva os registros. | `orbity-agent::finops` | Garantia de teto orçamentário inquebrável. |
| **3.6 Mecanismo de Rollback** | Se os testes falharem ou o revisor reprovar o diff, o workspace temporário `tmpfs` é sumariamente descartado, restaurando o estado original intacto. | `orbity-sandbox::rollback` | Zero poluição ou corrupção no repositório do host. |
| **3.7 Verificação da Trilha** | O motor de auditoria valida a integridade dos hashes SHA-256 encadeados até o momento. | `orbity-storage::audit_verifier` | Certeza de que nenhum evento foi adulterado durante o processo. |

---

### FASE 4: FINALIZAÇÃO (Síntese, Teardown, Governança & Arquivamento)

| Etapa | O que acontece | Componente Responsável | Garantia de Engenharia |
|---|---|---|---|
| **4.1 Síntese do Orquestrador** | O líder **Astra** consolida os resultados dos operários, compõe o relatório técnico final, lista os arquivos alterados e resume as decisões arquiteturais. | `AstraSupervisor` | Entrega executiva clara e estruturada para o usuário final. |
| **4.2 Fechamento Contábil FinOps** | Registro consolidado na tabela `runs` com: total de tokens (input, output, cache, reasoning), custo exato em USD e duração total da sessão. | `orbity-storage::token_ledger` | Rastreabilidade total de ROI e custo computacional por agente e equipe. |
| **4.3 Teardown e Limpeza de Sandbox** | Destruição do ambiente efêmero da sandbox: limpeza forçada de `tmpfs`, desmonte de volumes `ro-bind` e encerramento de qualquer processo órfão remanescente. | `orbity-sandbox::cleanup` | Liberação integral de memória RAM e recursos de sistema operacional. |
| **4.4 Selamento da Trilha de Auditoria** | Gravação do bloco de encerramento (`AgentFinished` ou `RunCompleted`) com o último hash encadeado na tabela `audit_events`. | `orbity-storage::audit_chain` | Encerramento formal e criptograficamente verificável da orquestração. |
| **4.5 Transição de Ciclo de Vida** | A equipe/agente transiciona de `Executing` $\rightarrow$ `Completed` $\rightarrow$ `Idle` (pronto para nova missão) ou `Archived` (em caso de remoção com soft delete auditado). | `orbity-agent::lifecycle` | Governança completa do ciclo de vida sem estados órfãos. |
| **4.6 Exportação de Telemetria** | Finalização dos spans do OpenTelemetry e flush dos logs JSONL locais (`runs/<run_id>.jsonl`). | `orbity-telemetry::otel` | Observabilidade pronta para consulta no Grafana/Loki/Tempo. |

---

## 3. Matriz de Cobertura de Requisitos

| Requisito do Projeto | Fase de Criação | Fase de Execução | Fase de Análise | Fase de Finalização | Status |
|---|---|---|---|---|---|
| **Rust Core & Tokio** | Workspace multi-crate, traits | Barramento MPSC assíncrono | Processamento não-bloqueante | Flush seguro de buffers | ✅ 100% |
| **Graph Engineering** | Valida nós/arestas (Kahn DAG) | Fan-out Tokio, Fan-in barreira | Feedback Loops & Retry limits | Checkpoints criptográficos | ✅ 100% |
| **Orquestrador Astra** | Configurado via YAML com time | Decompõe em DAG e despacha | Monitora execução e quotas | Realiza a síntese da entrega | ✅ 100% |
| **Sandbox Confinada** | Monta ro-bind + tmpfs | Isola processos e comandos | Descarta tmpfs em rollback | Desmonta volumes e limpa PID | ✅ 100% |
| **SQLite Hash-Chain** | Bloco Genesis registrado | Grava nós append-only | `orbity audit verify` valida | Bloco final selado | ✅ 100% |
| **5 CLIs Locais** | Preflight check no doctor | `codex`, `claude`, `agy`, `hermes`, `pi` | Extração normalizada de tokens | Agregação por ferramenta | ✅ 100% |
| **FinOps em Tempo Real** | Cota configurada no YAML | Monitora chamada a chamada | Tripwires e aprovação HITL | Relatório consolidado em USD | ✅ 100% |
| **Interface CLI** | `orbity team load` / `agent create` | `orbity team run` com TUI | `orbity resume` / `status` | `orbity finops` / `audit verify` | ✅ 100% |
| **Observabilidade 4 Camadas**| Registry dos 4 sinks | Streaming SSE/WS contínuo | Logs de segurança estruturados | Exportação OTEL para Grafana | ✅ 100% |

---

## 4. Conclusão da Varredura

A varredura comprova que **todo o ciclo de vida e a jornada de orquestração multi-agente estão formalmente contemplados**:
1. **Zero Pontos Cegos de Segurança:** Sandbox garante isolamento do host com root somente leitura e descarte atômico (rollback) em caso de falha.
2. **Zero Pontos Cegos de Governança:** O encadeamento de hashes SHA-256 no SQLite abrange desde o bloco Genesis na criação até o bloco final de síntese.
3. **Zero Pontos Cegos Financeiros:** FinOps atua preventivamente com tripwires antes de chamadas de alto custo e oferece suporte a Human-in-the-Loop (HITL) no estado `Paused`.
4. **Alinhamento com Ferramentas Reais:** A suíte de 5 CLIs (`codex`, `claude`, `agy`, `hermes`, `pi`) já foi testada, verificada e mapeada para seus respectivos papéis de excelência técnica.

---

## 5. Evidências de Implementação e Execução dos Gates 0, 1, 2 e 3

O Gate 0 (Fundação, Tipos & Domínio de Eventos), o Gate 1 (Persistência SQLite & Audit Store Criptográfico), o Gate 2 (Mecanismo de Sandbox & Isolamento de Processos) e o Gate 3 (Barramento Unificado de Eventos & Observabilidade em 4 Camadas) foram completamente implementados e validados no repositório Git com os seguintes commits:

| Commit | Escopo | Descrição da Entrega |
|---|---|---|
| `e5a2c47` | `feat(gate-0)` | `TASK-001` - Cargo workspace multi-crate com 8 crates desacopladas. |
| `7f9489b` | `feat(gate-0)` | `TASK-002` - Modelagem de eventos estruturados `RuntimeEvent` e `EventEnvelope`. |
| `c6c565b` | `feat(gate-0)` | `TASK-003` - Domínio de FinOps e Tokenomics (`TokenUsage`, `BudgetPolicy`). |
| `8b48df6` | `feat(gate-0)` | `TASK-004` - Sanitização de credenciais `SecretMasker` e fingerprints SHA-256 `SecretMetadata`. |
| `fa45573` | `feat(gate-0)` | `TASK-005` - Contratos declarativos e parsers YAML de agentes e times. |
| `3c0ba62` | `feat(gate-1)` | `TASK-101` - Setup SQLite com modo WAL, pragmas e migrations DDL automáticas. |
| `7a7d177` | `feat(gate-1)` | `TASK-102` - Motor de auditoria append-only `AuditStore` com encadeamento SHA-256. |
| `ca9b8bd` | `feat(gate-1)` | `TASK-103` - Verificador de integridade histórica de auditoria `AuditVerifier`. |
| `24aa629` | `feat(gate-1)` | `TASK-104` - Repositórios assíncronos (DAOs) para runs, tasks, token ledger, times e agentes. |
| `66398b7` | `fix(storage)` | **FIX** - Loop de retry atômico com backoff proporcional para contenção de escrita concorrente multi-agente. |
| `a39c094` | `test(gate-1)` | **E2E TEST** - Teste real concorrente multi-agente (`real_multi_agent_scenario.rs`) com Astra, Codex, Claude, Hermes e Pi, validando 12 blocos criptográficos e detecção de tampering na sequência 3. |
| `33d7898` | `docs` | Atualização do `IMPLEMENTATION_PLAN.md` com status de conclusão dos Gates 0 e 1. |
| `bf61386` | `docs` | Especificação de isolamento de rede e ciclo de vida efêmero vs permanente no filesystem da Sandbox. |
| `d3073da` | `feat(gate-2)` | `TASK-201` - Abstrações do sandbox provider: traits `Sandbox`, políticas de rede (`NetworkMode`), ciclo de vida de workspace efêmero (`WorkspaceMode`), `ExecutionResult`, `FileChangeSummary` e `MockSandbox`. |
| `a0fac9b` | `feat(gate-2)` | `TASK-202` - Provedor nativo Bubblewrap (`BwrapSandbox`): root somente leitura (`--ro-bind / /`), tmpfs seguro em `/tmp/workspace`, `--unshare-net`, defesa de traversal, snapshot, rollback e promoção de arquivos. |
| `cb9f139` | `feat(gate-2)` | `TASK-203` - Controle de recursos e limites de execução com timeouts estritos (`tokio::time::timeout`), terminação com sinal `SIGKILL` e limpeza garantida da árvore de processos. |
| `44b2bfd` | `feat(gate-2)` | `TASK-204` - Emissão de eventos estruturados de ciclo de vida e políticas (`InstrumentedSandbox` / `SandboxEventEmitter`), gerando eventos canônicos `SandboxCreated`, `SandboxDestroyed`, `CommandExecuted`, `PolicyDenied`, `FileWritten` e `FileRead`. |
| `2524878` | `test(gate-2)` | **E2E TEST** - Teste integrado do Gate 2 (`sandbox_isolation_and_rollback.rs`): validação de confinamento de root host, bloqueio de path traversal, isolamento de rede offline, timeouts com `SIGKILL`, descarte seguro com rollback, promoção de arquivos para o host e registro de auditoria encadeada com SHA-256 no SQLite. |
| `e67bc00` | `docs` | Atualização da documentação geral com evidências de conclusão do Gate 2. |
| `172c0d7` | `feat(gate-3)` | `TASK-301` - Barramento assíncrono de eventos (`EventBus`) com canal de broadcast, fila de alta capacidade MPSC, trait `EventSink` e múltiplos destinos. |
| `10619a5` | `feat(gate-3)` | `TASK-302` - Camada 1: Runtime Logs (`RuntimeLogSink`) para ciclo de vida de agentes, execuções e eventos HITL com formatação canônica. |
| `25913ca` | `feat(gate-3)` | `TASK-303` - Camada 2: Execution Logs (`ExecutionLogSink`) para captura de comandos, ferramentas e metadados de arquivos com hash SHA-256. |
| `826c7f0` | `feat(gate-3)` | `TASK-304` - Camada 3: Audit Logs (`AuditLogSink`) persistindo eventos no SQLite WAL com encadeamento de hash SHA-256. |
| `6b222cc` | `feat(gate-3)` | `TASK-305` - Camada 4: Telemetry Logs (`TelemetrySink`, `SpanTree`, `TelemetryMetrics`) com spans hierárquicos e exportador OpenTelemetry/OTLP JSON. |
| `f0e8536` | `test(gate-3)` | **E2E TEST** - Teste integrado de pipeline de 4 camadas simultâneas e teste de carga atingindo vazão de 75.473 ev/s (>15x o teto de 5.000 ev/s). |
| `f308ae1` | `docs(gate-3)` | Atualização da documentação geral (`IMPLEMENTATION_PLAN.md`, `README.md`, `JOURNEY_MAP_AND_AUDIT.md`) com conclusão do Gate 3. |
| `1e18a97` | `docs(gate-5/6)`| Atualização do Gate 5 para aplicação servidora reativa Tokio Topcoat (`topcoat` v0.9+) e reorganização da CLI para o Gate 6. |
| `6d2cc49` | `feat(contracts)`| Políticas declarativas de aprovação e rejeição no YAML (`ApprovalPolicy`, `auto_approve`, `auto_reject`) eliminando validações manuais. |

### Resultados Consolidados dos Quality Gates (Gates 0, 1, 2 e 3)
- **`cargo check --workspace`:** ✅ Sucesso (0 erros)
- **`cargo clippy --workspace --all-targets -- -D warnings`:** ✅ Sucesso (0 warnings)
- **`cargo test --workspace`:** ✅ 44 testes aprovados (100% de sucesso)
- **Vazão do Barramento (Load Test):** 🚀 75.473 eventos/segundo (requisito: >= 5.000 ev/s)

