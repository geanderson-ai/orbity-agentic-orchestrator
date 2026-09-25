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
| **3.4 Human-in-the-Loop (HITL)** | Se o custo de um modelo exceder `expensive_model_approval_threshold_usd`, o orquestrador transiciona para o estado `Paused`, aguardando autorização humana via CLI (`orbity resume <RUN_ID> --approve`) ou WebSocket. | `orbity-agent::hitl` | Nenhuma cobrança descontrolada é feita sem consentimento prévio. |
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
