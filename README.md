<div align="center">
  <img src="assets/orbity-logo.jpg" alt="Orbity Logo" width="220" style="border-radius: 50%; box-shadow: 0 8px 30px rgba(0,255,163,0.2);" />
  <h1>🪐 ORBITY</h1>
  <p><strong>Orquestrador Autônomo Multi-Agente de Alta Performance em Rust</strong></p>

  [![Made in Brazil](https://img.shields.io/badge/Made%20in-Brasil%20%F0%9F%87%A7%F0%9F%87%B7-009c3b?style=for-the-badge&logoColor=white)](https://github.com)
  [![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
  [![SQLite](https://img.shields.io/badge/SQLite-WAL_%2B_SHA--256-blue?style=for-the-badge&logo=sqlite)](https://sqlite.org)
  [![Sandbox](https://img.shields.io/badge/Sandbox-Bubblewrap-cyan?style=for-the-badge)](https://github.com/containers/bubblewrap)
  [![GitHub Pages](https://img.shields.io/badge/GitHub_Pages-Online-indigo?style=for-the-badge&logo=github)](docs/index.html)
</div>

<br/>

**Orbity** é um runtime e orquestrador corporativo de agentes autônomos de inteligência artificial construído em **Rust**, focado em **segurança de sandbox estrita**, **governança com trilha de auditoria append-only imutável em SQLite (com encadeamento de hash SHA-256)**, **observabilidade em 4 camadas** e **FinOps em tempo real** orquestrado nativamente pelo motor reativo **Tokio Topcoat**. Orgulhosamente desenvolvido e projetado no Brasil 🇧🇷.

🌐 **GitHub Pages do Projeto:** Disponível na pasta [`docs/`](docs/index.html) com visualizador interativo em tempo real, simulador de grafos, matriz de Gates & Tasks e console de auditoria.

---

## 🏛️ Os 6 Pilares do Sistema

1. **🦀 Rust & Concorrência Tokio:** Desempenho nativo com barramento assíncrono de eventos de alta vazão (>5.000 ev/s), gerenciamento de memória seguro e ausência de locks globais.
2. **🕸️ Graph Engineering (`Nodes` & `Edges`):** Modelagem rigorosa de times e fluxos como grafos computacionais dirigidos, com ordenação topológica (Kahn), fan-out paralelo, fan-in de barreira e loops de feedback controlados.
3. **🧠 Orquestrador Tokio Topcoat & Motor de Grafos:** Arquitetura reativa que executa grafos dirigidos (DAGs), delega subtarefas especializadas para a suíte de 5 CLIs (`Codex`, `Claude`, `Agy`, `Hermes`, `Pi`) e gerencia a memória Blackboard sem dependência de supervisor Astra.
4. **🔗 SQLite Audit Store com Hashes Encadeados:** Trilha de auditoria append-only estilo blockchain-lite em modo WAL. Cada evento incorpora o hash SHA-256 do evento anterior, permitindo detecção matemática de qualquer adulteração nos registros históricos.
5. **🛡️ Sandbox Confinada (bwrap / namespaces):** Isolamento total da execução de ferramentas e comandos externos: root filesystem em modo somente leitura (`ro-bind`), diretório de trabalho efêmero em `tmpfs`, isolamento de rede configurável e limpeza de credenciais sensíveis.
6. **💰 FinOps & 4 Camadas de Logs:** Separação estrita entre Runtime, Execution, Audit e Telemetry logs (OpenTelemetry exportável para Grafana/Loki), com monitoramento de tokens e tripwires orçamentários por nó e por aresta.

---

## 📦 Arquitetura Modular de Crates (`orbity-*`)

O workspace Cargo é particionado em crates desacopladas:

| Crate | Responsabilidade Principal |
|---|---|
| **`crates/orbity-core`** | Tipos fundamentais, enums de eventos (`RuntimeEvent`), `TokenUsage`, políticas e sanitização de segredos. |
| **`crates/orbity-storage`** | Persistência SQLite WAL, migrações e ledger append-only com encadeamento criptográfico SHA-256. |
| **`crates/orbity-sandbox`** | Isolamento de comandos e ferramentas via Linux namespaces / Bubblewrap (`bwrap`), tmpfs e cgroups. |
| **`crates/orbity-graph`** | Motor de Graph Engineering, tipos `GraphNode`, `GraphEdge`, ordenação topológica, DAG e checkpoints. |
| **`crates/orbity-agent`** | Orquestração de tarefas, suíte de 5 CLIs (`codex`, `claude`, `agy`, `hermes`, `pi`) e ciclo de vida de agentes. |
| **`crates/orbity-telemetry`** | Barramento de observabilidade com Rust `tracing` e exportador OpenTelemetry para Grafana/Loki. |
| **`crates/orbity-server`** | Servidor de aplicação full-stack Tokio Topcoat (v0.9+) com views reativas, shards com morphing DOM e WebSockets server-push. |
| **`crates/orbity-cli`** | Binário de linha de comando com auto-sync de YAMLs, modo TUI (`ratatui`) e preflight health check. |

---

## 🚀 Como Ativar o GitHub Pages no Repositório

O projeto já inclui a página interativa e o workflow automatizado do GitHub Actions:

### Opção 1: Via GitHub Actions (Recomendado)
1. No seu repositório no GitHub, acesse **Settings** > **Pages**.
2. Em **Build and deployment** > **Source**, selecione: **GitHub Actions**.
3. O workflow [`.github/workflows/deploy-pages.yml`](.github/workflows/deploy-pages.yml) fará o deploy automático a cada commit na branch `main` ou `master`.

### Opção 2: Via Branch / Pasta docs
1. No seu repositório no GitHub, acesse **Settings** > **Pages**.
2. Em **Build and deployment** > **Source**, selecione **Deploy from a branch**.
3. Em **Branch**, selecione a branch principal (`main` ou `master`) e escolha a pasta `/docs`.
4. Clique em **Save**. Seu site estará no ar em poucos segundos na URL:  
   `https://<seu-usuario>.github.io/<nome-do-repositorio>/`

---

## 📋 Plano de Implementação (Gates & Tasks)

O projeto segue a metodologia de **Quality Gates** estruturada em 8 fases:

- **[Gate 0: Fundação do Workspace, Tipos & Domínio de Eventos](IMPLEMENTATION_PLAN.md#gate-0-fundação-do-workspace-tipos--domínio-de-eventos)** `[CONCLUÍDO ✅]`
- **[Gate 1: Persistência SQLite & Audit Store Criptográfico](IMPLEMENTATION_PLAN.md#gate-1-persistência-sqlite--audit-store-criptográfico)** `[CONCLUÍDO ✅]`
- **[Gate 2: Mecanismo de Sandbox & Isolamento de Processos](IMPLEMENTATION_PLAN.md#gate-2-mecanismo-de-sandbox--isolamento-de-processos)** `[CONCLUÍDO ✅]`
- **[Gate 3: Barramento Unificado de Eventos & Observabilidade em 4 Camadas](IMPLEMENTATION_PLAN.md#gate-3-barramento-unificado-de-eventos--observabilidade-em-4-camadas)** `[CONCLUÍDO ✅]`
- **[Gate 4: Graph Engineering & Orquestração Multi-Agente Tokio (Sem Astra)](IMPLEMENTATION_PLAN.md#gate-4-graph-engineering-orbity-graph--orquestração-multi-agente-em-tokio-sem-astra)** `[CONCLUÍDO ✅]`
- **[Gate 5: Aplicação Servidora Reativa com Tokio Topcoat](IMPLEMENTATION_PLAN.md#gate-5-aplicação-servidora-reativa-com-tokio-topcoat)** `[CONCLUÍDO ✅]`
- **[Gate 6: Interface CLI de Orquestração & Modo Terminal TUI](IMPLEMENTATION_PLAN.md#gate-6-interface-cli-de-orquestração--modo-terminal-tui)** `[CONCLUÍDO ✅]`
- **[Gate 7: Testes E2E, Validação de Segurança & Hardening](IMPLEMENTATION_PLAN.md#gate-7-testes-e2e-validação-de-segurança--hardening)** `[CONCLUÍDO ✅]`

---

## 🛡️ Evidências de Implementação dos Gates 0 ao 7

Todos os 8 Quality Gates (0 ao 7) foram implementados em Rust nativo e validados com 52 testes unitários e de integração, incluindo cenário real multi-agente, injeção de adulteração de auditoria, confinamento de sandbox com Bubblewrap, pipeline simultâneo de 4 camadas de observabilidade, motor de grafos Tokio DAG sem supervisor Astra, servidor reativo Tokio Topcoat e preflight health check da CLI:

### Tabela de Rastreabilidade de Commits

| Commit SHA | Tipo / Escopo | Task | Descrição da Implementação |
|---|---|---|---|
| `e5a2c47` | `feat(gate-0)` | **TASK-001** | Estruturação do Cargo Workspace Multi-Crate com 8 crates desacopladas. |
| `7f9489b` | `feat(gate-0)` | **TASK-002** | Modelagem canônica do enum `RuntimeEvent` (Lifecycle, Execution, Security, Sandbox, FinOps). |
| `c6c565b` | `feat(gate-0)` | **TASK-003** | Modelagem de domínio FinOps (`TokenUsage`, `BudgetPolicy`, tripwires e Human-in-the-Loop). |
| `8b48df6` | `feat(gate-0)` | **TASK-004** | Mecanismo de sanitização `SecretMasker` e metadados de credenciais (`SecretMetadata`). |
| `fa45573` | `feat(gate-0)` | **TASK-005** | Parsers YAML declarativos para equipes (`teams/forester.yaml`) e agentes (`agents/agente01.yaml`). |
| `3c0ba62` | `feat(gate-1)` | **TASK-101** | Setup SQLite com modo WAL, pragmas de performance e migrations automatizadas com 8 tabelas e índices. |
| `7a7d177` | `feat(gate-1)` | **TASK-102** | Motor de auditoria append-only `AuditStore` com encadeamento de hash SHA-256 e bloco Genesis. |
| `ca9b8bd` | `feat(gate-1)` | **TASK-103** | Verificador de integridade histórica `AuditVerifier` com validação de cadeia contínua e imutabilidade. |
| `24aa629` | `feat(gate-1)` | **TASK-104** | DAOs assíncronos: `RunDao`, `TaskDao`, `TokenLedgerDao`, `TeamDao` e `AgentDao`. |
| `66398b7` | `fix(storage)` | **FIX** | Loop atômico de retry com backoff contra contenção concorrente de múltiplos agentes em `append_event`. |
| `a39c094` | `test(gate-1)` | **E2E TEST** | Teste em cenário real multi-agente (Astra + Codex + Claude + Hermes + Pi) com verificação e detecção de tampering. |
| `33d7898` | `docs` | **PLAN** | Atualização do `IMPLEMENTATION_PLAN.md` marcando todas as tarefas de Gate 0 e Gate 1 concluídas. |
| `bf61386` | `docs` | **SPECS** | Especificação técnica de políticas de rede e ciclo de vida de filesystem no Gate 2. |
| `d3073da` | `feat(gate-2)` | **TASK-201** | Abstrações de Sandbox: traits `Sandbox`, modos de rede (`Isolated`, `EgressAllowlist`, `HostMediated`), ciclo de vida de workspace efêmero vs permanente, e `MockSandbox`. |
| `a0fac9b` | `feat(gate-2)` | **TASK-202** | Implementação nativa Linux com Bubblewrap (`BwrapSandbox`): root somente leitura (`--ro-bind / /`), tmpfs seguro em `/tmp/workspace`, `--unshare-net`, defesa de traversal, snapshot, rollback e promoção de arquivos. |
| `cb9f139` | `feat(gate-2)` | **TASK-203** | Controle estrito de recursos e timeouts de execução com terminação forçada `SIGKILL` e limpeza garantida da árvore de processos. |
| `44b2bfd` | `feat(gate-2)` | **TASK-204** | Emissão de eventos estruturados de ciclo de vida e políticas (`InstrumentedSandbox` / `SandboxEventEmitter`) com `PolicyDenied`, `CommandExecuted`, `FileWritten` e `FileRead`. |
| `2524878` | `test(gate-2)` | **E2E TEST** | Teste integrado de confinamento, isolamento de rede, timeouts com SIGKILL, rollback atômico, promoção seletiva e trilha de auditoria SQLite encadeada. |
| `e67bc00` | `docs` | **DOCS** | Atualização da documentação geral com evidências de conclusão do Gate 2. |
| `172c0d7` | `feat(gate-3)` | **TASK-301** | Barramento assíncrono de eventos (`EventBus`) com canal de broadcast, fila de alta capacidade MPSC, trait `EventSink` e múltiplos destinos. |
| `10619a5` | `feat(gate-3)` | **TASK-302** | Camada 1: Runtime Logs (`RuntimeLogSink`) para ciclo de vida de agentes, execuções e eventos HITL com formatação canônica. |
| `25913ca` | `feat(gate-3)` | **TASK-303** | Camada 2: Execution Logs (`ExecutionLogSink`) para captura de comandos, ferramentas e metadados de arquivos com hash SHA-256. |
| `826c7f0` | `feat(gate-3)` | **TASK-304** | Camada 3: Audit Logs (`AuditLogSink`) persistindo eventos no SQLite WAL com encadeamento de hash SHA-256. |
| `6b222cc` | `feat(gate-3)` | **TASK-305** | Camada 4: Telemetry Logs (`TelemetrySink`, `SpanTree`, `TelemetryMetrics`) com spans hierárquicos e exportador OpenTelemetry/OTLP JSON. |
| `f0e8536` | `test(gate-3)` | **E2E TEST** | Teste integrado de pipeline de 4 camadas simultâneas e teste de carga atingindo vazão de 75.473 ev/s (>15x o teto de 5.000 ev/s). |
| `f308ae1` | `docs(gate-3)` | **DOCS** | Atualização da documentação geral (`IMPLEMENTATION_PLAN.md`, `README.md`, `JOURNEY_MAP_AND_AUDIT.md`) com conclusão do Gate 3. |
| `1e18a97` | `docs(gate-5/6)` | **DOCS** | Atualização do Gate 5 para aplicação servidora reativa Tokio Topcoat (`topcoat` v0.9+) e reorganização da CLI para o Gate 6. |
| `6d2cc49` | `feat(contracts)` | **TASK-005/TASK-409** | Políticas declarativas de aprovação e rejeição no YAML (`ApprovalPolicy`, `expensive_model_action`, `auto_approve`, `auto_reject`) eliminando validações manuais desnecessárias. |
| `ca81afd` | `refactor` | **REFACTOR** | Revisão de arquitetura dos Gates 4 e 5 para Tokio Topcoat e remoção completa do supervisor Astra. |
| `7c15fea` | `feat(gate-4)` | **TASK-401 a 410** | Motor de grafos em Tokio Topcoat (`GraphEngine`, `GraphExecutor`), ordenação topológica de Kahn, fan-out/fan-in, feedback loops com teto de iterações, injeção de contexto Blackboard, checkpoints criptográficos SQLite, 5 adaptadores de CLIs (`codex`, `claude`, `agy`, `hermes`, `pi`), tripwires FinOps e orquestrador topológico sem supervisor Astra. |
| `0afb45c` | `feat(gate-5)` | **TASK-501 a 505** | Servidor de aplicação full-stack Tokio Topcoat (`orbity-server`), contexto da aplicação `Cx`, views reativas (`view!`), sinais de cliente (`signal`), componentes shard (`#[shard]`), streaming SSR com macros `live!` e `emit!`, WebSockets server-push (`ServerPushManager`) e console HITL de governança e FinOps. |
| `8d35460` | `feat(gate-6)` | **TASK-601 a 606** | Engine da CLI `orbity` com derivação de comandos `clap` v4 (`run`, `resume`, `status`, `logs`, `audit verify`, `finops`, `serve`, `doctor`, `agent`, `team`), preflight health check das 5 CLIs e reconciliação automática declarativa de pastas YAML com cálculo de hash SHA-256 e SQLite WAL. |
| `10e4df4` | `feat(gate-7)` | **TASK-701 a 705** | Testes de integração E2E e hardening: detecção de adulteração em trilha de auditoria, confinamento estrito de sandbox e bloqueio de path traversal, tripwire de orçamento FinOps e teste de stress concorrente com 4 instâncias paralelas do motor de grafos. |

### Resultados dos Testes de Concorrência, Confinamento e Observabilidade
- **Suíte de Testes:** 52 testes executados e aprovados via `cargo test --workspace` (100% sucesso).
- **Linter & Compilação:** 0 warnings em `cargo clippy --workspace --all-targets -- -D warnings`.
- **Vazão do Barramento (Load Test):** 🚀 **75.473 eventos/segundo** (excede o requisito mínimo de 5.000 ev/s em mais de 15x).
- **Cenário Multi-Agente Concorrente (`crates/orbity-storage/tests/real_multi_agent_scenario.rs`):**
  - **Orquestrador Tokio Topcoat:** Coordenação e agregação contábil FinOps.
  - **4 Agentes Concorrentes (Tokio):** `Codex Dev`, `Claude Sentinel`, `Hermes Researcher` e `Pi Assistant`.
  - **12 Eventos Criptográficos:** Encadeados com sucesso no SQLite WAL; hash final SHA-256 verificado.
  - **Injeção de Violação:** Alteração deliberada de 1 byte no SQLite detectada com 100% de precisão pelo `AuditVerifier` acusando `AuditVerificationResult::Tampered` no índice exato da violação.
- **Cenário de Confinamento e Sandbox (`crates/orbity-sandbox/tests/sandbox_isolation_and_rollback.rs`):**
  - **Proteção do Host Root:** Bloqueio comprovado de escrita fora do workspace temporário (`/etc`, `/bin`, `/var`).
  - **Isolamento de Rede:** Bloqueio total de tráfego de rede no modo `Isolated` (`--unshare-net`).
  - **Prevenção de Path Traversal:** Rejeição de caminhos com `../` ou rotas de escape para diretórios protegidos.
  - **Timeouts Rígidos com SIGKILL:** Comandos em loop infinito terminados forçadamente sem vazamento de processos.
  - **Rollback Atômico vs Promoção:** Descarte instantâneo de alterações não aprovadas; sincronização limpa com cálculo de hash SHA-256 no diretório permanente mediante aprovação.
- **Pipeline de 4 Camadas Simultâneas (`crates/orbity-telemetry/tests/four_layer_observability_and_load.rs`):**
  - **Broadcast Streaming:** 10 de 10 eventos recebidos em tempo real para WebSockets/SSE.
  - **Camada 1 (Runtime):** 6 eventos de ciclo de vida rastreados e formatados.
  - **Camada 2 (Execution):** Comandos e arquivos alterados capturados com medição de latência.
  - **Camada 3 (Audit):** 10 eventos encadeados no SQLite com validação criptográfica SHA-256.
  - **Camada 4 (Telemetry):** Spans hierárquicos (run -> supervisor -> worker) e métricas consolidadas exportadas em formato OTLP JSON.
  - **Arquivo Local (.jsonl):** 10 linhas registradas em disco sem corrupção.


---

## 💻 CLI Quickstart (`orbity`)

```bash
# ==============================================================================
# 0. PREFLIGHT HEALTH CHECK (Automático na 1ª execução ou via 'orbity doctor')
# ==============================================================================

# Checagem das 5 CLIs (codex, claude, agy, hermes, pi) + Sandbox Bubblewrap:
orbity doctor
# Saída:
# ⚡ Orbity Agentic Platform - Pre-flight Health Check
# [✓] Sandbox Runtime: Bubblewrap (/usr/bin/bwrap)
# [✓] Storage Engine: SQLite 3.45.1 (WAL + SHA-256 Audit Store)
# Suíte de Agentes CLI:
#   [✓] codex   v0.154.0        -> /home/geanderson/.local/bin/codex
#   [✓] claude  v2.1.239        -> /home/geanderson/.local/bin/claude
#   [✓] agy     v1.2.11         -> /home/geanderson/.local/bin/agy
#   [✓] hermes  v0.21.0         -> /home/geanderson/.local/bin/hermes
#   [✓] pi      v0.78.1         -> /home/geanderson/.hermes/node/bin/pi
# ✓ 5 de 5 CLIs operacionais. Ambiente pronto para orquestração!

# ==============================================================================
# 1. SETUP DECLARATIVO DE EQUIPES (YAML)
# ==============================================================================

# Carregar equipe forester a partir de seu arquivo YAML (ex: forester.yaml vira equipe 'forester')
orbity team load ./examples/teams/forester.yaml

# Executar tarefa com a equipe 'forester' via motor de grafos Topcoat
orbity team run forester "Auditar e refatorar conexões de banco SQLite em orbity-storage"

# ==============================================================================
# 2. OPERAÇÕES CRUD DE AGENTES
# ==============================================================================

# Criar agente com orquestrador interno e time associado
orbity agent create -f ./examples/agents/agente01.yaml

# Listar agentes ativos e inspecionar status
orbity agent list --team forester

# Atualizar orquestrador em tempo real
orbity agent update agt_01h89x2k --budget-usd 2.00

# ==============================================================================
# 3. AUDITORIA CRIPTOGRÁFICA & FINOPS
# ==============================================================================

# Validação criptográfica da integridade da trilha de auditoria SQLite
orbity audit verify run_74f9c

# Relatório consolidado de FinOps (tokens e custo por agente)
orbity finops summary --since 2026-09-01

# ==============================================================================
# 4. SERVIDOR REATIVO TOKIO TOPCOAT (Console Web & WebSockets)
# ==============================================================================

# Iniciar servidor full-stack reativo Tokio Topcoat com dashboard em tempo real:
orbity serve --port 8080
# Console disponível em: http://127.0.0.1:8080
```

---

## 🛠️ Guia de Implantação e Instalação em Produção

### 1. Pré-requisitos de Sistema (Linux)
- **Kernel Linux:** $\ge 5.15$ com unprivileged user namespaces ativado (`sysctl -w kernel.unprivileged_userns_clone=1`).
- **Bubblewrap:** `/usr/bin/bwrap` para confinamento seguro de processos (`sudo apt install bubblewrap` ou `dnf install bubblewrap`).
- **SQLite:** 3.45+ compilado com suporte a WAL.
- **Suíte de 5 CLIs Nativas:** `codex`, `claude`, `agy`, `hermes` e `pi` acessíveis no `$PATH`.

### 2. Compilação Otimizada
```bash
# Compilar todo o workspace em modo release com otimizações estritas
cargo build --release --workspace

# O binário executável final estará disponível em:
./target/release/orbity --help
```

### 3. Execução como Serviço de Sistema (`systemd`)
Para ambientes de servidores e nuvem, crie o arquivo de serviço `/etc/systemd/system/orbity.service`:
```ini
[Unit]
Description=Orbity Multi-Agent Reactive Orchestrator
After=network.target

[Service]
Type=simple
User=orbity
Group=orbity
WorkingDirectory=/opt/orbity
ExecStart=/opt/orbity/target/release/orbity serve --port 8080
Restart=always
RestartSec=5s
LimitNOFILE=65536
Environment="RUST_LOG=info,orbity_core=debug,orbity_server=debug"
Environment="DATABASE_URL=sqlite:///opt/orbity/orbity.db?mode=rwc"

[Install]
WantedBy=multi-user.target
```

---

## 📚 Documentação Técnica Completa do Projeto

- [📖 Manual Didático de Arquitetura & Engenharia (Produção)](overview.md) — Explicação aprofundada de todos os componentes, fluxo de vida, garantias de segurança e FinOps.
- [🗺️ Mapa da Jornada Multi-Agente & Varredura Completa](JOURNEY_MAP_AND_AUDIT.md) — As 4 fases de vida: Criação, Execução, Análise e Finalização.
- [📜 Contratos, Ciclo de Vida e Setup Declarativo (YAML)](CONTRACTS_AND_LIFECYCLE.md) — Especificações de esquemas, políticas de auto-aprovação e enums Rust.
- [📋 Plano de Implementação em Quality Gates (0 ao 7)](IMPLEMENTATION_PLAN.md) — Registro completo dos 8 gates, DoD e evidências de commits.
- [Exemplo de Equipe Declarativa: forester.yaml](examples/teams/forester.yaml) — Arquivo YAML pronto para orquestração em produção.
- [Exemplo de Agente Declarativo: agente01.yaml](examples/agents/agente01.yaml) — Exemplo de agente individual.
- [🌐 Portal Interativo GitHub Pages](docs/index.html) — Simulador de grafos interativo, console de auditoria e matriz de tarefas.
