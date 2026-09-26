# 📜 Contratos, Ciclo de Vida e Setup Declarativo (YAML)

> **Orbity Multi Agentic Harness**  
> **Linguagem Base:** Rust (2021/2024 edition)  
> **Crates Principais:** `orbity-core`, `orbity-agent`, `orbity-storage`, `orbity-cli`

---

## 1. Princípio Arquitetural Fundamental

> **"Todo Agente criado possui uma identidade única (`AgentId`) e encapsula, obrigatoriamente, um Orquestrador interno. Este Orquestrador possui uma Topologia de Grafo Computacional (Nodes & Edges), um Prompt comportamental e um Time de Agentes Operários interconectados ao seu redor."**

No Orbity, um "Agente" não é apenas uma simples chamada a uma LLM com memória curta. Ele é uma **Unidade Autônoma de Computação, Grafo e Governança**:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        AGENTE (ID: agt_01h89x2k)                       │
│                                                                        │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │                 ORQUESTRADOR INTERNO (Supervisor)              │   │
│   │                                                                │   │
│   │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │   │
│   │  │      PROMPT      │  │  GRAPH TOPOLOGY  │  │    FINOPS    │  │   │
│   │  │ System / Invars  │  │  Nodes & Edges   │  │ Budget / USD │  │   │
│   │  └──────────────────┘  └──────────────────┘  └──────────────┘  │   │
│   └───────────────────────────────┬────────────────────────────────┘   │
│                                   │                                    │
│                 Executa e Transiciona o Grafo de Tarefas               │
│                                   ▼                                    │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │         REDE TOPOLÓGICA DE OPERÁRIOS (Graph Engineering)       │   │
│   │                                                                │   │
│   │         [Node: Topcoat Lead] ─ (Fan-out Paralelo) ───┐         │   │
│   │                 │                                    │         │   │
│   │                 ▼                                    ▼         │   │
│   │        [Node: Codex Dev]                    [Node: Hermes]     │   │
│   │                 │                                    │         │   │
│   │                 └───────► [Barrier Fan-in] ◄─────────┘         │   │
│   │                                  │                             │   │
│   │                                  ▼                             │   │
│   │                        [Node: Claude Reviewer]                 │   │
│   │                                  │                             │   │
│   │                                  ▼                             │   │
│   │                     [Tool Node: Sandbox Tests]                 │   │
│   │                                  │                             │   │
│   │                 (Feedback Loop se Falhar / HITL se Passar)     │   │
│   └──────────────────────────────────┬─────────────────────────────┘   │
│                                      ▼                                 │
│                       CONFINAMENTO EM SANDBOX (bwrap)                  │
│                   TRILHA DE CHECKPOINTS SHA-256 (SQLite)               │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Setup Declarativo via YAML em Pastas (`teams/` & `agents/`)

Para desacoplar a definição dos agentes do código de infraestrutura, o sistema suporta **setup declarativo via arquivos YAML**:

### 2.1 Convenção de Nomes e Equipes
- Qualquer arquivo colocado na pasta `teams/<nome>.yaml` vira automaticamente uma **Equipe de Trabalho** cujo nome oficial é o próprio nome do arquivo:
  - `teams/forester.yaml` $\rightarrow$ Equipe **`forester`**.
  - `teams/sre_ops.yaml` $\rightarrow$ Equipe **`sre_ops`**.
  - `teams/qa_sentinel.yaml` $\rightarrow$ Equipe **`qa_sentinel`**.
- Arquivos na pasta `agents/<agente>.yaml` declaram instâncias de agentes ou sub-orquestradores que podem operar isoladamente ou ser importados por equipes:
  - `agents/agente01.yaml` $\rightarrow$ Agente **`agente01`** (ID: `agt_01h89x2k`).

### 2.2 Estrutura Canônica do YAML de Equipe (`forester.yaml`)

