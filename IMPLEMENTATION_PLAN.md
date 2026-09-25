# Plano de Implementação: Orquestrador de Agentes Autônomos (Meza Agentic Orchestrator)
## Estrutura em Gates e Tasks

> **Status:** Proposto / Pronto para Execução  
> **Linguagem Base:** Rust (2021/2024 edition)  
> **Interfaces Principais:** CLI (Linha de Comando) & Webhook/SSE/WebSocket para Dashboard  
> **Componentes Obrigatórios:** Rust, CLI (`clap`/`ratatui`), Orquestrador Astra (Supervisor/Planner), Persistência & Auditoria SQLite (Append-Only Hash-Chained), Sandbox (Isolamento de Processos/Contêineres).  
> **Origem dos Requisitos:** `overview.md`

---

## 1. Visão Geral da Arquitetura & Diretrizes Técnicas

O sistema é um **runtime e orquestrador de agentes de IA de alto desempenho e segurança empresarial**, focado em observabilidade em 4 camadas, controle orçamentário em tempo real (FinOps para agentes), execução isolada em sandboxes seguras e trilha de auditoria imutável com garantia criptográfica.

```
                          ┌────────────────────────┐
                          │     CLI Orquestrador   │
                          │      (clap / TUI)      │
                          └───────────┬────────────┘
                                      │
                                      ▼
                        ┌────────────────────────────┐
                        │   Supervisor Astra (Core)  │
                        │   Planner / Router / FinOps│
                        └─────────────┬──────────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              ▼                       ▼                       ▼
     ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
     │  Worker Codex   │     │  Worker Claude  │     │  Worker Hermes  │
     │   (Code/Test)   │     │ (Review/Logic)  │     │  (Search/Tool)  │
     └────────┬────────┘     └────────┬────────┘     └────────┬────────┘
              │                       │                       │
              └───────────────────────┼───────────────────────┘
                                      ▼
                        ┌────────────────────────────┐
                        │     Sandbox Execution      │
                        │ (bwrap / cgroups / isolate)│
                        └─────────────┬──────────────┘
                                      │
                       Eventos Estruturados (JSON)
                                      ▼
                        ┌────────────────────────────┐
                        │   Barramento de Eventos    │
                        │    (Tokio MPSC/Broadcast)  │
                        └─────────────┬──────────────┘
             ┌────────────────────────┼────────────────────────┐
             ▼                        ▼                        ▼
   ┌───────────────────┐    ┌───────────────────┐    ┌───────────────────┐
   │ 1. Runtime Logs   │    │ 2. Execution Logs │    │ 3. Audit Store    │
   │ (Lifecycle/Crash) │    │ (Comandos/Files)  │    │ SQLite Hash-Chain │
   └───────────────────┘    └───────────────────┘    └───────────────────┘
             │                        │                        │
             └────────────────────────┼────────────────────────┘
                                      ▼
                            ┌───────────────────┐
                            │ 4. Telemetry Logs │
                            │ OTEL / FinOps DB  │
                            └─────────┬─────────┘
                                      │
                                      ▼
                           Servidor Axum (SSE/WS)
                           ──> UI Visual (Cards/SVG)
```

### Tecnologias e Crates do Ecossistema Rust
- **Runtime Assíncrono:** `tokio` (multi-threaded, timers, process I/O)
- **CLI Engine:** `clap` (derive API v4), `ratatui` + `crossterm` (para monitoramento interativo em terminal)
- **Persistência & Auditoria:** `sqlx` (SQLite com modo WAL, pooling assíncrono), `sha2` (encadeamento de hashes de auditoria)
- **Serialização & Tipagem:** `serde`, `serde_json`, `chrono`
- **Observabilidade:** `tracing`, `tracing-subscriber`, `opentelemetry`, `tracing-opentelemetry`
- **Isolamento de Sandbox:** `nix`, `caps`, Bubblewrap (`bwrap`) wrapper ou Linux Namespaces/cgroups
- **Servidor Streaming (Visualização):** `axum`, `tower-http`, `tokio-tungstenite` (WebSocket)

---

## 2. Mapa Geral de Gates (Fases)

| Gate | Título | Objetivo Central |
|---|---|---|
| **Gate 0** | **Fundação do Workspace, Tipos & Domínio de Eventos** | Configuração do Cargo workspace multi-crate, modelagem de dados, enums de eventos e mascaramento de segredos. |
| **Gate 1** | **Persistência SQLite & Audit Store Criptográfico** | Armazenamento de execuções/tarefas e ledger append-only com hash encadeado para integridade estrita. |
| **Gate 2** | **Mecanismo de Sandbox & Isolamento de Processos** | Criação, destruição e contenção de ferramentas e comandos executados pelos agentes via sandbox segura. |
| **Gate 3** | **Barramento Unificado de Eventos & Observabilidade em 4 Camadas** | Pipeline de Runtime, Execution, Audit e Telemetry logs via Rust `tracing` e OpenTelemetry. |
| **Gate 4** | **Graph Engineering (`orbity-graph`), Astra Supervisor & Multi-Agent Network** | Computação orientada a grafos com Nós e Arestas, ordenação topológica, loops de feedback, fan-in/fan-out, FinOps e orquestrador Astra. |
| **Gate 5** | **Interface CLI de Orquestração** | Comandos para disparo de tarefas, inspeção de auditoria, verificação de integridade e acompanhamento. |
| **Gate 6** | **Streaming em Tempo Real & Camada de Visualização** | Servidor Axum com WebSockets/SSE emitindo grafos de execução (Cards + SVG Edges) e traces distribuídos. |
| **Gate 7** | **Testes E2E, Validação de Segurança & Hardening** | Testes de injeção de violação de hash, contenção de sandbox, limites de orçamento e benchmarking. |

---

## 3. Detalhamento de Gates e Tasks

---

### GATE 0: Fundação do Workspace, Tipos & Domínio de Eventos

> **Objetivo:** Estabelecer a base estrutural do repositório em Rust, a taxonomia canônica dos eventos de orquestração e a garantia de não-exposição de segredos.

#### Critérios de Entrada
- Repositório Git inicializado.
- Ferramental Rust (toolchain estável >= 1.80) instalado.

#### Tarefas

