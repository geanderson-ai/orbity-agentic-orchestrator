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

**Orbity** é um runtime e orquestrador corporativo de agentes autônomos de inteligência artificial construído em **Rust**, focado em **segurança de sandbox estrita**, **governança com trilha de auditoria append-only imutável em SQLite (com encadeamento de hash SHA-256)**, **observabilidade em 4 camadas** e **FinOps em tempo real** coordenado pelo supervisor **Astra**. Orgulhosamente desenvolvido e projetado no Brasil 🇧🇷.

🌐 **GitHub Pages do Projeto:** Disponível na pasta [`docs/`](docs/index.html) com visualizador interativo em tempo real, simulador de grafos, matriz de Gates & Tasks e console de auditoria.

---

## 🏛️ Os 6 Pilares do Sistema

1. **🦀 Rust & Concorrência Tokio:** Desempenho nativo com barramento assíncrono de eventos de alta vazão (>5.000 ev/s), gerenciamento de memória seguro e ausência de locks globais.
2. **🕸️ Graph Engineering (`Nodes` & `Edges`):** Modelagem rigorosa de times e fluxos como grafos computacionais dirigidos, com ordenação topológica (Kahn), fan-out paralelo, fan-in de barreira e loops de feedback controlados.
3. **🧠 Supervisor Astra:** Agente orquestrador que sintetiza grafos dirigidos, delega subtarefas especializadas para a suíte de 5 CLIs (`Codex`, `Claude`, `Agy`, `Hermes`, `Pi`) e gerencia a memória Blackboard.
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
| **`crates/orbity-agent`** | Supervisor Astra, suíte de 5 CLIs (`codex`, `claude`, `agy`, `hermes`, `pi`) e ciclo de vida de agentes. |
| **`crates/orbity-telemetry`** | Barramento de observabilidade com Rust `tracing` e exportador OpenTelemetry para Grafana/Loki. |
| **`crates/orbity-server`** | Servidor Axum embutido com streaming em tempo real (SSE e WebSockets) para a interface gráfica. |
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

- **[Gate 0: Fundação do Workspace, Tipos & Domínio de Eventos](IMPLEMENTATION_PLAN.md#gate-0-fundação-do-workspace-tipos--domínio-de-eventos)**
- **[Gate 1: Persistência SQLite & Audit Store Criptográfico](IMPLEMENTATION_PLAN.md#gate-1-persistência-sqlite--audit-store-criptográfico)**
- **[Gate 2: Mecanismo de Sandbox & Isolamento de Processos](IMPLEMENTATION_PLAN.md#gate-2-mecanismo-de-sandbox--isolamento-de-processos)**
- **[Gate 3: Barramento Unificado de Eventos & Observabilidade em 4 Camadas](IMPLEMENTATION_PLAN.md#gate-3-barramento-unificado-de-eventos--observabilidade-em-4-camadas)**
- **[Gate 4: Engine de FinOps, Orçamento & Supervisão com Astra](IMPLEMENTATION_PLAN.md#gate-4-engine-de-finops-orçamento--supervisão-com-astra)**
- **[Gate 5: Interface CLI de Orquestração](IMPLEMENTATION_PLAN.md#gate-5-interface-cli-de-orquestração)**
- **[Gate 6: Streaming em Tempo Real & Camada de Visualização](IMPLEMENTATION_PLAN.md#gate-6-streaming-em-tempo-real--camada-de-visualização)**
- **[Gate 7: Testes E2E, Validação de Segurança & Hardening](IMPLEMENTATION_PLAN.md#gate-7-testes-e2e-validação-de-segurança--hardening)**

Consulte o documento completo com todas as 32 tarefas detalhadas em [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).

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

# Executar tarefa com a equipe 'forester' liderada pelo supervisor Astra
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
```

---

## 📚 Documentos do Projeto

- [Mapa da Jornada Multi-Agente & Varredura (Criação, Execução, Análise, Finalização)](JOURNEY_MAP_AND_AUDIT.md)
- [Contratos, Ciclo de Vida e Setup Declarativo (YAML)](CONTRACTS_AND_LIFECYCLE.md)
- [Plano de Implementação em Gates e Tasks](IMPLEMENTATION_PLAN.md)
- [Exemplo de Equipe Declarativa: forester.yaml](examples/teams/forester.yaml)
- [Exemplo de Agente Declarativo: agente01.yaml](examples/agents/agente01.yaml)
- [Documento de Overview Original](overview.md)
- [Página Web GitHub Pages](docs/index.html)
- [Workflow de Deploy do GitHub Pages](.github/workflows/deploy-pages.yml)