```yaml
version: "1.0"
team:
  name: "forester" # inferido a partir do arquivo 'forester.yaml'
  description: "Equipe de engenharia de software e segurança com Codex CLI, Claude Code e Agy Antigravity"
  
  finops:
    max_budget_usd: 2.00
    max_total_tokens: 300000
    expensive_model_approval_threshold_usd: 0.80
    expensive_model_action: "auto_approve" # Aprovação declarativa no YAML: modelos caros rodam sem pausa interativa

  # Política Declarativa de Aprovação e Rejeição (elimina validações manuais desnecessárias)
  approval_policy:
    mode: "automatic" # "automatic" | "hybrid" | "manual"
    auto_approve:
      - rule: "sandbox_tests_passed"
        condition: "outcome.exit_code == 0"
      - rule: "cost_within_budget"
        condition: "cumulative_cost_usd <= max_budget_usd"
      - rule: "allowed_cli_tools"
        tools: ["codex", "claude", "agy", "hermes", "pi"]
      - rule: "safe_inspection_commands"
        patterns: ["cargo test*", "cargo check*", "cargo clippy*", "git diff*", "git status*"]
    auto_reject:
      - rule: "destructive_host_commands"
        patterns: ["rm -rf /", "*--no-preserve-root*", "curl * | bash", "*id_rsa*"]
      - rule: "budget_hard_cap_exceeded"
        condition: "cumulative_cost_usd > max_budget_usd"
      - rule: "sandbox_timeout_violation"
        condition: "duration_seconds > timeout_seconds"
    fallback_action: "approve" # "approve" | "reject" | "escalate_to_human"

  sandbox_defaults:
    provider: "bwrap"
    filesystem:
      root_mode: "ro-bind"
      workspace_mode: "tmpfs"
    network: "isolated"
    timeout_seconds: 90

  # Orquestrador Líder Obrigatório
  orchestrator:
    id: "forester-lead"
    name: "Forester Lead Orchestrator"
    role: "Lead Architect & Security Officer"
    runner: "agy" # Antigravity CLI com planning mode
    cli_options:
      mode: "plan"
      effort: "high"
      output_format: "json"
    prompt:
      system: |
        Você é o líder da equipe Forester. Decomponha tarefas em DAG, delegue
        para Codex CLI, Claude Code e Agy Antigravity, supervisionando Sandbox e SQLite.
    plan:
      strategy: "PlanAndExecuteDAG"
      max_iterations: 10
      on_worker_failure: "HaltAndReport"
      default_pipeline:
        - step_id: "deep_research"
          delegate_to: "agy-worker"
          action: "agy.analyze_codebase"
        - step_id: "code_implementation"
          delegate_to: "codex-worker"
          action: "codex.exec"
          sandbox_action: "cargo test --all"
        - step_id: "code_review"
          delegate_to: "claude-code-worker"
          action: "claude.review"
        - step_id: "final_synthesis"
          action: "orchestrator.synthesize"

  # Time de operários ao redor do orquestrador (CLIs nativas)
  workers:
    - id: "codex-worker"
      name: "Codex CLI Worker"
      runner: "codex"
      cli_subcommand: "exec"
      allowed_tools: ["sandbox.run_command", "sandbox.read_file", "sandbox.write_file"]

    - id: "claude-code-worker"
      name: "Claude Code Sentinel"
      runner: "claude"
      cli_args: ["--print", "--output-format", "json", "--dangerously-skip-permissions"]
      allowed_tools: ["sandbox.read_file", "ast.diff_analyzer"]

    - id: "agy-worker"
      name: "Agy Antigravity Scout"
      runner: "agy"
      cli_args: ["--print", "--output-format", "json", "--effort", "medium", "--dangerously-skip-permissions"]
      allowed_tools: ["web.search", "docs.fetch", "mcp.inspect"]
```

### 2.3 Política Declarativa de Aprovação e Rejeição no YAML (Sem Validações Manuais Constantes)

Para que a equipe de IA opere com máxima autonomia sem exigir que o desenvolvedor fique continuamente respondendo a prompts de confirmação interativa (**Human-in-the-Loop**), as diretrizes de aprovação e rejeição são declaradas diretamente no próprio `.yaml`:

1. **Modo Autônomo (`mode: "automatic"`):**
   - O runtime avalia os critérios de forma determinística em sub-milissegundos.
   - Ações que correspondam às regras de `auto_approve` recebem `ApprovalGranted` automaticamente na trilha criptográfica.
   - Violações de `auto_reject` disparam `ApprovalRejected` imediato com aborto seguro ou feedback loop, sem intervenção humana.

2. **Modo Híbrido (`mode: "hybrid"`):**
   - Regras conhecidas de aprovação e rejeição são tratadas de forma autônoma; apenas casos desconhecidos ou comandos fora da lista recaem em `fallback_action: "escalate_to_human"`, solicitando intervenção manual.

3. **Autonomia FinOps (`expensive_model_action: "auto_approve"`):**
   - Quando ativado no bloco `finops:`, modelos com custo elevado (ex: Claude Opus, GPT-4) são automaticamente liberados para raciocínio crítico, desde que o teto financeiro global (`max_budget_usd`) não seja ultrapassado.