- [x] **TASK-001: Estruturação do Cargo Workspace Multi-Crate**
  - **Escopo:** Criar estrutura modular para garantir isolamento de responsabilidades e tempos rápidos de compilação.
  - **Módulos:**
    - `crates/orbity-core`: Tipos fundamentais, enums de eventos, structs de tokens e interfaces comuns.
    - `crates/orbity-storage`: Camada SQLite, migrações e audit store encadeado.
    - `crates/orbity-sandbox`: Abstração de processos isolados e confinamento.
    - `crates/orbity-graph`: Motor de Graph Engineering, tipos de Nós e Arestas, ordenação topológica, DAG e ciclos controlados.
    - `crates/orbity-agent`: Adaptadores para Astra, Codex, Claude, Agy, Hermes, Pi e ciclo de vida.
    - `crates/orbity-telemetry`: Configuração de tracing, OpenTelemetry e métricas.
    - `crates/orbity-server`: API Axum, endpoints SSE e WebSockets.
    - `crates/orbity-cli`: Ponto de entrada executável para o usuário final.
  - **Critério de Aceite (DoD):** `cargo check --workspace` compila com sucesso; dependências entre crates devidamente referenciadas via `path`.

- [x] **TASK-002: Modelagem Canônica de Eventos Estruturados**
  - **Escopo:** Implementar em `orbity-core` a taxonomia completa dos eventos do ciclo de vida:
    - Ciclo de Vida: `AgentStarted`, `AgentFinished`, `AgentFailed`.
    - Execução: `CommandExecuted`, `ToolCalled`, `FileRead`, `FileWritten`, `NetworkRequest`.
    - Políticas & Segurança: `PolicyAllowed`, `PolicyDenied`, `SecretRequested`, `SecretGranted`.
    - Sandbox: `SandboxCreated`, `SandboxDestroyed`.
    - FinOps: `TokenUsageUpdated`, `BudgetThresholdReached`, `BudgetExceeded`, `ContextWindowThresholdReached`.
  - **Critério de Aceite (DoD):** Enum `RuntimeEvent` implementando `Serialize`, `Deserialize`, `Clone` e `Debug`, com testes unitários de serialização JSON sem perdas de schema.

- [x] **TASK-003: Modelagem de FinOps e Tokenomics**
  - **Escopo:** Criar as estruturas de auditoria de consumo:
    ```rust
    pub struct TokenUsage {
        pub input_tokens: u64,
        pub output_tokens: u64,
        pub cached_tokens: u64,
        pub reasoning_tokens: u64,
        pub total_tokens: u64,
        pub estimated_cost_usd: Option<f64>,
    }
    pub struct BudgetPolicy {
        pub max_cost_usd: f64,
        pub max_tokens: u64,
        pub max_agent_calls: usize,
        pub expensive_model_approval_threshold: Option<f64>,
    }
    ```
  - **Critério de Aceite (DoD):** Métodos de soma/acumulação e verificação de limites orçamentários com testes unitários cobrindo transbordamento de cota.

- [x] **TASK-004: Sanitização e Redação de Segredos (Secret Masker)**
  - **Escopo:** Mecanismo obrigatório para impedir vazamento de chaves de API, credenciais e dados confidenciais nos logs estruturados.
  - **Especificação:** Registro de metadados com hash ou ID ofuscado (`secret_id: "github_token"`, `value_logged: false`).
  - **Critério de Aceite (DoD):** Teste unitário validando que strings contendo chaves conhecidas (ex: `sk-...`) são mascaradas antes da persistência e serialização de qualquer evento.

- [x] **TASK-005: Modelagem de Contratos de Agentes, Orquestradores e Equipes (YAML + Rust)**
  - **Escopo:** Em `orbity-core`, criar schemas de definição declarativa e tipos de domínio:
    - Structs: `AgentRecord`, `OrchestratorConfig`, `PromptConfig`, `PlanConfig`, `PlanStep`, `WorkerConfig`.
    - Enum: `AgentLifecycleState` (`Draft`, `Spawning`, `Idle`, `Planning`, `Executing`, `Paused`, `Completed`, `Failed`, `Archived`).
    - Parser YAML com `serde_yaml` suportando convenção de pastas (`teams/<nome>.yaml` onde o arquivo vira o nome da equipe, ex: `forester.yaml`, e `agents/<id>.yaml`).
  - **Critério de Aceite (DoD):** Parser lê e valida com sucesso arquivos como `examples/teams/forester.yaml` e `examples/agents/agente01.yaml` gerando structs tipadas e validadas.

#### Critérios de Saída do Gate 0 (Quality Gate)
- Todos os crates configurados e compilando sem warnings (`cargo clippy --workspace -- -D warnings`).
- 100% de cobertura de testes nos parsers de eventos, mascaramento de segredos e parsing de YAMLs de equipes/agentes.

---

### GATE 1: Persistência SQLite & Audit Store Criptográfico

> **Objetivo:** Fornecer armazenamento relacional assíncrono para tarefas, execuções e finanças, juntamente com uma trilha de auditoria append-only imutável baseada em hash encadeado (SHA-256).

#### Critérios de Entrada
- Gate 0 concluído e aprovado.
- Dependência de `sqlx` com backend SQLite configurada com suporte a migrations.

#### Tarefas

- [x] **TASK-101: Configuração do Banco SQLite e Migrações de Esquema**
  - **Escopo:** Criar migrations automatizadas em `orbity-storage` com pragmas de performance (`PRAGMA journal_mode=WAL;`, `PRAGMA synchronous=NORMAL;`, `PRAGMA foreign_keys=ON;`).
  - **Tabelas:**
    - `teams` (name, description, config_yaml, config_hash, created_at, updated_at)
    - `agents` (id, name, team_name, state, orchestrator_model, prompt_system, plan_strategy, plan_json, finops_budget_usd, created_at, updated_at)
    - `agent_workers` (id, agent_id, worker_name, role, model, allowed_tools_json, created_at)
    - `runs` (id, status, initiated_at, completed_at, total_tokens, total_cost_usd, metadata)
    - `tasks` (id, run_id, parent_task_id, agent_name, status, input_prompt, output_result, duration_ms)
    - `sandboxes` (id, run_id, path, created_at, destroyed_at, status)
    - `token_ledger` (id, run_id, task_id, agent_name, input_tokens, output_tokens, cached_tokens, reasoning_tokens, cost_usd, recorded_at)
    - `audit_events` (id, run_id, task_id, sequence_num, event_type, payload_json, previous_hash, current_hash, recorded_at)
  - **Critério de Aceite (DoD):** Migrações executadas com sucesso via código; esquema suporta índices para consultas por `run_id`, `task_id`, `agent_name`, `team_name` e `state`.

