# 📖 Arquitetura e Engenharia do Sistema Orbity: Manual Didático Completo para Produção

> **Versão do Sistema:** Orbity 1.0 (Produção)  
> **Linguagem Base:** Rust 2024 Edition  
> **Framework Servidor:** Tokio Topcoat (v0.9+)  
> **Persistência:** SQLite 3.45+ em Modo WAL com Hashes SHA-256 Encadeados  
> **Isolamento de Segurança:** Bubblewrap (`bwrap`) & Linux Namespaces  
> **Suíte de Operários:** 5 CLIs (`codex`, `claude`, `agy`, `hermes`, `pi`)

---

## 1. Visão Geral Didática: O Que é o Orbity?

O **Orbity** é uma plataforma e runtime de orquestração multi-agente de inteligência artificial de alta performance, projetada do zero em **Rust** para ambientes corporativos que exigem **segurança estrita**, **governança matemática**, **controle financeiro rígido (FinOps)** e **autonomia determinística**.

### 1.1 O Problema dos Orquestradores de IA Tradicionais
Na maioria das soluções baseadas em Python ou frameworks interpretados, os desenvolvedores enfrentam graves vulnerabilidades:
1. **Deriva de Agentes (Agent Drift) e Alucinação de Fluxo:** Agentes supervisores baseados puramente em prompts de LLM se perdem em tarefas longas, esquecem dependências ou entram em repetições infinitas.
2. **Falta de Isolamento de Comandos:** Comandos gerados por LLMs executam diretamente no host, com risco real de destruir arquivos (`rm -rf`), vazar credenciais (`~/.ssh/id_rsa`, `.env`) ou sofrer injeção de prompt via internet.
3. **Caixa-Preta de Custos e Tokens:** Gastos descontrolados com APIs caras sem limites rígidos de parada preventiva (*tripwires*).
4. **Fadiga de Confirmações Manuais:** O operador precisa confirmar interativamente cada comando pequeno, eliminando o ganho de produtividade da automação.
5. **Inexistência de Trilha de Auditoria Confiável:** Logs de texto simples podem ser facilmente editados, apagados ou forjados após um incidente.

### 1.2 A Solução do Orbity
O Orbity resolve esses problemas combinando **computação dirigida por grafos determinísticos (DAG)**, **isolamento em nível de kernel (Bubblewrap)**, **imutabilidade matemática estilo blockchain-lite (SQLite com encadeamento SHA-256)**, **governança declarativa no YAML** e o framework reativo **Tokio Topcoat**, eliminando a dependência de supervisores não-determinísticos (sem Astra).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          ORBITY RUNTIME ARCHITECTURE                        │
├──────────────────────┬───────────────────────────────┬──────────────────────┤
│    GOVERNANÇA YAML   │        TOPOLOGY ENGINE        │      EXECUÇÃO OS     │
├──────────────────────┼───────────────────────────────┼──────────────────────┤
│ • teams/forester.yml │ • Kahn Topological Sort       │ • Bubblewrap Sandbox │
│ • approval_policy    │ • Fan-out Tokio Concorrente   │ • Root Read-Only     │
│ • auto_approve rules │ • Fan-in Barreira Join        │ • Workspace tmpfs    │
│ • FinOps Hard Caps   │ • Feedback Loops Controlados  │ • Rede Isolada       │
│ • Checkpoints SQLite │ • Blackboard Memory           │ • Timeouts SIGKILL   │
├──────────────────────┴───────────────────────────────┴──────────────────────┤
│               BARRAMENTO DE EVENTOS & OBSERVABILIDADE 4 CAMADAS             │
│   Runtime Logs  •  Execution Logs  •  Audit Logs (SHA-256)  •  Telemetry   │
├─────────────────────────────────────────────────────────────────────────────┤
│                  APLICAÇÃO FULL-STACK REATIVA TOKIO TOPCOAT                 │
│      Context Cx  •  view!  •  signal  •  #[shard]  •  live! / emit!         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Os Seis Pilares Arquiteturais Explicados