4. **Regras de Nó de Grafo (`human_gate` com `approval_rule`):**
   - Nós de validação como deploy ou merge declaram `approval_rule.auto_approve_when: "outcome.exit_code == 0"`, dispensando validação manual quando todos os testes na sandbox Bubblewrap passaram com sucesso.

---

## 3. Contratos de Dados em Rust (`orbity-core`)

### 3.1 Identidade e Ciclo de Vida do Agente

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String); // Ex: agt_01h89x2k

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentLifecycleState {
    Draft,        // Configuração criada ou carregada do YAML, aguardando validação
    Spawning,     // Alocando recursos, registrando no SQLite e preparando Sandbox
    Idle,         // Pronto para receber tarefas (Ready)
    Planning,     // Orquestrador interno gerando o DAG de execução
    Executing,    // Delegando e acompanhando trabalhadores
    Paused,       // Pausado (ex: aguardando aprovação de teto de custo FinOps)
    Completed,    // Orquestração concluída com síntese
    Failed,       // Erro irrecuperável ou falha crítica de sandbox
    Archived,     // Removido logicamente (soft-delete auditado)
}
```

### 3.2 O Agente e seu Orquestrador Interno

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: AgentId,
    pub name: String,
    pub team_name: Option<String>, // Ex: "forester"
    pub state: AgentLifecycleState,
    pub orchestrator: OrchestratorConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub id: String,
    pub role: String,
    pub runner: CliRunnerType,
    pub prompt: PromptConfig,
    pub plan: PlanConfig,
    pub team_nodes: Vec<GraphNode>,            // Nós da rede topológica
    pub topology: GraphTopology,               // Arestas e regras de travessia
    pub finops: BudgetPolicy,
    pub sandbox: SandboxPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub system: String,
    pub guidelines: Vec<String>,
    pub context_variables: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanConfig {
    pub strategy: PlanExecutionStrategy,
    pub max_iterations: usize,
    pub topology: GraphTopology,
    pub on_node_failure: FailurePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanExecutionStrategy {
    StaticGraph,        // Topologia fixa declarada no YAML
    DynamicTopcoatGraph, // Topcoat sintetiza o grafo em tempo de execução
    HybridGraph,        // Topologia base com expansões dinâmicas
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTopology {
    pub id: uuid::Uuid,
    pub entrypoint: String,
    pub terminal_nodes: Vec<String>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub id: String,
    pub name: String,
    pub role: String,
    pub runner: CliRunnerType,
    pub cli_command: Option<String>,
    pub cli_subcommand: Option<String>,
    pub cli_args: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub prompt: Option<PromptConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CliRunnerType {
    #[serde(rename = "codex")]
    CodexCli,
    #[serde(rename = "claude")]
    ClaudeCode,
    #[serde(rename = "agy")]
    AgyAntigravity,
    #[serde(rename = "hermes")]
    HermesAgent,
    #[serde(rename = "pi")]
    PiAssistant,
    #[serde(rename = "custom")]
    Custom(String),
}
```

### 3.3 Contrato de Execução da Suíte de 5 CLIs (`CliRunnerTrait`)

Cada CLI da suíte é invocada dentro da sandbox Bubblewrap com isolamento de sistema de arquivos e processos:

```rust
#[async_trait]
pub trait CliRunner: Send + Sync {
    /// Nome identificador da ferramenta (codex, claude, agy, hermes, pi)
    fn name(&self) -> &str;

    /// Constrói os argumentos de linha de comando para modo não-interativo estruturado
    fn build_args(&self, prompt: &str, config: &WorkerConfig) -> Vec<String>;

    /// Faz o parse do stdout da CLI (JSON / Stream JSON) e normaliza para TokenUsage
    fn parse_output(&self, stdout: &[u8]) -> Result<CliExecutionOutput, AgentError>;
}

// 1. Codex CLI: `codex exec [PROMPT]`
// Executa implementações de código e suítes de testes na sandbox.

// 2. Claude Code: `claude -p [PROMPT] --output-format json --dangerously-skip-permissions`
// Fornece raciocínio arquitetural e revisão de segurança emitindo JSON estruturado.

// 3. Agy Antigravity: `agy -p [PROMPT] --output-format json --effort medium|high --dangerously-skip-permissions`
// Realiza planejamento profundo, pesquisa de contexto e skills nativas do Google Antigravity.

// 4. Hermes Agent: `hermes run [PROMPT] --usage-file <path>`
// Focado em execução de ferramentas de busca, chamadas especializadas e métricas via --usage-file.

// 5. Pi Assistant: `pi -p [PROMPT] --mode json --no-session`
// Focado em edição ágil de código, transformações de arquivos e modo headless JSON.
```