- [x] **TASK-102: Motor de Audit Append-Only com Hashes Encadeados**
  - **Escopo:** Implementar a lógica criptográfica para garantir que qualquer adulteração nos eventos passados quebre o hash encadeado:
    $$\text{current\_hash} = \text{SHA256}(\text{previous\_hash} \parallel \text{sequence\_num} \parallel \text{event\_type} \parallel \text{payload\_json} \parallel \text{recorded\_at})$$
  - **Estrutura:** Genesis block para o primeiro evento da `run` (`previous_hash = "0000000000000000000000000000000000000000000000000000000000000000"`).
  - **Critério de Aceite (DoD):** Inserção estritamente atômica de eventos; tentativa de inserir nó com hash incorreto ou fora de ordem é rejeitada.

- [x] **TASK-103: Verificador de Integridade Histórica (Audit Verifier)**
  - **Escopo:** Criar rotina que percorre toda a cadeia de uma `run_id` recalculando os hashes de cada evento para validar a imutabilidade da trilha.
  - **Critério de Aceite (DoD):** Teste unitário e de integração demonstrando que:
    1. Execuções válidas retornam `AuditVerification::Valid`.
    2. Modificação manual de 1 caractere no banco SQLite em `payload_json` acusa erro `AuditVerification::Tampered { event_id, expected_hash, actual_hash }`.

- [x] **TASK-104: Repositórios de Acesso a Dados (DAOs)**
  - **Escopo:** Implementar traits e repositórios para `RunRepository`, `TaskRepository`, `TokenLedgerRepository` e `AuditRepository`.
  - **Critério de Aceite (DoD):** Operações CRUD essenciais com suporte a transações assíncronas do `sqlx`.

#### Critérios de Saída do Gate 1 (Quality Gate)
- Validador criptográfico testado contra cenários normais e de corrupção forçada.
- Teste de concorrência com escrita simultânea de múltiplos workers validando a integridade das sequências de hash no SQLite em modo WAL.

---

### GATE 2: Mecanismo de Sandbox & Isolamento de Processos

> **Objetivo:** Fornecer um ambiente controlado, seguro e auditável para a execução de ferramentas, compiladores (ex: `cargo test`), scripts e comandos disparados pelos agentes, com **controle estrito de chamadas externas de rede** e **ciclo de vida híbrido de filesystem (efêmero durante testes com rollback automático e persistência permanente apenas sob aprovação)**.

#### Critérios de Entrada
- Gate 1 concluído e aprovado.
- Definição clara dos limites de contenção (sistema de arquivos, variáveis de ambiente, processos filhos e isolamento de rede).

#### Diretrizes Arquiteturais de Rede e Filesystem
1. **Controle de Chamadas Externas (Rede):**
   - Chamadas aos modelos de IA (APIs OpenAI, Anthropic, Gemini) ocorrem pelo orquestrador no Host com credenciais protegidas por `SecretMasker`.
   - Comandos internos da sandbox operam sob políticas explícitas:
     - `NetworkMode::Isolated` (Padrão para builds/testes): `--unshare-net`, 100% offline (apenas loopback `lo`), bloqueando exfiltração de dados e SSRF.
     - `NetworkMode::EgressAllowlist(Vec<String>)`: Conexões restritas apenas a domínios pré-aprovados (ex: `crates.io`). Tentativas fora da lista disparam evento `PolicyDenied`.
     - `NetworkMode::HostMediated`: Ferramentas externas de busca (`web.search`, `docs.fetch`) rodam intermediadas pelo supervisor no Host, injetando apenas o texto higienizado na sandbox.
2. **Ciclo de Vida do Filesystem (Efêmero vs. Permanente):**
   - **Root do Host Protegido:** Montado em modo somente leitura (`ro-bind` em `/`, `/usr`, `/lib`, `/bin`).
   - **Workspace Efêmero de Execução (`tmpfs` ou `/tmp/orbity-sandbox-{id}`):** Toda escrita, compilação e teste ocorre em espaço descartável isolado.
   - **Rollback Atômico:** Em caso de falha nos testes ou reprovação na revisão, o diretório efêmero é sumariamente destruído (`cleanup`), mantendo o host 100% intacto.
   - **Promoção Permanente (`Promote`):** Apenas após aprovação de testes e revisão, as alterações validadas são sincronizadas ao repositório permanente do host, emitindo eventos `FileWritten` com hashes SHA-256 no log de auditoria.

#### Tarefas

- [ ] **TASK-201: Abstração e Trait de Sandbox Provider com Políticas de Rede e Filesystem**
  - **Escopo:** Definir a interface em `orbity-sandbox`:
    ```rust
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum NetworkMode {
        Isolated,                      // --unshare-net (100% offline, padrão)
        EgressAllowlist(Vec<String>),  // Apenas domínios autorizados
        HostMediated,                  // Chamadas intermediadas pelo supervisor
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum WorkspaceMode {
        EphemeralTmpfs,                // tmpfs em memória (descarte garantido)
        EphemeralCopyOnWrite(PathBuf), // Cópia de trabalho em /tmp com snapshot
        Direct(PathBuf),               // Apenas para monitoramento sem isolamento
    }

    #[async_trait]
    pub trait Sandbox: Send + Sync {
        async fn initialize(&mut self) -> Result<SandboxId, SandboxError>;
        async fn run_command(&self, cmd: &str, args: &[String], env: &HashMap<String, String>, timeout: Duration) -> Result<ExecutionResult, SandboxError>;
        async fn write_file(&self, relative_path: &Path, content: &[u8]) -> Result<(), SandboxError>;
        async fn read_file(&self, relative_path: &Path) -> Result<Vec<u8>, SandboxError>;
        async fn snapshot(&self) -> Result<SnapshotId, SandboxError>;
        async fn rollback(&self, snapshot: SnapshotId) -> Result<(), SandboxError>;
        async fn promote_changes(&self, target_host_path: &Path) -> Result<Vec<FileChangeSummary>, SandboxError>;
        async fn cleanup(&mut self) -> Result<(), SandboxError>;
    }
    ```
  - **Critério de Aceite (DoD):** Trait compilada suportando modos de rede (`Isolated`, `EgressAllowlist`, `HostMediated`), ciclo de vida efêmero com descarte seguro (`rollback`/`cleanup`) e sincronização de arquivos aprovados (`promote_changes`).