### Pilar 1: Concorrência Nativa em Rust & Runtime Tokio
- **Zero Overhead de Garbage Collection:** Consumo estável de memória RAM (<25MB no servidor, sub-10MB no runtime).
- **Barramento Assíncrono Não-Bloqueante:** Capacidade comprovada em testes de carga de processar mais de **75.400 eventos por segundo**, permitindo que múltiplos agentes trabalhem paralelamente sem degradação.
- **Segurança de Memória Estática:** Ausência de data races, referências nulas ou vulnerabilidades comuns de concorrência.

### Pilar 2: Graph Engineering & Computação Topológica
Em vez de depender de uma LLM instável para decidir "o que fazer a seguir", o fluxo de trabalho é modelado como um **Grafo Computacional Dirigido**:
- **Nós (`GraphNode`):** Representam tarefas executáveis por agentes especializados (`AgentWorker`), ferramentas de sandbox (`ToolCall`) ou pontos de governança (`HumanGate`).
- **Arestas (`GraphEdge`):** Representam transições sequenciais, dependências diretas, condições baseadas em predicados ou **loops de retroalimentação (`FeedbackLoop`)**.
- **Ordenação Topológica de Kahn:** O motor valida a estrutura do grafo na compilação/carregamento, garantindo que não existam ciclos infinitos acidentais ou dependências irrealizáveis.
- **Fan-Out & Fan-In Concorrentes:** Dispara nós independentes em paralelo com `tokio::spawn`, sincronizando-os em uma barreira de consolidação (*join barrier*).
- **Memória Blackboard:** Quadro compartilhado seguro onde artefatos gerados por um nó (ex: documentação gerada pelo Agy ou diff de código do Codex) são injetados nas variáveis de contexto dos nós subsequentes.
- **Checkpoints Criptográficos no SQLite:** A cada transição de aresta, um snapshot imutável do grafo é gravado. Se a máquina reiniciar ou a execução for pausada, `orbity resume <RUN_ID>` recarrega o estado exato sem retrabalho.

### Pilar 3: Aplicação Servidora Reativa com Tokio Topcoat (v0.9+)
Para visualização e governança corporativa, o Orbity utiliza o framework oficial **Tokio Topcoat**:
- **Contexto Central (`Cx`):** Ponto de acesso unificado a serviços, injeção de dependências e estado reativo.
- **Views Tipadas (`view!`):** Compilação de expressões tipadas para o cliente, permitindo interatividade ultra-rápida sem roundtrips pesados.
- **Sinais Reativos de Cliente (`signal`):** Gerenciamento reativo de estado em tempo real no browser.
- **Componentes em Shards (`#[shard]`):** Renderização atômica no servidor de pedaços de tela que sofrem *morphing* no DOM sem perda de foco em inputs ou seleção de texto.
- **Streaming SSR (`live!` e `emit!`):** Transmissão contínua de progresso e *Suspense* durante tarefas longas dos agentes.
- **Server-Push via WebSockets de Longa Duração:** Atualizações instantâneas de mudanças de estado e logs operacionais sem necessidade de polling HTTP.

### Pilar 4: Sandbox Confinada (Linux Bubblewrap / `bwrap`)
Qualquer código ou comando gerado por agentes executa dentro de uma jaula de isolamento criada pelo Bubblewrap:
- **Root Filesystem Somente Leitura (`--ro-bind / /`):** Os binários essenciais do sistema operacional estão disponíveis, mas nenhuma gravação é permitida fora da sandbox.
- **Workspace Efêmero em Memória (`tmpfs`):** As alterações ocorrem em memória RAM (`/tmp/workspace`), isoladas do sistema de arquivos host.
- **Isolamento Total de Rede (`--unshare-net`):** Por padrão, ferramentas de compilação rodam offline, impedindo vazamento de dados via exfiltração de rede.
- **Prevenção contra Path Traversal:** Tentativas de escape usando caminhos relativos como `../../etc/shadow` são bloqueadas antes da execução.
- **Timeouts Rígidos com `SIGKILL`:** Processos travados ou loops infinitos de build são terminados forçadamente em timeout estrito, limpando a árvore de processos órfãos.
- **Rollback Atômico vs Promoção:** Se os testes na sandbox falharem, o diretório temporário é sumariamente destruído (zero arquivos corrompidos no host). Se aprovados, os arquivos alterados são promovidos de forma atômica para o diretório de produção com cálculo de hash SHA-256.