### 3.4 Sistema de Preflight Check na Primeira Inicialização (`orbity init` / `orbity doctor`)

Quando a CLI `orbity` é executada pela primeira vez (ou através dos comandos `orbity init` / `orbity doctor`), ela realiza uma varredura automática no sistema para checar a disponibilidade e versões das 5 ferramentas fundamentais:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub all_passed: bool,
    pub bwrap_sandbox: ComponentStatus,
    pub sqlite_storage: ComponentStatus,
    pub cli_tools: Vec<CliToolCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliToolCheck {
    pub name: String,         // "codex", "claude", "agy", "hermes", "pi"
    pub binary_path: Option<String>,
    pub version: Option<String>,
    pub status: HealthStatus, // Available, Missing, Outdated, Error
    pub recommendation: Option<String>,
}
```

#### Exemplo Visual do Check na Inicialização da CLI:

```text
⚡ Orbity Multi Agentic Harness - Pre-flight Health Check
─────────────────────────────────────────────────────────────────────────────
[✓] Sandbox Runtime: Bubblewrap (/usr/bin/bwrap) 0.8.0
[✓] Storage Engine: SQLite 3.45.1 (WAL mode + SHA-256 Audit Store)

Suíte de Agentes CLI Detectados:
  [✓] codex   v0.154.0        -> /home/geanderson/.local/bin/codex
  [✓] claude  v2.1.239        -> /home/geanderson/.local/bin/claude
  [✓] agy     v1.2.11         -> /home/geanderson/.local/bin/agy
  [✓] hermes  v0.21.0         -> /home/geanderson/.local/bin/hermes
  [✓] pi      v0.78.1         -> /home/geanderson/.hermes/node/bin/pi

### 3.5 Motor de Reconciliação Declarativa Automática (`agents/` e `teams/`)

Cada vez que a CLI `orbity` é executada (seja para disparar uma tarefa, listar recursos ou verificar auditoria), o **Folder Scanner & Reconciler** faz uma varredura atômica em sub-milissegundos na pasta declarativa:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub scanned_files: usize,
    pub created: Vec<AgentSyncSummary>,
    pub updated: Vec<AgentSyncSummary>,
    pub archived: Vec<AgentSyncSummary>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSyncSummary {
    pub file_path: String,
    pub agent_id: AgentId,
    pub name: String,
    pub action: SyncActionType, // Created, Updated, Archived, Unchanged
    pub content_hash: String,
}

#[async_trait]
pub trait DeclarativeFolderReconciler: Send + Sync {
    /// Executa a varredura nas pastas `agents/` e `teams/`, compara os hashes SHA-256
    /// com a coluna `config_hash` do SQLite e sincroniza novidades ou updates.
    async fn reconcile_workspace(&self, base_dir: &std::path::Path) -> Result<ReconciliationReport, AgentLifecycleError>;
}
```

#### Como Funciona a Reconciliação por Hash (Zero Overhead):
1. **Leitura Rápida:** O scanner lê os arquivos `.yaml` em `agents/` e `teams/`.
2. **Comparação Criptográfica (SHA-256):**
   - Se o arquivo não existe no SQLite $\rightarrow$ **Novo Agente (`Created`)**: Valida schema, gera `AgentId`, insere na tabela `agents`/`teams` e grava evento `AgentCreatedFromYaml` com hash na trilha de auditoria.
   - Se o arquivo existe e o hash SHA-256 do conteúdo mudou $\rightarrow$ **Atualização (`Updated`)**: Executa `agent_update` no SQLite, atualiza prompt, plano e orçamentos, e grava evento `AgentUpdatedFromYaml` na trilha.
   - Se o arquivo foi excluído da pasta $\rightarrow$ **Arquivamento (`Archived`)**: Marca soft-delete auditado preservando todo o histórico de execuções anteriores.
   - Se o hash do arquivo for idêntico $\rightarrow$ **Inalterado (`Unchanged`)**: Nenhuma operação no banco é disparada (overhead inferior a 2 milissegundos).

#### Exemplo Visual no Terminal ao Rodar a CLI:
```text
🔄 Sincronizando definições declarativas (agents/ & teams/)...
  [+] Novo agente detectado: agents/agente02.yaml -> Registrado como 'Agente02' (ID: agt_02m84k1z)
  [~] Atualização detectada: agents/agente01.yaml -> Prompt e orçamento FinOps atualizados
