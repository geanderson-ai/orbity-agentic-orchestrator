<div align="center">
  <img src="assets/orbity-logo.jpg" alt="Orbity Logo" width="220" style="border-radius: 50%; box-shadow: 0 8px 30px rgba(0,255,163,0.2);" />
  <h1>🪐 ORBITY</h1>
  <p><strong>Orquestrador Autônomo Multi-Agente de Alta Performance em Rust</strong></p>

  [![Made in Brazil](https://img.shields.io/badge/Made%20in-Brasil%20%F0%9F%87%A7%F0%9F%87%B7-009c3b?style=for-the-badge&logoColor=white)](https://github.com/geanderson/meza-agentic-orchestrator)
  [![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
  [![SQLite](https://img.shields.io/badge/SQLite-WAL_%2B_SHA--256-blue?style=for-the-badge&logo=sqlite)](https://sqlite.org)
  [![Sandbox](https://img.shields.io/badge/Sandbox-Bubblewrap-cyan?style=for-the-badge)](https://github.com/containers/bubblewrap)
  [![Tokio Topcoat](https://img.shields.io/badge/Tokio-Topcoat_0.9-blueviolet?style=for-the-badge)](https://tokio.rs)
  [![GitHub Pages](https://img.shields.io/badge/GitHub_Pages-Online-indigo?style=for-the-badge&logo=github)](docs/index.html)
</div>

<br/>

**Orbity** é um runtime e orquestrador corporativo de inteligência artificial autônoma construído em **Rust**. Ele substitui a fragilidade de scripts soltos por **engenharia determinística baseada em grafos computacionais (DAG)**, **isolamento estrito no nível do kernel com Bubblewrap**, **governança matemática com trilha append-only no SQLite WAL (hasheada com SHA-256)**, **FinOps preventivo com tripwires orçamentários** e um console web reativo em tempo real alimentado pelo framework **Tokio Topcoat**.

Orgulhosamente projetado e desenvolvido no Brasil 🇧🇷.

---

## 🎯 Por Que o Orbity?

A maioria dos orquestradores de agentes de IA executa código de forma perigosa: scripts em Python rodam diretamente no seu sistema operacional, sofrem com alucinações de fluxo, entram em repetições infinitas e geram faturas surpresa de API.

O **Orbity** foi projetado para levar a autonomia de agentes para a **produção industrial**:

- 🛡️ **Confinamento em Sandbox Estrita:** Todo comando executa em jaulas seguras via Bubblewrap (`bwrap`) com o sistema host em modo somente leitura (`ro-bind`), diretório efêmero em RAM (`tmpfs`) e rede isolada. Se um teste falhar, o rollback é instantâneo (2ms).
- 🔗 **Trilha de Auditoria Criptográfica Inviolável:** Cada comando, arquivo modificado e decisão é registrado em uma cadeia append-only no SQLite WAL com hashes SHA-256 encadeados estilo blockchain-lite. Qualquer adulteração histórica é detectada com 100% de precisão matemática.
- 🕸️ **Computação Dirigida por Grafos (DAG):** Em vez de agentes supervisores imprevisíveis, o fluxo é validado na compilação com a ordenação topológica de Kahn, disparando tarefas em paralelo (*fan-out*) e sincronizando em barreiras (*fan-in*).
- 👥 **Suíte de 5 Ferramentas CLI Integradas:** Roteamento nativo das melhores ferramentas de desenvolvimento: `agy` (pesquisa e planejamento), `codex` (geração de código e testes Rust), `claude` (revisão crítica e diffs), `hermes` (busca externa e ferramentas) e `pi` (edições cirúrgicas rápidas).
- 💰 **FinOps com Tripwires Rígidos:** Monitoramento contínuo de tokens e custo acumulado em USD. Se o consumo projetado ameaçar estourar o orçamento definido no YAML, o sistema interrompe a execução preventivamente.
- ⚙️ **Governança Declarativa no YAML:** Políticas de auto-aprovação (`auto_approve`) e auto-rejeição (`auto_reject`) no próprio arquivo de configuração da equipe, garantindo autonomia máxima sem exigir validações manuais repetitivas para tarefas seguras.
- ⚡ **Servidor Web Reativo Tokio Topcoat:** Dashboard em tempo real com DOM morphing em componentes `#[shard]`, streaming SSR e WebSockets server-push sem polling.

---

## 🚀 Instalação e Pré-requisitos

### 1. Pré-requisitos de Sistema (Linux)
- **Kernel Linux:** $\ge 5.15$ com namespaces de usuário ativados (`sysctl -w kernel.unprivileged_userns_clone=1`).
- **Bubblewrap:** `/usr/bin/bwrap` para confinamento seguro de processos:
  ```bash
  # Ubuntu / Debian
  sudo apt install bubblewrap sqlite3
  # Fedora / RHEL
  sudo dnf install bubblewrap sqlite
  ```
- **Rust Toolchain:** Versão estável recente ($\ge 1.80$):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### 2. Compilar o Projeto
```bash
git clone https://github.com/geanderson/meza-agentic-orchestrator.git
cd meza-agentic-orchestrator

# Compilar em modo release com otimizações estritas
cargo build --release

# O binário da CLI estará pronto em:
./target/release/orbity --help
```

---

## 💻 Como Usar na Prática (Guia Passo a Passo)

### Passo 0: Diagnóstico do Ambiente (`orbity doctor`)
Antes de iniciar qualquer tarefa, execute o preflight health check para verificar se a sandbox, o SQLite e a suíte de 5 CLIs estão funcionais no seu sistema:

```bash
orbity doctor
```
**Exemplo de Saída Esperada:**
```text
⚡ Orbity Agentic Platform - Pre-flight Health Check
─────────────────────────────────────────────────────────────────────────────
[✓] Sandbox Runtime: Bubblewrap (/usr/bin/bwrap) 0.8.0
[✓] Storage Engine: SQLite 3.45.1 (WAL mode + SHA-256 Audit Store)

Suíte de Agentes CLI Detectados:
  [✓] codex   v0.154.0        -> /home/user/.local/bin/codex
  [✓] claude  v2.1.239        -> /home/user/.local/bin/claude
  [✓] agy     v1.2.11         -> /home/user/.local/bin/agy
  [✓] hermes  v0.21.0         -> /home/user/.local/bin/hermes
  [✓] pi      v0.78.1         -> /home/user/.hermes/node/bin/pi

✓ 5 de 5 CLIs operacionais. Ambiente pronto para orquestração multi-agente!
```

---

### Passo 1: Configuração Declarativa da Equipe (`teams/forester.yaml`)
As equipes e regras de governança são declaradas em arquivos YAML limpos:

```yaml
version: "1.0"
team:
  name: "forester"
  description: "Equipe de engenharia e segurança com Codex, Claude Code e Agy"

  # Controle Orçamentário e FinOps
  finops:
    max_budget_usd: 2.00
    max_total_tokens: 300000
    expensive_model_approval_threshold_usd: 0.80
    expensive_model_action: "auto_approve" # Roda sem pausa humana se couber no orçamento

  # Políticas de Aprovação e Rejeição Automatizadas
  approval_policy:
    mode: "automatic" # "automatic" | "hybrid" | "manual"
    auto_approve:
      - rule: "sandbox_tests_passed"
        condition: "outcome.exit_code == 0"
      - rule: "cost_within_budget"
        condition: "cumulative_cost_usd <= max_budget_usd"
      - rule: "allowed_cli_tools"
        tools: ["codex", "claude", "agy", "hermes", "pi"]
    auto_reject:
      - rule: "destructive_host_commands"
        patterns: ["rm -rf /", "*--no-preserve-root*", "*id_rsa*"]
      - rule: "budget_hard_cap_exceeded"
        condition: "cumulative_cost_usd > max_budget_usd"
```

---

### Passo 2: Carregar a Equipe no Sistema
O Orbity lê a pasta `examples/teams/` ou `teams/`, calcula o hash SHA-256 das definições e sincroniza automaticamente com o banco SQLite em sub-milissegundos:

```bash
orbity team load ./examples/teams/forester.yaml
```

---

### Passo 3: Executar uma Orquestração Multi-Agente
Dispare um objetivo em linguagem natural definindo um teto financeiro:

```bash
orbity team run forester "Auditar e refatorar conexões de banco SQLite em orbity-storage" --budget-usd 1.50
```

**O que acontece nos bastidores:**
1. O motor compila o grafo computacional (DAG) e estabelece o **Bloco Genesis #0** da trilha de auditoria.
2. A **Sandbox Bubblewrap** é criada com filesystem seguro e tmpfs efêmero em `/tmp/workspace`.
3. O **`agy`** pesquisa a estrutura da codebase e injeta os achados na memória compartilhada (**Blackboard**).
4. O **`codex`** escreve o código Rust e roda `cargo test` dentro do tmpfs isolado.
5. Se os testes passarem, o **`claude`** realiza a auditoria do diff de código.
6. Aprovado o resultado, os arquivos são promovidos de forma atômica para o host e o bloco final com hash SHA-256 é selado.

---

### Passo 4: Iniciar o Dashboard Web Reativo (`orbity serve`)
Inicie a aplicação servidora Tokio Topcoat para acompanhar os agentes em tempo real:

```bash
orbity serve --port 8080
```
Abra `http://127.0.0.1:8080` no navegador para visualizar:
- Cards dos agentes com streaming de progresso via WebSockets a 60 FPS.
- Componentes reativos `#[shard]` que se atualizam no servidor via morphing de DOM.
- Painel interativo de FinOps com medição de tokens e custo acumulado em USD.
- Botões de aprovação e rejeição com latência inferior a 50ms para exceções de governança.

---

### Passo 5: Validar a Integridade da Trilha de Auditoria
A qualquer momento, verifique se os registros de uma execução sofreram qualquer tipo de fraude ou manipulação:

```bash
orbity audit verify <RUN_ID>
```
- Retorna `0` com cadeia **100% íntegra e verificada**.
- Acusa `AuditVerificationResult::Tampered` apontando a linha exata se qualquer bit do banco tiver sido alterado manualmente.

---

### Passo 6: Retomar Execuções Pausadas (Human-in-the-Loop)
Se uma execução foi pausada por uma regra de salvaguarda ou solicitação de aprovação manual:

```bash
# Aprovar e destravar a execução:
orbity resume <RUN_ID> --approve

# Rejeitar e abortar com segurança:
orbity resume <RUN_ID> --reject
```

---

## 📖 Referência Rápida de Comandos da CLI (`orbity`)

| Comando | Descrição | Exemplo de Uso |
|---|---|---|
| `orbity doctor` | Executa o preflight health check de ferramentas, sandbox e SQLite. | `orbity doctor` |
| `orbity team load <PATH>` | Carrega e valida uma equipe a partir de um arquivo YAML. | `orbity team load ./examples/teams/forester.yaml` |
| `orbity team list` | Lista todas as equipes carregadas e ativas. | `orbity team list` |
| `orbity team run <TEAM> <PROMPT>` | Inicia a execução de um objetivo complexo com a equipe informada. | `orbity team run forester "Criar endpoint REST" --budget-usd 2.00` |
| `orbity serve [--port <PORT>]` | Inicia o servidor web Tokio Topcoat com o dashboard reativo. | `orbity serve --port 8080` |
| `orbity audit verify <RUN_ID>` | Valida matematicamente a integridade dos hashes SHA-256 encadeados. | `orbity audit verify run_74f9c` |
| `orbity finops summary` | Exibe relatório consolidado de gastos por ferramenta e modelo. | `orbity finops summary --since 2026-09-01` |
| `orbity resume <RUN_ID>` | Retoma uma execução pausada pelo guardrail FinOps ou HITL. | `orbity resume run_74f9c --approve` |
| `orbity status <RUN_ID>` | Exibe o estado em tempo real, nós ativos e tokens consumidos. | `orbity status run_74f9c` |
| `orbity logs <RUN_ID>` | Exibe logs estruturados com filtro de camadas (`runtime`, `execution`, `audit`, `telemetry`). | `orbity logs run_74f9c --layer audit --json` |
| `orbity agent create/list` | Operações CRUD declarativas de agentes individuais. | `orbity agent create -f ./examples/agents/agente01.yaml` |

---

## 🏭 Implantação em Produção como Serviço (`systemd`)

Para executar o servidor reativo **Tokio Topcoat** em servidores dedicados ou instâncias em nuvem Linux:

```ini
# /etc/systemd/system/orbity.service
[Unit]
Description=Orbity Multi-Agent Reactive Orchestrator Server
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
Environment="DATABASE_URL=sqlite:///opt/orbity/data/orbity.db?mode=rwc"

# Proteções do Kernel
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/opt/orbity/data /tmp
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Ativação do serviço:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now orbity.service
sudo systemctl status orbity.service
```

---

## 📚 Documentação Técnica Aprofundada

Para consultar especificações internas de arquitetura, diagramas de sequência detalhados, matriz de requisitos e históricos de engenharia, consulte os documentos na pasta [`docs/`](docs/):

- [📖 **Manual de Arquitetura & Engenharia (overview.md)**](docs/overview.md) — Guia aprofundado explicando os 6 pilares, garantias de segurança no kernel, modelo matemático de encadeamento SHA-256 e integração corporativa com Grafana/Loki.
- [📜 **Contratos, Ciclo de Vida e Setup Declarativo (CONTRACTS_AND_LIFECYCLE.md)**](docs/CONTRACTS_AND_LIFECYCLE.md) — Especificação detalhada de schemas YAML, enums Rust de ciclo de vida e regras de auto-aprovação.
- [🗺️ **Mapa da Jornada Multi-Agente & Varredura (JOURNEY_MAP_AND_AUDIT.md)**](docs/JOURNEY_MAP_AND_AUDIT.md) — Detalhamento minucioso das 4 fases (Criação, Execução, Análise, Finalização) e matriz de conformidade.
- [📋 **Plano de Implementação & Quality Gates (IMPLEMENTATION_PLAN.md)**](docs/IMPLEMENTATION_PLAN.md) — Blueprint dos 8 Quality Gates (0 ao 7), critérios de aceite (DoD) e registro criptográfico de commits.
- [🎬 **Storytelling & Roteiro de Vídeo Remotion (VIDEO_STORYTELLING_REMOTION.md)**](docs/VIDEO_STORYTELLING_REMOTION.md) — Roteiro cinematográfico e código React para o vídeo programático do Orbity.
- [🌐 **Portal Interativo Web (docs/index.html)**](docs/index.html) — Simulador interativo de grafos, terminal interativo e visualizador da matriz de tarefas.
- [📁 **Exemplos Práticos de Equipes e Agentes (examples/)**](examples/teams/forester.yaml) — Arquivos de exemplo prontos para uso em produção.

---

## 📄 Licença

Distribuído sob licença dual **MIT** ou **Apache-2.0**. Orgulhosamente construído com 🦀 Rust e projetado para a era da inteligência artificial autônoma determinística.