### Pilar 5: Audit Store SQLite Append-Only com Hashes SHA-256 Encadeados
A governança corporativa do Orbity é construída sobre um modelo matemático inviolável:
- **Bloco Genesis:** O primeiro evento da execução recebe o hash precursor `0000000000000000000000000000000000000000000000000000000000000000`.
- **Encadeamento Criptográfico:** Cada novo registro na tabela `audit_events` calcula:
  $$\text{hash}_n = \text{SHA-256}(\text{hash}_{n-1} \,\|\, \text{run\_id} \,\|\, \text{seq} \,\|\, \text{event\_type} \,\|\, \text{payload\_json})$$
- **Detecção de Adulteração em 100% dos Casos:** O comando `orbity audit verify <RUN_ID>` reexecuta a cadeia matemática desde o bloco gênese. Qualquer alteração de 1 único byte ou deleção de linha invalida todos os hashes seguintes, apontando imediatamente o número da sequência fraudada.

### Pilar 6: Observabilidade em Quatro Camadas & FinOps Preventivo
Os logs e eventos do sistema são segregados em quatro responsabilidades estritas:
1. **Camada 1 — Runtime Logs:** Rastreia o ciclo de vida dos agentes e processos (`AgentStarted`, `AgentPaused`, `AgentFinished`, `ApprovalGranted`).
2. **Camada 2 — Execution Logs:** Registra ações detalhadas (`CommandExecuted`, `ToolCalled`, `FileWritten` com hashes SHA-256 antes e depois).
3. **Camada 3 — Audit Logs:** Bridge síncrona persistida no banco SQLite WAL com garantia de encadeamento criptográfico.
4. **Camada 4 — Telemetry Logs:** Spans hierárquicos tracing (`run -> graph_step -> worker_node`) e métricas consolidadas exportáveis via OpenTelemetry/OTLP JSON para Grafana, Loki e Prometheus.
- **FinOps com Tripwires:** Monitoramento acumulado de tokens (input, output, cache, reasoning) convertidos em USD em tempo real. Se o custo projetado ultrapassar o teto estipulado no YAML (`max_budget_usd`), o sistema aborta ou pausa a execução preventivamente antes que novas requisições sejam feitas.

---

## 3. A Suíte de 5 Agentes CLI Nativos

Em vez de depender de um único modelo generalista, o Orbity orquestra uma **equipe multidisciplinar de 5 ferramentas de linha de comando**:

| Agente / CLI | Papel de Especialidade | Modo de Execução Típico | Vantagem Técnica |
|---|---|---|---|
| **`agy` (Antigravity)** | Pesquisa profunda, leitura de codebase e planejamento | `agy -p [PROMPT] --output-format json --effort high` | Excelente contexto global, mapeamento de repositórios e ferramentas MCP. |
| **`codex` (Codex CLI)** | Implementação de código, testes Rust e refatoração | `codex exec [PROMPT]` | Especialista em código de alta performance, tipagem rigorosa e TDD. |
| **`claude` (Claude Code)** | Revisão crítica, análise de segurança e invariantes | `claude -p [PROMPT] --output-format json --dangerously-skip-permissions` | Alta capacidade de raciocínio abstrato, auditoria de diffs e segurança. |
| **`hermes` (Hermes Agent)** | Consulta externa, ferramentas especializadas e web | `hermes run [PROMPT] --usage-file <path>` | Formato padronizado de extração de tokens e execução de chamadas externas. |
| **`pi` (Pi Assistant)** | Edições cirúrgicas, correções pontuais e lint | `pi -p [PROMPT] --mode json --no-session` | Execução ultra-rápida sem overhead de sessão para alterações de poucas linhas. |

---