- [ ] **TASK-202: Implementação da Sandbox Nativa (Linux bwrap / Namespace Isolation)**
  - **Escopo:** Implementar o provedor de sandbox usando Bubblewrap (`bwrap`) ou Linux namespaces nativos:
    - Root filesystem montado em modo somente leitura (`ro-bind`).
    - Diretório de trabalho isolado montado em leitura/escrita temporário (`tmpfs` ou pasta efêmera `/tmp/orbity-sandbox-{id}`).
    - Isolamento de PID, IPC e UTC.
    - Isolamento de rede estrito via `--unshare-net` (offline por padrão para comandos de build/testes).
    - Variáveis de ambiente filtradas (limpeza de `AWS_*`, `GITHUB_*`, `OPENAI_*` sensíveis do host).
  - **Critério de Aceite (DoD):** Execução de comando bloqueando acesso a arquivos fora do diretório autorizado (ex: `/etc/passwd` ou `~/.ssh`) e bloqueio total de sockets de rede no modo `Isolated`.

- [ ] **TASK-203: Controle de Recursos e Timeouts Estritos**
  - **Escopo:** Limites de execução por comando (ex: timeout padrão de 30s, cgroup para teto de memória e CPU).
  - **Critério de Aceite (DoD):** Comandos com loop infinito (ex: `yes` ou `while true; do :; done`) são terminados com sinal `SIGKILL` no timeout e retornam evento `CommandTimeout`.

- [ ] **TASK-204: Emissão de Eventos de Ciclo de Vida do Sandbox**
  - **Escopo:** Integrar a execução do sandbox com a geração de eventos: `SandboxCreated`, `SandboxDestroyed`, `CommandExecuted` (com exit code, stdout resumido, stderr e duração em milissegundos) e `PolicyDenied` em caso de tentativa de violação de rede ou filesystem.
  - **Critério de Aceite (DoD):** Cada ação no sandbox produz seu respectivo evento estruturado e preenche os metadados requeridos pelo `overview.md`.

#### Critérios de Saída do Gate 2 (Quality Gate)
- Tentativa de escapar da sandbox, acessar rede no modo isolado ou alterar arquivos protegidos do host resulta em bloqueio testado e comprovado em teste de integração.
- Limpeza garantida de recursos temporários (`cleanup`) mesmo em caso de falha ou interrupção do agente.
- Promoção atômica para o repositório permanente validada apenas após sucesso de testes e revisão.

---

### GATE 3: Barramento Unificado de Eventos & Observabilidade em 4 Camadas

> **Objetivo:** Orquestrar o fluxo de eventos e logs através das 4 camadas descritas no `overview.md` (Runtime, Execution, Audit e Telemetry), suportando OpenTelemetry, logs locais estruturados e SQLite.

#### Critérios de Entrada
- Gate 1 (Storage) e Gate 2 (Sandbox) concluídos.

#### Tarefas

- [ ] **TASK-301: Barramento de Eventos Assíncrono (Event Collector / Bus)**
  - **Escopo:** Criar barramento em `orbity-core` baseado em `tokio::sync::broadcast` e canais `mpsc` para distribuição eficiente e não-bloqueante de eventos estruturados.
  - **Distribuidores (Sinks):**
    - Sink 1: Stdout em JSON estruturado (para consumo de pipes externos).
    - Sink 2: Arquivo local `.jsonl` com rotação por execução (`runs/<run_id>.jsonl`).
    - Sink 3: Audit Store SQLite com hash encadeado (imutabilidade).
    - Sink 4: OpenTelemetry / Tracing span exporter.
    - Sink 5: WebSocket/SSE Broadcaster para a UI.
  - **Critério de Aceite (DoD):** Eventos publicados são recebidos por todos os subscribers sem perda ou contenção excessiva de memória (buffer circular dimensionado).

- [ ] **TASK-302: Camada 1 - Runtime Logs (Lifecycle do Agente e Processo)**
  - **Escopo:** Rastrear início, parada, timeouts, reinicializações e falhas críticas dos agentes.
  - **Critério de Aceite (DoD):** Eventos `AgentStarted`, `AgentFinished`, `AgentFailed` e logs de runtime emitidos com dados de `run_id`, `task_id`, `agent` e estado.

- [ ] **TASK-303: Camada 2 - Execution Logs (Ações Operacionais)**
  - **Escopo:** Registrar detalhadamente cada comando executado, ferramenta invocada, arquivo lido ou modificado no sandbox.
  - **Critério de Aceite (DoD):** Captura de `CommandExecuted` contendo `exit_code`, `duration_ms`, `sandbox_id` e metadados de arquivo (`FileWritten` com hash do arquivo alterado).

- [ ] **TASK-304: Camada 3 - Audit Logs (Governança e Trilha Imutável)**
  - **Escopo:** Gravação em SQLite com encadeamento de hash SHA-256 e políticas de segurança (`PolicyAllowed`, `PolicyDenied`, `SecretRequested`).
  - **Critério de Aceite (DoD):** Gravação síncrona/atômica na tabela `audit_events` via worker dedicado do barramento.

- [ ] **TASK-305: Camada 4 - Telemetry Logs (Métricas & OpenTelemetry)**
  - **Escopo:** Configurar `tracing-subscriber` com camadas OpenTelemetry para coletar spans hierárquicos:
    ```
    run
     └─ span: astra (supervisor)
          ├─ span: codex (worker)
          ├─ span: hermes (worker)
          └─ span: claude (worker)
    ```
  - **Critério de Aceite (DoD):** Exportação de métricas de duração de tarefas, contagem de chamadas e compatibilidade com collectors OpenTelemetry (Loki/Grafana/Tempo).

#### Critérios de Saída do Gate 3 (Quality Gate)
- Pipeline de 4 camadas operando simultaneamente com um único evento de entrada gerando logs nos destinos configurados.
- Teste de carga demonstrando vazão de pelo menos 5.000 eventos/segundo no barramento local sem travamentos.

---

### GATE 4: Graph Engineering (`orbity-graph`), Astra Supervisor & Multi-Agent Network

> **Objetivo:** Implementar o motor de computação orientada a grafos (**Graph Engineering** com Nodes e Edges) em `orbity-graph`, o agente supervisor Astra responsável por planejar e decompor tarefas em grafos dirigidos, orquestração da suíte de 5 CLIs (`codex`, `claude`, `agy`, `hermes`, `pi`), controle estrito de FinOps por nó/aresta e checkpoints imutáveis de estado.

#### Critérios de Entrada
- Gate 2 (Sandbox) e Gate 3 (Barramento de Eventos) operacionais.
- Modelagem de tipos base em `orbity-core` e tabelas SQLite em `orbity-storage`.

#### Tarefas