✓ 1 criado, 1 atualizado em 3ms. Estado sincronizado com SQLite!
```

---

## 4. As Operações do Ciclo de Vida (CRUD de Agentes & Execução de Grafo)

O gerenciamento de instâncias e a execução topológica são regidos pela trait `AgentLifecycleManager`:

```rust
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCreateInput {
    pub name: String,
    pub team_name: Option<String>,
    pub orchestrator: OrchestratorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUpdateInput {
    pub name: Option<String>,
    pub prompt_system: Option<String>,
    pub plan: Option<PlanConfig>,
    pub topology: Option<GraphTopology>,
    pub team_nodes: Option<Vec<GraphNode>>,
    pub finops: Option<BudgetPolicy>,
}

#[derive(Debug, Clone, Default)]
pub struct AgentFilter {
    pub team_name: Option<String>,
    pub state: Option<AgentLifecycleState>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExecutionReport {
    pub execution_id: uuid::Uuid,
    pub agent_id: AgentId,
    pub status: GraphExecutionStatus,
    pub traversed_edges: usize,
    pub executed_nodes: Vec<NodeExecutionSummary>,
    pub cumulative_tokens: TokenUsage,
    pub cumulative_cost_usd: rust_decimal::Decimal,
    pub last_checkpoint_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphExecutionStatus {
    Traversing,
    BarrierWaiting,
    GateSuspended { prompt: String },
    FeedbackRetrying { iteration: u32, max_iterations: u32 },
    Completed,
    Failed { reason: String },
    BudgetTripped,
}

#[async_trait]
pub trait AgentLifecycleManager: Send + Sync {
    /// agentCreate: Cria um novo agente com seu orquestrador interno e valida a topologia de grafo (Kahn).
    /// Gera ID único, persiste no SQLite e emite evento 'AgentCreated' no barramento criptográfico.
    async fn agent_create(&self, input: AgentCreateInput) -> Result<AgentRecord, AgentLifecycleError>;

    /// agentList: Lista agentes ativos ou filtrados por equipe, status ou tags.
    async fn agent_list(&self, filter: AgentFilter) -> Result<Vec<AgentRecord>, AgentLifecycleError>;

    /// agentGet: Recupera um agente específico pelo seu AgentId.
    async fn agent_get(&self, id: &AgentId) -> Result<Option<AgentRecord>, AgentLifecycleError>;

    /// agentUpdate: Atualiza prompt, orçamento ou topologia de nós/arestas com hot-reload seguro.
    /// Emite evento 'AgentUpdated' na trilha de auditoria.
    async fn agent_update(&self, id: &AgentId, update: AgentUpdateInput) -> Result<AgentRecord, AgentLifecycleError>;

    /// agentDelete: Finaliza sandboxes associadas, aborta nós em voo e marca soft-delete auditado.
    /// Emite evento 'AgentDeleted' com hash encadeado no SQLite.
    async fn agent_delete(&self, id: &AgentId, force: bool) -> Result<(), AgentLifecycleError>;

    /// execute_graph: Dispara a execução assíncrona da topologia de grafo associada ao agente/equipe.
    async fn execute_graph(&self, id: &AgentId, initial_input: serde_json::Value) -> Result<GraphExecutionReport, AgentLifecycleError>;

    /// resume_graph: Retoma a execução a partir do checkpoint imutável mais recente gravado no SQLite.
    async fn resume_graph(&self, execution_id: uuid::Uuid, approved: bool) -> Result<GraphExecutionReport, AgentLifecycleError>;

    /// load_from_yaml: Faz parse de um arquivo YAML (ex: forester.yaml) e sincroniza a equipe e seu grafo.
    async fn load_from_yaml(&self, yaml_path: &std::path::Path) -> Result<AgentRecord, AgentLifecycleError>;
}
```

---

## 5. Máquina de Estados Unificada (Agente + Grafo de Execução)

A jornada multi-agente integra o ciclo de vida da instância com a máquina de estados reativa da execução do grafo:

```
                      ┌────────────────┐
                      │  Arquivo YAML  │ (ex: forester.yaml)
                      └───────┬────────┘
                              │ load_from_yaml / agent_create (Validação Kahn DAG)
                              ▼
                      ┌────────────────┐
                      │     DRAFT      │
                      └───────┬────────┘
                              │ spawn
                              ▼
                      ┌────────────────┐
                      │    SPAWNING    │ (Preparação da Sandbox e SQLite)
                      └───────┬────────┘
                              │ pronto
                              ▼
                      ┌────────────────┐
        ┌───────────> │      IDLE      │ <──────────────────────────────┐
        │             └───────┬────────┘                                │
        │                     │ execute_graph / job_assigned            │
        │                     ▼                                         │
        │             ┌────────────────┐                                │
        │             │    PLANNING    │ (Topcoat expande nós e arestas)│
        │             └───────┬────────┘                                │
        │                     │ Grafo pronto                            │
        │                     ▼                                         │
        │             ┌──────────────────────────────────────────────┐  │
        │             │             EXECUTING (Grafo Ativo)          │  │
        │             │                                              │  │
        │             │   ┌──────────────────────────────────────┐   │  │
        │             │   │ ● NodeTraversing (Fan-out Tokio)     │   │  │
        │             │   │ ● BarrierWaiting (Aguardando Fan-in) │   │  │
        │             │   │ ● ConditionalRouting (Predicados)    │   │  │
        │             │   │ ● FeedbackRetrying (Retorno/Correção)│   │  │
        │             │   │ ● Checkpointing (Hash SHA-256 SQLite)│   │  │
        │             │   └──────────────────┬───────────────────┘   │  │
        │             └──────────────────────┼───────────────────────┘  │
        │                                    │                          │
        │                          HITL Gate │ (Approval Required)      │
        │                                    ▼                          │
        │                             ┌──────────────┐                  │
        │                             │    PAUSED    │                  │
        │                             │(Human Gate)  │                  │
        │                             └──────┬───────┘                  │
        │                                    │ orbity resume            │
        │                                    │ (--approve / --reject)   │
        │                                    ▼                          │
        │                        Retoma do Checkpoint                   │
        │                                                               │
        │ Sucesso (Terminais alcançados)                                │
        └───────────────────────────────────────────────────────────────┘
                                             │
                                             ▼ Falha Crítica / Tripwire FinOps
                                      ┌──────────────┐
                                      │    FAILED    │
                                      └──────┬───────┘
                                             │ agent_delete
                                             ▼
                                      ┌──────────────┐
                                      │   ARCHIVED   │
                                      └──────────────┘
```

---

## 6. Persistência Relacional em SQLite (`orbity-storage`)

Para sustentar a arquitetura de **Graph Engineering** e auditoria imutável, o banco SQLite armazena o estado relacional, as topologias de grafos e a trilha de checkpoints criptográficos:

```sql
-- 1. Tabela de Equipes (inferidas de forester.yaml, etc)
CREATE TABLE IF NOT EXISTS teams (
    name TEXT PRIMARY KEY,
    description TEXT,
    config_yaml TEXT NOT NULL,
    config_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. Tabela de Agentes (com Orquestrador)
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    team_name TEXT REFERENCES teams(name) ON DELETE SET NULL,
    state TEXT NOT NULL CHECK(state IN ('Draft', 'Spawning', 'Idle', 'Planning', 'Executing', 'Paused', 'Completed', 'Failed', 'Archived')),
    orchestrator_runner TEXT NOT NULL,
    prompt_system TEXT NOT NULL,
    plan_strategy TEXT NOT NULL,
    topology_id TEXT REFERENCES graph_topologies(id),
    finops_budget_usd REAL NOT NULL DEFAULT 1.00,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. Tabela de Topologias de Grafo (Nodes & Edges Declarativos)
CREATE TABLE IF NOT EXISTS graph_topologies (
    id TEXT PRIMARY KEY,               -- UUID da Topologia
    team_name TEXT NOT NULL,
    definition_json TEXT NOT NULL,     -- Nodes & Edges completos
    config_hash TEXT NOT NULL,         -- SHA-256 do arquivo YAML
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 4. Tabela de Execuções de Grafo (State Machine Ativa)
CREATE TABLE IF NOT EXISTS graph_executions (
    id TEXT PRIMARY KEY,               -- UUID da Execução (Run ID)
    topology_id TEXT NOT NULL REFERENCES graph_topologies(id),
    agent_id TEXT NOT NULL REFERENCES agents(id),
    status TEXT NOT NULL,              -- Traversing, BarrierWaiting, GateSuspended, Completed, Failed
    active_nodes_json TEXT NOT NULL,   -- Conjunto de nós executando concorrentemente
    blackboard_state TEXT NOT NULL,    -- Memória compartilhada / artefatos acumulados
    cumulative_cost_usd REAL NOT NULL DEFAULT 0.0,
    cumulative_tokens INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 5. Tabela de Checkpoints Criptográficos de Grafo (Append-Only SHA-256)
CREATE TABLE IF NOT EXISTS graph_checkpoints (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL REFERENCES graph_executions(id),
    step_sequence INTEGER NOT NULL,
    active_nodes_json TEXT NOT NULL,
    completed_nodes_json TEXT NOT NULL,
    blackboard_snapshot TEXT NOT NULL,
    parent_hash TEXT NOT NULL,         -- SHA-256 do checkpoint anterior
    checkpoint_hash TEXT NOT NULL,     -- SHA-256 (parent_hash + step + blackboard)
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 6. Tabela de Execuções Individuais por Nó na Sandbox
CREATE TABLE IF NOT EXISTS graph_node_executions (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL REFERENCES graph_executions(id),
    node_id TEXT NOT NULL,
    cli_runner TEXT NOT NULL,          -- codex, claude, agy, hermes, pi, tool
    sandbox_id TEXT NOT NULL,
    exit_code INTEGER,
    duration_ms INTEGER,
    tokens_input INTEGER,
    tokens_output INTEGER,
    cost_usd REAL,
    error_message TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

---

## 7. Experiência de Linha de Comando (CLI `orbity`)

A CLI oferece controle completo tanto imperativo quanto declarativo baseado na pasta de YAMLs:

```bash
# ==============================================================================
# 1. SETUP DECLARATIVO DE EQUIPES E AGENTES (YAML)
# ==============================================================================

# Carregar equipe forester a partir de seu arquivo YAML
orbity team load ./examples/forester/teams/forester.yaml
# Saída:
# ✓ Equipe 'forester' registrada com sucesso!
#   └─ Orquestrador Líder: Forester Lead Orchestrator (ID: forester-lead)
#   └─ Workers Registrados: 3 (codex-worker, claude-auditor, hermes-researcher)
#   └─ Orçamento Teto: $1.50 USD

# Listar todas as equipes carregadas
orbity team list

# Disparar uma tarefa direcionada para a equipe 'forester'
orbity team run forester "Auditar e refatorar conexões de banco SQLite em orbity-storage"

# ==============================================================================
# 2. OPERAÇÕES CRUD DIRETAS DE AGENTES
# ==============================================================================

# agentCreate: Criar novo agente via YAML individual ou argumentos
orbity agent create -f ./examples/forester/agents/agente01.yaml
orbity agent create --name "BugHunter" --model "claude-3-5-sonnet" --plan "AutonomousLoop"

# agentList: Listar agentes com seus orquestradores e estados atuais
orbity agent list
orbity agent list --team forester --state Idle

# agentGet: Inspecionar detalhes de um agente (prompt, plano e workers)
orbity agent get agt_01h89x2k

# agentUpdate: Atualizar prompt ou orçamento em tempo real
orbity agent update agt_01h89x2k --budget-usd 2.00
orbity agent update agt_01h89x2k -f ./examples/forester/agents/agente01_v2.yaml

# agentDelete: Desativar agente de forma auditada
orbity agent delete agt_01h89x2k

# ==============================================================================
# 3. COMANDOS DE GRAPH ENGINEERING (TOPOLOGIA & RETOMADA)
# ==============================================================================

# Validar topologia e integridade de arestas de uma equipe (Kahn DAG + Cycle check)
orbity graph validate ./examples/forester/teams/forester.yaml

# Inspecionar nós, arestas, barreiras e pontos de aprovação humana no terminal
orbity graph inspect forester

# Executar a topologia completa de grafo para um objetivo
orbity graph run forester "Auditar e refatorar conexões de banco SQLite em orbity-storage"

# Inspecionar checkpoints imutáveis gravados no SQLite durante a travessia
orbity graph checkpoints <EXECUTION_ID>

# Retomar grafo pausado em nó HumanGate (HITL) ou interrompido
orbity graph resume <EXECUTION_ID> --approve
```

---

## 8. Resumo dos Benefícios do Design

1. **Separação de Preocupações:** O desenvolvedor ou DevOps declara as equipes e papéis em arquivos YAML claros (`forester.yaml`), enquanto o runtime Rust garante tipagem estrita, performance assíncrona e segurança de sandbox.
2. **Hierarquia Clara:** Fim da ambiguidade. Cada agente tem **seu próprio orquestrador**, com um **plano explícito**, um **prompt** e um **time de workers sob seu comando direto**.
3. **Auditabilidade Integral:** Todas as operações (`create`, `update`, `delete`, `delegate`) geram eventos gravados na trilha com hash encadeado SHA-256 no SQLite.

---

## 9. Contratos de Graph Engineering (`orbity-graph`)

O time de agentes e suas ferramentas operam formalmente como um **Grafo Computacional Dirigido**. Cada nó executa uma unidade atômica de trabalho e cada aresta dita como o estado flui e como decisões e feedbacks são processados.

```rust
use std::collections::{HashMap, HashSet};
use async_trait::async_trait;
use uuid::Uuid;
use rust_decimal::Decimal;

// Identificadores fortamente tipados
pub type NodeId = String;
pub type GraphId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeKind {
    Supervisor { engine: String },
    Agent { cli: CliType, worker_ref: String },
    Tool { command: String, timeout_seconds: u64 },
    ConditionalRouter { predicate_expr: String },
    HumanGate { prompt: String, timeout_seconds: Option<u64> },
    JoinBarrier { quorum: Option<usize> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeKind {
    Direct,                                    // Execução sequencial direta
    ParallelFanOut,                           // Disparo concorrente em sandboxes isoladas
    BarrierFanIn,                             // Ponto de encontro / barreira de sincronização
    Conditional { predicate: String },         // Roteamento baseado em resultado/status
    FeedbackLoop { max_iterations: u32 },     // Retorno para correção com teto estrito
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub retries_limit: u32,
    pub budget_limit_usd: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
    pub inject_context: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTopology {
    pub id: GraphId,
    pub team_name: String,
    pub entrypoint: NodeId,
    pub terminal_nodes: HashSet<NodeId>,
    pub nodes: HashMap<NodeId, GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphCheckpoint {
    pub execution_id: Uuid,
    pub step_sequence: u64,
    pub current_nodes: HashSet<NodeId>,
    pub completed_nodes: HashSet<NodeId>,
    pub blackboard_state: serde_json::Value,
    pub cumulative_cost_usd: Decimal,
    pub parent_hash: [u8; 32],
    pub checkpoint_hash: [u8; 32],
}

#[async_trait]
pub trait GraphEngine: Send + Sync {
    /// Valida a integridade topológica do grafo (detecção de ciclos não-controlados e nós órfãos)
    fn validate_topology(&self, topology: &GraphTopology) -> Result<(), GraphValidationError>;

    /// Calcula a ordenação topológica para nós independentes (Algoritmo de Kahn)
    fn compute_execution_order(&self, topology: &GraphTopology) -> Result<Vec<NodeId>, GraphValidationError>;

    /// Executa o grafo gerenciando paralelismo, barreiras, condicionais e feedbacks
    async fn execute_graph(
        &self,
        topology: &GraphTopology,
        initial_context: serde_json::Value,
    ) -> Result<GraphExecutionSummary, GraphExecutionError>;

    /// Retoma a execução de um grafo a partir do último checkpoint criptográfico do SQLite
    async fn resume_from_checkpoint(
        &self,
        checkpoint_id: Uuid,
    ) -> Result<GraphExecutionSummary, GraphExecutionError>;
}

/// Eventos Canônicos emitidos pelo motor de Graph Engineering para o Barramento e SQLite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphRuntimeEvent {
    GraphExecutionStarted {
        graph_id: GraphId,
        entrypoint: NodeId,
        team_name: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    NodeDispatched {
        graph_id: GraphId,
        node_id: NodeId,
        kind: NodeKind,
        sandbox_id: String,
    },
    NodeCompleted {
        graph_id: GraphId,
        node_id: NodeId,
        duration_ms: u64,
        tokens: TokenUsage,
        cost_usd: Decimal,
    },
    NodeFailed {
        graph_id: GraphId,
        node_id: NodeId,
        error: String,
        retries_remaining: u32,
    },
    EdgeTraversed {
        graph_id: GraphId,
        from_node: NodeId,
        to_node: NodeId,
        kind: EdgeKind,
        condition_evaluated: Option<bool>,
    },
    BarrierAwaiting {
        graph_id: GraphId,
        barrier_node: NodeId,
        expected_nodes: Vec<NodeId>,
        completed_nodes: Vec<NodeId>,
    },
    FeedbackLoopTriggered {
        graph_id: GraphId,
        from_node: NodeId,
        to_node: NodeId,
        iteration: u32,
        max_iterations: u32,
        injected_context_keys: Vec<String>,
    },
    GraphCheckpointSaved {
        graph_id: GraphId,
        step_sequence: u64,
        active_nodes: Vec<NodeId>,
        state_hash: [u8; 32],
        parent_hash: [u8; 32],
    },
    GraphCompleted {
        graph_id: GraphId,
        total_traversed_edges: usize,
        total_tokens: TokenUsage,
        total_cost_usd: Decimal,
        status: GraphExecutionStatus,
    },
}
```