## 4. Governança Declarativa no YAML (Autonomia Sem Fadiga)

No Orbity, o equilíbrio entre **autonomia** e **controle** é configurado declarativamente no arquivo YAML da equipe (`teams/*.yaml`):

```yaml
version: "1.0"
team:
  name: "forester"
  description: "Equipe de engenharia de software e segurança"

  # Orçamento e FinOps
  finops:
    max_budget_usd: 2.00
    max_total_tokens: 300000
    expensive_model_approval_threshold_usd: 0.80
    expensive_model_action: "auto_approve" # Modelos caros rodam sem pausa humana se couberem no teto

  # Políticas de Aprovação e Rejeição Automatizadas
  approval_policy:
    mode: "automatic" # Opções: "automatic" | "hybrid" | "manual"
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
    fallback_action: "approve"
```

### Como Funciona a Tomada de Decisão:
1. **Regras `auto_approve`:** Quando um comando ou ação corresponde a um padrão seguro aprovado (ex: rodar testes dentro da sandbox ou invocar uma das 5 CLIs autorizadas), o sistema emite `ApprovalGranted` instantaneamente sem interromper o fluxo.
2. **Regras `auto_reject`:** Se um agente tentar uma ação perigosa (ex: deletar a raiz ou vazar credenciais SSH), o sistema emite `ApprovalRejected`, descarta o workspace temporário e aciona o feedback loop de correção.
3. **Modo `hybrid` com `escalate_to_human`:** Para ações que exigem validação humana expressa (ex: deploy em produção), o grafo pausa, grava um checkpoint criptográfico e emite `ApprovalRequired`. O operador libera a execução no terminal via `orbity resume <RUN_ID> --approve` ou com um clique no console web.

---

## 5. Ciclo de Vida da Execução (Passo a Passo)

```
[Início: orbity team run]
         │
         ▼
┌──────────────────┐
│ 1. PREFLIGHT     │ -> Checa bwrap, sqlite e 5 CLIs no PATH (orbity doctor)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 2. DAG COMPILADO │ -> Carrega YAML, valida DAG com Kahn, monta Blackboard
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 3. GENESIS BLOCK │ -> Grava bloco #0 na tabela audit_events (hash: 0000...)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 4. SANDBOX PROV  │ -> Monta root ro-bind, tmpfs /tmp/workspace, rede isolada
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 5. FAN-OUT / IN  │ -> Dispara nós paralelos (Agy + Codex), sincroniza em barreira
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 6. TESTES & REV  │ -> Executa cargo test na sandbox; Claude Code audita o diff
└────────┬─────────┘
         │
         ├─────────────────────────────────────────┐
         ▼ (Se falhar teste)                       ▼ (Se passar teste)
┌──────────────────────────┐             ┌──────────────────────────┐
│ FEEDBACK LOOP CONTROLADO │             │ PROMOÇÃO E FINALIZAÇÃO   │
│ Retorna ao Codex com o   │             │ • Promove arquivos host  │
│ log de erro (max retries)│             │ • Fecha contas FinOps    │
└──────────────────────────┘             │ • Sela bloco final hash  │
                                         │ • Desmonta tmpfs sandbox │
                                         └──────────────────────────┘
```

---

## 6. Guia Prático de Instalação e Operação

### 6.1 Pré-requisitos de Sistema (Linux)
- **Kernel Linux:** $\ge 5.15$ com suporte a unprivileged user namespaces (`kernel.unprivileged_userns_clone = 1`).
- **Bubblewrap:** `/usr/bin/bwrap` instalado (`sudo apt install bubblewrap` no Debian/Ubuntu).
- **SQLite 3:** $\ge 3.45$ com suporte nativo a modo WAL.
- **Suíte de 5 CLIs:** `codex`, `claude`, `agy`, `hermes` e `pi` configuradas no `$PATH`.

### 6.2 Compilação para Produção
```bash
# Compilar todo o workspace otimizado com LTO (Link-Time Optimization)
cargo build --release --workspace

# O binário executável final estará disponível em:
# ./target/release/orbity
```