- [ ] **TASK-401: Modelagem de Graph Engineering em Rust (`orbity-graph`) - Nodes, Edges e GraphState**
  - **Escopo:** Criar o crate `crates/orbity-graph` e modelar as estruturas fundamentais:
    ```rust
    pub enum NodeKind {
        Supervisor { engine: String },            // Ex: Astra planner/evaluator
        Agent { cli: CliType, config: AgentSpec },// Ex: Codex, Claude, Agy, Hermes, Pi
        Tool { command: String, timeout_secs: u64 },
        ConditionalRouter { predicate_expr: String },
        HumanGate { prompt: String, timeout_secs: Option<u64> },
        JoinBarrier { quorum: Option<usize> },
    }

    pub enum EdgeKind {
        Direct,                                   // A -> B sequencial
        ParallelFanOut,                          // A -> [B, C, D]
        BarrierFanIn,                            // [B, C, D] -> E
        Conditional { predicate: String },        // Transição com predicado lógico
        FeedbackLoop { max_iterations: u32 },    // Loop de retorno e correção com teto
    }

    pub struct GraphNode {
        pub id: NodeId,
        pub kind: NodeKind,
        pub retries_limit: u32,
        pub budget_limit_usd: Option<Decimal>,
    }

    pub struct GraphEdge {
        pub from: NodeId,
        pub to: NodeId,
        pub kind: EdgeKind,
        pub payload_filter: Option<Vec<String>>,
    }

    pub struct GraphDefinition {
        pub id: GraphId,
        pub name: String,
        pub nodes: HashMap<NodeId, GraphNode>,
        pub edges: Vec<GraphEdge>,
        pub start_node: NodeId,
        pub terminal_nodes: HashSet<NodeId>,
    }
    ```
  - **Critério de Aceite (DoD):** Modelagem completa com serialização/deserialização `serde`, construtores fluentes (`GraphBuilder`) e validação estrutural sem dependências circulares não-controladas.

- [ ] **TASK-402: Algoritmos de Topologia: Validação DAG, Ordenação Topológica e Detecção de Deadlocks**
  - **Escopo:** Implementar algoritmos de teoria dos grafos:
    - Ordenação Topológica com o Algoritmo de Kahn para planejar a ordem de execução dos nós.
    - Algoritmo de Tarjan ou DFS com cores para detecção de ciclos não-declarados (apenas arestas explicitamente marcadas como `FeedbackLoop` são aceitas; ciclos acidentais são rejeitados na compilação do grafo).
    - Validação de alcançabilidade: verificar se todos os nós atingem um nó terminal e se não há nós órfãos.
  - **Critério de Aceite (DoD):** Testes unitários com topologias válidas (diamante, pipeline sequencial, fan-out/fan-in) e grafos inválidos (ciclos sem limite, deadlocks) com erro semântico claro.

- [ ] **TASK-403: Motor de Execução Assíncrono (`GraphExecutor`): Fan-out Paralelo e Fan-in/Join**
  - **Escopo:** Implementar em `orbity-graph` o executor concorrente Tokio:
    - `Fan-out`: Dispara nós prontos concorrentemente usando `tokio::spawn`, cada um operando em sua própria sandbox confinada.
    - `Fan-in / Barrier`: Nó receptor aguarda a conclusão de todos os nós precursores (ou quórum especificado) antes de desbloquear.
    - Agregação de saídas: Consolida as saídas dos nós precursores no Blackboard compartilhado.
  - **Critério de Aceite (DoD):** Teste de integração com 3 nós em paralelo reduzidos para 1 nó agregador executados sem race conditions.

- [ ] **TASK-404: Roteamento Condicional, Predicados e Feedback Loops com Limites de Ciclos**
  - **Escopo:** Implementar a lógica dinâmica de transição:
    - Avaliação de predicados (`outcome == 'failed'`, `test_exit_code != 0`, `confidence_score >= 0.85`).
    - Feedback Loops: Quando um teste ou auditoria falha, o fluxo retorna para o nó gerador com o log de erro e diff como contexto.
    - Circuit Breaker: Cada loop possui contador atômico (`current_iterations`); atingindo `max_iterations`, o loop aborta com evento `GraphLoopBudgetExceeded`.
  - **Critério de Aceite (DoD):** Simulação de teste com falha inicial, 2 retries de correção via feedback loop e sucesso no terceiro retry; e teste de estouro de teto de retries.

- [ ] **TASK-405: Checkpoints de Estado de Grafo no SQLite com Encadeamento de Hashes**
  - **Escopo:** Persistir o estado do grafo a cada transição de aresta:
    ```rust
    pub struct GraphStateCheckpoint {
        pub execution_id: Uuid,
        pub step_number: u32,
        pub active_nodes: Vec<NodeId>,
        pub completed_nodes: Vec<NodeId>,
        pub blackboard_snapshot: serde_json::Value,
        pub parent_hash: [u8; 32],
        pub state_hash: [u8; 32],
    }
    ```
    - Permite suspender e retomar execuções de grafos longos (`Resume from checkpoint`).
    - Se a CLI for interrompida (ex: `SIGINT` ou reboot do host), `orbity resume <RUN_ID>` recarrega o estado imutável do SQLite e continua a partir do último checkpoint sem retrabalho.
  - **Critério de Aceite (DoD):** Interrupção forçada no meio de um grafo com 5 nós e recuperação bem-sucedida a partir do SQLite validando a cadeia criptográfica.

- [ ] **TASK-406: Trait de Execução de Agentes e Adaptadores da Suíte de 5 CLIs**
  - **Escopo:** Criar a interface de nó de agente em `orbity-agent` e implementar runners para:
    - `CodexCliWorker`: `codex exec [PROMPT]` dentro da sandbox para geração e testes de código.
    - `ClaudeCodeWorker`: `claude -p [PROMPT] --output-format json --dangerously-skip-permissions` para arquitetura e revisão.
    - `AgyAntigravityWorker`: `agy -p [PROMPT] --output-format json --effort medium|high` para pesquisa profunda e planejamento.
    - `HermesAgentWorker`: `hermes run [PROMPT] --usage-file <path>` para chamadas de ferramentas externas.
    - `PiAssistantWorker`: `pi -p [PROMPT] --mode json --no-session` para refatoração e edições rápidas.
  - **Critério de Aceite (DoD):** Adaptadores encapsulados como `GraphNode` executáveis via `GraphExecutor`.

- [ ] **TASK-407: Normalização de Métricas de Tokens por CLI (Codex, Claude, Agy, Hermes, Pi)**
  - **Escopo:** Extratores de tokens normalizados (input, output, cache, reasoning, custo estimado) integrados ao barramento de eventos.
  - **Critério de Aceite (DoD):** Cada nó de agente emite evento `TokenUsageUpdated` estruturado com medição por nó e agregada do grafo.