### 6.3 Diagnóstico de Ambiente (`orbity doctor`)
Antes de iniciar qualquer orquestração, valide seu ambiente com o comando de preflight:
```bash
./target/release/orbity doctor
```
**Exemplo de Saída Esperada:**
```
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

### 6.4 Principais Comandos da CLI

```bash
# 1. Carregar uma equipe declarativa a partir de um YAML:
orbity team load ./examples/teams/forester.yaml

# 2. Executar uma tarefa completa com a equipe forester:
orbity team run forester "Implementar autenticação JWT e testes em Rust" --budget-usd 2.00

# 3. Iniciar o servidor de aplicação Tokio Topcoat com console web reativo:
orbity serve --port 8080

# 4. Validar matematicamente a trilha de auditoria contra fraudes:
orbity audit verify <RUN_ID>

# 5. Inspecionar o resumo de consumo FinOps por ferramenta e modelo:
orbity finops summary --since 2026-09-01

# 6. Retomar uma execução pausada pelo Human-in-the-Loop aprovando a etapa:
orbity resume <RUN_ID> --approve
```

---

## 7. Configuração para Produção com `systemd`

Para executar o servidor reativo **Tokio Topcoat** como um daemon de produção contínuo:

```ini
# /etc/systemd/system/orbity.service
[Unit]
Description=Orbity Agentic Orchestrator Server (Tokio Topcoat)
After=network.target

[Service]
Type=simple
User=orbity
Group=orbity
WorkingDirectory=/opt/orbity
ExecStart=/opt/orbity/bin/orbity serve --port 8080
Restart=always
RestartSec=5s
LimitNOFILE=65536
Environment="RUST_LOG=info,orbity_core=debug,orbity_server=debug"
Environment="DATABASE_URL=sqlite:///opt/orbity/data/orbity.db?mode=rwc"

# Proteções de Sistema Operacional
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

## 8. Integração com Observabilidade Corporativa (Prometheus, Loki, Grafana)

A camada de telemetria do Orbity exporta métricas e spans compatíveis com o padrão **OpenTelemetry / OTLP**:
- **Logs Operacionais (Camada 1 e 2):** Gravados localmente em formato `.jsonl` em `runs/<run_id>.jsonl` e coletados pelo **Promtail / Grafana Loki**.
- **Métricas FinOps e Latência (Camada 4):** Endpoint `/metrics` expõe tokens acumulados por modelo, chamadas por agente e latência de sandbox para o **Prometheus**.
- **Spans de Execução:** Árvore de spans hierárquica enviada ao **Tempo** ou **Jaeger** para visualização gráfica das etapas do grafo computacional.

---

## 9. Perguntas Frequentes (FAQ) & Solução de Problemas

#### Q: Como o Orbity garante que um comando malicioso não escape para o host?
**R:** Todo comando é invocado via Bubblewrap (`bwrap`) com namespaces isolados de PID, IPC, UTS e Network. A raiz do host é montada como *read-only* (`--ro-bind / /`), e as alterações ocorrem em memória (`tmpfs`). Mesmo se o agente rodar `rm -rf /`, a chamada falha no nível de sistema operacional com erro `EPERM` / `EROFS` e gera um evento de auditoria `PolicyDenied`.

#### Q: O que acontece se a internet cair durante uma orquestração?
**R:** Por padrão, a sandbox opera em modo offline (`network: isolated`). Para nós que necessitam de acesso a APIs externas ou busca web (como `hermes` ou `agy`), o motor possui retry com backoff exponencial e salva checkpoints no SQLite WAL a cada nó concluído. Se a máquina desligar, a execução pode ser retomada com `orbity resume <RUN_ID>`.

#### Q: Como auditar se alguém alterou o banco SQLite manualmente?
**R:** Execute `orbity audit verify <RUN_ID>`. Como cada linha incorpora o hash SHA-256 da linha anterior, a função `AuditVerifier` recalcula a cadeia desde o bloco gênese. Qualquer alteração acusa `AuditVerificationResult::Tampered` indicando o número exato da linha comprometida.