- [ ] **TASK-408: Agente Orquestrador Astra (Supervisor, Graph Planner & Blackboard Memory)**
  - **Escopo:** Lógica do supervisor Astra que:
    1. Analisa a intenção do usuário e sintetiza uma `GraphDefinition` com nós especializados e arestas.
    2. Coordena a memória Blackboard (`TaskArtifact`, leituras e escritas isoladas).
    3. Conduz a síntese final dos resultados do grafo.
  - **Critério de Aceite (DoD):** Astra gera dinamicamente um grafo de execução com nós e arestas válidos a partir de um prompt em linguagem natural.

- [ ] **TASK-409: Motor de Orçamento FinOps por Nó/Aresta, Tripwires e Human-in-the-Loop (HITL)**
  - **Escopo:** 
    - Verificação de orçamento antes de cada disparo de nó e travessia de aresta.
    - Suporte a nós `HumanGate`: pausa o grafo, grava checkpoint e emite `ApprovalRequired`.
    - Retomada com `orbity resume <RUN_ID> --approve`.
    - Tripwire orçamentário rígido gerando `BudgetExceeded`.
  - **Critério de Aceite (DoD):** Grafo pausado em nó de aprovação humana e retomado via CLI sem perder contexto.

- [ ] **TASK-410: Gerenciador de Ciclo de Vida do Agente e Parser Declarativo de Topologia de Grafo (YAML)**
  - **Escopo:** Implementar CRUD de agentes (`agentCreate`, `agentList`, `agentGet`, `agentUpdate`, `agentDelete`) e deserializador YAML para equipes em grafo (`teams/forester.yaml` com blocos `nodes:` e `edges:`).
  - **Critério de Aceite (DoD):** Carregamento de `examples/teams/forester.yaml` instanciando um `GraphDefinition` com validação de tipagem e integridade.

#### Critérios de Saída do Gate 4 (Quality Gate)
- O motor `orbity-graph` executa grafos com fan-out paralelo, fan-in de barreira e loops de feedback controlados com teto de repetições.
- Checkpoints criptográficos em SQLite permitem retomada de grafos interrompidos sem perda de integridade.
- Orquestrador Astra coordena tarefas respeitando limites de FinOps por nó e teto global.
- Suíte de 5 CLIs e nós de aprovação humana (HITL) operando de forma integrada na topologia.

---

### GATE 5: Interface CLI de Orquestração

> **Objetivo:** Fornecer a linha de comando completa (`meza`) para submissão de tarefas, acompanhamento em tempo real, auditoria e governança do sistema.

#### Critérios de Entrada
- Gate 1 a 4 operando de forma integrada.

#### Tarefas

- [ ] **TASK-501: Estrutura de Comandos com `clap` (v4 Derive)**
  - **Escopo:** Criar os subcomandos da CLI `orbity`:
    - `orbity run <PROMPT>`: Inicia uma nova orquestração. Opções: `--budget-usd <VAL>`, `--max-tokens <VAL>`, `--sandbox <MODE>`, `--interactive`.
    - `orbity resume <RUN_ID> [--approve|--reject]`: Retoma uma execução pausada pelo guardrail FinOps (Human-in-the-Loop).
    - `orbity status <RUN_ID>`: Exibe o status da execução, nós ativos e consumo de tokens.
    - `orbity logs <RUN_ID>`: Exibe logs (com filtros: `--layer runtime|execution|audit|telemetry`, `--json`).
    - `orbity audit verify <RUN_ID>`: Executa a validação criptográfica da trilha de auditoria SQLite.
    - `orbity finops summary [--since <DATE>]`: Apresenta relatório consolidado de gastos por ferramenta e modelo.
    - `orbity sandbox list/clean`: Inspeciona e limpa ambientes de sandbox residuais.
  - **Critério de Aceite (DoD):** Comandos com ajuda completa (`--help`), validação tipada de argumentos e saídas consistentes.

- [ ] **TASK-502: Modo Streaming no Terminal & TUI com `ratatui`**
  - **Escopo:** Exibição interativa durante o comando `meza run`:
    - Cabeçalho: Status geral, tempo decorrido, tokens totais e custo estimado acumulado.
    - Painel de Agentes: Cards em grade com estado (`running`, `waiting`, `done`, `failed`) e consumo individual.
    - Log View: Streaming dos eventos operacionais recebidos pelo barramento.
  - **Critério de Aceite (DoD):** Terminal atualizado sem flickering; encerramento limpo via `Ctrl+C` com cancelamento gracioso dos processos e sandboxes filhos.

- [ ] **TASK-503: Modo Headless / Pipeline UNIX (JSON Output)**
  - **Escopo:** Permitir uso do `orbity run --output json` para que outras ferramentas, CI/CD ou scripts leiam stdout formatado linha a linha.
  - **Critério de Aceite (DoD):** Nenhuma mensagem informativa polui o canal de stdout no modo JSON; logs operacionais são direcionados para stderr ou arquivo.

- [ ] **TASK-504: Subcomandos de Agentes e Equipes Declarativas (`orbity agent` e `orbity team`)**
  - **Escopo:** Implementar subcomandos CLI para CRUD e carregamento via YAML:
    - `orbity agent create [-f <YAML>]` / `orbity agent list` / `orbity agent get <ID>` / `orbity agent update <ID>` / `orbity agent delete <ID>`.
    - `orbity team load <PATH_YAML>` (ex: `./examples/teams/forester.yaml`).
    - `orbity team list` / `orbity team run <TEAM_NAME> <PROMPT>`.
  - **Critério de Aceite (DoD):** Comandos com autocomplete, formatação de saída amigável em tabelas no terminal e execução de ponta a ponta a partir de arquivos YAML.

- [ ] **TASK-505: Sistema de Preflight Health Check na Inicialização da CLI (`orbity init` / `orbity doctor`)**
  - **Escopo:** Rotina de bootstrap automático executada na primeira execução da CLI ou sob demanda:
    - Verificação de runtime de sandbox (Bubblewrap `/usr/bin/bwrap`).
    - Verificação e descoberta automática das 5 ferramentas no PATH:
      1. `codex` (Codex CLI)
      2. `claude` (Claude Code)
      3. `agy` (Antigravity CLI)
      4. `hermes` (Hermes Agent)
      5. `pi` (Pi AI Coding Assistant)
    - Exibição de sumário formatado no terminal com versões e paths detectados.
    - Diagnóstico de eventuais binários ausentes com instruções de resolução e fallbacks de roteamento.
  - **Critério de Aceite (DoD):** Comando `orbity doctor` e preflight check na 1ª execução detectam com precisão as 5 ferramentas instaladas no sistema e persistem o status de saúde.

- [ ] **TASK-506: Motor de Reconciliação Declarativa Automática na Inicialização da CLI (Folder Scanner & Hash Sync)**
  - **Escopo:** Varredura atômica em sub-milissegundos disparada a cada execução da CLI nas pastas `agents/` e `teams/`:
    - Leitura dos arquivos `.yaml` e cálculo do hash SHA-256 do conteúdo.
    - Comparação instantânea com a coluna `config_hash` das tabelas `agents` e `teams` no SQLite.
    - Detecção automática de:
      - Arquivo novo $\rightarrow$ aciona `agentCreate` / `team_load`, aloca `AgentId` e insere no SQLite.
      - Arquivo alterado $\rightarrow$ aciona `agentUpdate`, faz hot-reload de prompts/planos/orçamentos e atualiza a base.
      - Arquivo excluído $\rightarrow$ marca soft-delete (`Archived`) auditado preservando o histórico.
      - Arquivo idêntico $\rightarrow$ nenhuma ação no banco (overhead < 2ms).
    - Exibição de sumário visual informativo no terminal quando houver mudanças sincronizadas.
  - **Critério de Aceite (DoD):** Adição, edição e remoção de arquivos em `examples/agents/` sincronizam automaticamente o banco SQLite e emitem eventos `AgentSyncedFromYaml` na trilha com hash encadeado.

#### Critérios de Saída do Gate 5 (Quality Gate)
- O binário compilado `orbity` responde a todos os subcomandos de forma idiomática e amigável.
- Preflight health check executado com sucesso validando as 5 CLIs (`codex`, `claude`, `agy`, `hermes`, `pi`).
- Reconciliação automática das pastas `agents/` e `teams/` funcional, sincronizando novidades e alterações sem intervenção manual.
- Comandos de gestão de agentes (`agent create`, `list`, `update`, `delete`) e equipes (`team load`, `run`) testados e operacionais.
- Verificação de auditoria executável diretamente pela linha de comando retornando código de saída `0` para integridade confirmada e `1` para violação.

---

### GATE 6: Streaming em Tempo Real & Camada de Visualização

> **Objetivo:** Disponibilizar servidor Axum embutido no runtime com endpoints de WebSocket e Server-Sent Events (SSE) para alimentar dashboards de observabilidade em tempo real (Agent Cards, SVG Edges e Trace Views).

#### Critérios de Entrada
- Gate 3 (Barramento) e Gate 4 (Orquestração Astra) implementados.

#### Tarefas

- [ ] **TASK-601: Servidor Axum Embutido e API de Eventos**
  - **Escopo:** Em `orbity-server`, disponibilizar servidor HTTP/WebSocket que pode ser iniciado pelo comando `meza serve` (ou `orbity serve`) ou como thread no `orbity run --dashboard`:
    - `GET /api/v1/runs`: Listagem de execuções históricas.
    - `GET /api/v1/runs/:id`: Detalhes de uma execução, árvore de tarefas e tokens.
    - `GET /api/v1/runs/:id/events/sse`: Stream Server-Sent Events de todos os eventos da execução.
    - `GET /api/v1/runs/:id/ws`: Conexão bidirecional WebSocket para controle e streaming.
  - **Critério de Aceite (DoD):** Endpoints com respostas JSON rápidas e streaming contínuo sem queda de conexões.

- [ ] **TASK-602: Estrutura do Grafo de Execução em Tempo Real (Graph Payload)**
  - **Escopo:** Endpoint e payload que sintetiza a topologia dos agentes:
    - Nós: Agente (`Astra`, `Codex`, `Claude`, `Hermes`), estado atual (`running`, `waiting`, `failed`, `done`), tokens gastos, custo em dólar.
    - Arestas: Direção da delegação (ex: `Astra -> Codex`, `Astra -> Claude`).
  - **Critério de Aceite (DoD):** Atualização imediata do grafo sempre que uma tarefa transiciona de estado ou consome tokens.

- [ ] **TASK-603: Dashboard Web Leve (HTML + SVG Puro / Cards sem sobrecarga de BPM)**
  - **Escopo:** Servir interface web estática embutida no binário Rust (via `rust-embed` ou `include_str!`):
    1. **Graph View:** Cards dos agentes conectados por curvas SVG dinâmicas.
    2. **Timeline View:** Gráfico estilo Gantt da duração de cada agente na execução.
    3. **Trace View:** Spans hierárquicos mostrando a sequência exata de chamadas de ferramentas e comandos.
    4. **FinOps Widget:** Marcadores de consumo de tokens vs. orçamento máximo estipulado.
  - **Critério de Aceite (DoD):** Dashboard abre no navegador com latência inferior a 100ms via WebSocket e renderiza a orquestração em tempo real.

#### Critérios de Saída do Gate 6 (Quality Gate)
- O dashboard web conecta ao backend Rust, consome a stream de eventos e reflete com precisão os estados dos agentes sem necessidade de recarregar a página.

---

### GATE 7: Testes E2E, Validação de Segurança & Hardening

> **Objetivo:** Validação integral e exaustiva de todo o sistema contra falhas de segurança, estouros orçamentários, corrupção de dados e falhas em sandbox.

#### Critérios de Entrada
- Todos os Gates anteriores (0 a 6) implementados.

#### Tarefas

- [ ] **TASK-701: Teste de Resistência e Adulteração de Auditoria**
  - **Escopo:** Teste automatizado que executa uma orquestração de 5 etapas, altera arbitrariamente 1 byte na tabela `audit_events` do SQLite e executa `meza audit verify`.
  - **Critério de Aceite (DoD):** A verificação detecta a fraude com 100% de sucesso e aponta o índice exato do evento violado.

- [ ] **TASK-702: Teste de Confinamento Estrito da Sandbox**
  - **Escopo:** Disparar agente simulado instruído a:
    1. Ler `~/.ssh/id_rsa`.
    2. Escrever em `/bin/malicious_exec`.
    3. Abrir socket de rede externa quando a política for offline.
  - **Critério de Aceite (DoD):** Todas as tentativas falham no nível do sistema operacional (sandbox) e produzem eventos `PolicyDenied` gravados no log de auditoria.

- [ ] **TASK-703: Teste de Tripwire Orçamentário e FinOps**
  - **Escopo:** Configurar orçamento de \$0.10 e disparar agentes gerando chamadas consecutivas que somem \$0.11.
  - **Critério de Aceite (DoD):** O motor de FinOps bloqueia a execução no limite de \$0.10, finaliza a execução com status `BudgetExceeded` e preserva os registros no SQLite.

- [ ] **TASK-704: Teste de Stress Multi-Agente Concorrente**
  - **Escopo:** Executar 4 instâncias concorrentes do supervisor Astra orquestrando Codex, Hermes e Claude simultaneamente.
  - **Critério de Aceite (DoD):** Nenhuma contenção de lock fatal no SQLite (modo WAL), integridade dos hashes preservada em todas as 4 runs e ausência de vazamento de memória.

- [ ] **TASK-705: Documentação Operacional e Guia de Execução**
  - **Escopo:** Elaborar documentação completa de arquitetura, manuais da CLI e receitas de configuração para implantação em produção.
  - **Critério de Aceite (DoD):** Manual disponibilizado com exemplos de comandos, esquemas JSON e instruções de instalação do binário.

#### Critérios de Saída do Gate 7 (Quality Gate Final)
- Suite completa de testes passando (`cargo test --workspace`).
- Zero falhas de segurança em testes de escape de sandbox.
- Relatório de auditoria e FinOps 100% verificado e validado.

---

## 4. Matriz de Rastreabilidade de Requisitos

A tabela abaixo valida que todas as exigências estritas foram mapeadas para tarefas de implementação:

| Requisito Obrigatório | Gates Relacionados | Tasks Específicas | Como é Atendido |
|---|---|---|---|
| **Rust** | Gate 0 ao 7 | Todas | Arquitetura 100% nativa em Rust, tipagem estrita com Serde, Tokio e SQLx. |
| **CLI** | Gate 5 | TASK-501, 502, 503 | Binário `meza` com comandos `run`, `status`, `logs`, `audit verify`, `finops`. |
| **Astra** | Gate 4 | TASK-403, 404 | Supervisor de planejamento, decomposição, orquestração multi-agente e síntese. |
| **SQLite** | Gate 1 | TASK-101, 102, 103, 104 | Armazenamento de runs, tasks, métricas de tokens e ledger de auditoria com hash encadeado. |
| **Sandbox** | Gate 2 | TASK-201, 202, 203, 204 | Isolamento de comandos de agentes via Bubblewrap/Namespaces com cgroups e timeouts. |
| **Observabilidade 4 Camadas** | Gate 3 | TASK-301, 302, 303, 304, 305 | Runtime, Execution, Audit e Telemetry integrados ao Rust Tracing e OpenTelemetry. |
| **FinOps & Tokens** | Gate 4 | TASK-402, 404 | Contabilização de input, output, cache, reasoning e enforcement de tetos orçamentários. |
| **Visualização em Tempo Real** | Gate 6 | TASK-601, 602, 603 | Axum com SSE/WebSocket, topologia em cards e arestas SVG em tempo real. |

---

## 5. Ordem de Dependência e Próximos Passos

```
[Gate 0: Core Types]
       │
       ├──────────────────────────┐
       ▼                          ▼
[Gate 1: SQLite Storage]   [Gate 2: Sandbox]
       │                          │
       └──────────┬───────────────┘
                  ▼
       [Gate 3: Observabilidade]
                  │
                  ▼
       [Gate 4: Astra & FinOps]
                  │
       ┌──────────┴───────────────┐
       ▼                          ▼
[Gate 5: CLI Engine]       [Gate 6: Web Streaming]
       │                          │
       └──────────┬───────────────┘
                  ▼
       [Gate 7: Hardening & E2E]
```

---

## 6. Registro de Execução e Evidências Criptográficas de Commits (Gates 0 e 1)

> **Status Atual:** Gate 0 e Gate 1 concluídos com 100% de aprovação e validados com testes unitários, de concorrência e integração multi-agente real.

### Tabela de Evidências por Commit

| Commit | Gate / Escopo | Task | Descrição da Entrega Técnica |
|---|---|---|---|
| `e5a2c47` | `feat(gate-0)` | **TASK-001** | Estruturação do Cargo Workspace Multi-Crate com 8 crates desacopladas (`orbity-core`, `orbity-storage`, `orbity-sandbox`, `orbity-graph`, `orbity-agent`, `orbity-telemetry`, `orbity-server`, `orbity-cli`). |
| `7f9489b` | `feat(gate-0)` | **TASK-002** | Modelagem canônica dos eventos estruturados no enum `RuntimeEvent` e `EventEnvelope`. |
| `c6c565b` | `feat(gate-0)` | **TASK-003** | Modelagem de FinOps e Tokenomics (`TokenUsage`, `BudgetPolicy`, métodos de acumulação e verificação de teto). |
| `8b48df6` | `feat(gate-0)` | **TASK-004** | Sanitização e redação de segredos com `SecretMasker` e metadados de credenciais (`SecretMetadata`). |
| `fa45573` | `feat(gate-0)` | **TASK-005** | Modelagem de contratos declarativos e parsers YAML de agentes e equipes (`forester.yaml`, `agente01.yaml`). |
| `3c0ba62` | `feat(gate-1)` | **TASK-101** | Configuração do pool SQLite com WAL pragmas (`WAL`, `NORMAL`, `foreign_keys`, `busy_timeout=5000`) e migrações DDL. |
| `7a7d177` | `feat(gate-1)` | **TASK-102** | Motor de auditoria append-only `AuditStore` com encadeamento criptográfico SHA-256 e bloco Genesis. |
| `ca9b8bd` | `feat(gate-1)` | **TASK-103** | Verificador de integridade histórica de auditoria `AuditVerifier` com validação de blocos sequenciais e detecção de fraude. |
| `24aa629` | `feat(gate-1)` | **TASK-104** | Repositórios de acesso a dados (DAOs): `RunDao`, `TaskDao`, `TokenLedgerDao`, `TeamDao` e `AgentDao`. |
| `66398b7` | `fix(storage)` | **FIX** | Correção de contenção em concorrência multi-agente via loop atômico de retry com backoff em `AuditStore::append_event`. |
| `a39c094` | `test(gate-1)` | **E2E TEST** | Teste em cenário real multi-agente concorrente (`real_multi_agent_scenario.rs`) com Astra, Codex, Claude, Hermes e Pi, validando 12 blocos criptográficos no SQLite e detecção de tampering na sequência 3. |
| `33d7898` | `docs` | **DOCS** | Atualização do `IMPLEMENTATION_PLAN.md` com marcação de tarefas concluídas nos Gates 0 e 1. |

### Resultados dos Quality Gates
- `cargo check --workspace`: ✅ Sucesso (0 erros)
- `cargo clippy --workspace --all-targets -- -D warnings`: ✅ Sucesso (0 warnings)
- `cargo test --workspace`: ✅ 21 testes aprovados (100% sucesso)

