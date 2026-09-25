# Orquestração de agentes via CLI

#### O ChatGPT disse:

Para **logs**, eu separaria em 4 camadas para não misturar observabilidade com auditoria:

Tipo

O que registra

Exemplo

**Runtime logs**

lifecycle do agente/processo

start, stop, timeout, retry, crash

**Execution logs**

o que o agente executou

comando, ferramenta, arquivo alterado

**Audit logs**

trilha imutável de segurança/governança

quem fez, quando, com qual permissão

**Telemetry logs**

performance e custo

duração, CPU, RAM, tokens, latência

Formato padrão: **JSON estruturado**.

JSON

```
{
  "timestamp": "2026-09-25T12:30:44Z",
  "level": "INFO",
  "run_id": "run_123",
  "task_id": "task_456",
  "agent": "codex",
  "event": "command.executed",
  "command": "cargo test",
  "exit_code": 0,
  "duration_ms": 1832,
  "sandbox_id": "sbx_42"
}
```

Para Rust, eu usaria principalmente:

```
tracing
tracing-subscriber
serde_json
opentelemetry
```

e faria tudo passar por um **Event/Audit Bus**:

```
Codex ─────┐
Claude ────┼─> tracing/events
Hermes ────┘
               │
               ├─> stdout JSON
               ├─> local file
               ├─> OpenTelemetry
               └─> audit storage
```

Para armazenamento, começaria simples:

```
logs locais JSONL
+
SQLite para runs/tasks
+
OpenTelemetry -> Grafana/Loki
```

Para auditoria forte, eu não usaria somente Loki. Teria um **audit store append-only**, idealmente com hash encadeado:

```
event_1
 hash: AAA

event_2
 previous_hash: AAA
 hash: BBB

event_3
 previous_hash: BBB
 hash: CCC
```

Assim, se alguém alterar um registro histórico, você detecta.

Eu criaria eventos bem definidos:

Rust

```
AgentStarted
AgentFinished
CommandExecuted
FileRead
FileWritten
NetworkRequest
ToolCalled
PolicyAllowed
PolicyDenied
SecretRequested
SecretGranted
SandboxCreated
SandboxDestroyed
AgentFailed
```

E uma regra importante: **não registrar secrets, tokens ou prompts sensíveis em plaintext**. O log deveria carregar referências ou hashes:

JSON

```
{
  "event": "secret.accessed",
  "secret_id": "github_token",
  "agent": "codex",
  "value_logged": false
}
```

O desenho final ficaria:

```
                 Agent Runtime
                      │
             structured events
                      │
              ┌───────▼───────┐
              │ Event Collector│
              └───────┬───────┘
                      │
        ┌─────────────┼────────────┐
        ▼             ▼            ▼
      Loki         OTEL         Audit DB
   observability   metrics     immutable
        │             │            │
        └─────────────┼────────────┘
                      ▼
                    Grafana
```

A combinação que eu escolheria no MVP é: **Rust `tracing` + OpenTelemetry + Grafana/Loki + SQLite/Postgres para auditoria estruturada**. Isso já te dá uma base empresarial bem sólida.

  

#### Você disse:

tokens usados

  

#### O ChatGPT disse:

Sim — **tokens usados devem ser parte central da auditoria**, por execução, agente, tarefa e sessão.

Eu registraria no `AgentResult`:

Rust

```
struct TokenUsage {
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
    reasoning_tokens: u64,
    total_tokens: u64,
    estimated_cost_usd: Option<f64>,
}
```

E cada execução geraria algo assim:

JSON

```
{
  "run_id": "run_123",
  "agent": "hermes",
  "model": "claude-sonnet",
  "input_tokens": 18240,
  "output_tokens": 3150,
  "cache_read_tokens": 9200,
  "reasoning_tokens": 0,
  "total_tokens": 21390,
  "estimated_cost_usd": 0.14
}
```

Isso é factível diretamente pelos CLIs/APIs. O Hermes, por exemplo, hoje já tem `--usage-file` e retorna `input_tokens`, `output_tokens`, cache, reasoning, total, chamadas de API e custo estimado; o modo `stream-json` também inclui tokens no evento final. [![](https://www.google.com/s2/favicons?domain=https://github.com&sz=128)GitHub](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/reference/cli-commands.md?utm_source=chatgpt.com) A API/OpenAI CLI retorna `usage.input_tokens`, `output_tokens` e `total_tokens`. [![](https://www.google.com/s2/favicons?domain=https://developers.openai.com&sz=128)OpenAI Developers+1](https://developers.openai.com/api/docs/libraries/openai-cli?utm_source=chatgpt.com)

No runtime eu agregaria em vários níveis:

```
Task
 ├─ Codex       31.2K tokens   $0.18
 ├─ Claude      44.8K tokens   $0.31
 ├─ Hermes      19.5K tokens   $0.09
 └─ Astra       12.1K tokens   $0.07
                              ─────
 TOTAL          107.6K         $0.65
```

E isso abre uma funcionalidade muito interessante: **o scheduler passa a decidir também por orçamento**.

```
task budget = $1.00

Astra
 ├─ Codex $0.20
 ├─ Claude $0.30
 ├─ Hermes $0.12
 └─ synthesis $0.08

spent      $0.70
remaining  $0.30
```

Você poderia definir políticas:

YAML

```
budget:
  max_cost_usd: 1.00
  max_tokens: 150000
  max_agent_calls: 8

routing:
  expensive_model_requires_approval_above: 0.50
```

E gerar eventos:

Rust

```
TokenUsageUpdated
BudgetThresholdReached
BudgetExceeded
ContextWindowThresholdReached
```

Então teu **HyperCore Runtime** passaria a auditar quatro dimensões simultaneamente:

**segurança + ações + recursos computacionais + economia de tokens**.

Isso é particularmente forte comercialmente, porque você consegue mostrar para uma empresa algo como:

> “Essa tarefa consumiu 84 mil tokens, US$ 0,41, chamou três agentes, executou 17 ferramentas e alterou quatro arquivos.”

Aí deixa de ser apenas observabilidade técnica e vira **FinOps para agentes de IA**.

![](https://www.google.com/s2/favicons?domain=https://developers.openai.com&sz=32)

Fontes

  

#### Você disse:

e uma camada de visualização em tempo real tipo react flow ou algo mais simplificado

  

#### O ChatGPT disse:

Sim. Para esse caso eu faria **bem mais simples que React Flow no início**.

A visualização precisa responder 5 coisas em tempo real:

Pergunta

Visual

Quem está rodando?

nó por agente

Quem chamou quem?

aresta entre agentes

Qual estado?

running / waiting / failed / done

Quanto gastou?

tokens + custo no nó

O que está acontecendo?

painel lateral de eventos/logs

Eu usaria esta arquitetura:

```
Rust Runtime
   │
   ├─ WebSocket / SSE
   ▼
Web UI
   │
   ├─ Agent Graph
   ├─ Timeline
   ├─ Token/Cost counters
   └─ Event Console
```

Na UI, algo assim:

```
        ┌──────────────┐
        │ Astra        │
        │ running      │
        │ 12.4k tokens │
        └──────┬───────┘
               │
      ┌────────┼────────┐
      ▼        ▼        ▼
 ┌────────┐ ┌────────┐ ┌────────┐
 │ Codex  │ │ Hermes │ │ Claude │
 │ done   │ │running │ │waiting │
 │ $0.12  │ │ $0.08  │ │ $0.00  │
 └────────┘ └────────┘ └────────┘
```

### Stack que eu escolheria

Para MVP:

-   **frontend:** Svelte ou React
    
-   **grafo:** `@xyflow/react` / React Flow
    
-   **streaming:** WebSocket
    
-   **backend:** Axum em Rust
    
-   **estado:** evento em memória + persistência no runtime
    

Mas tem uma alternativa ainda mais simples: **HTML + SVG puro**.

Você não precisa começar com um canvas complexo. Cada agente pode ser um card posicionado em grid, e as conexões desenhadas com SVG.

A vantagem é que fica muito mais controlável e leve.

### Eu faria 3 visualizações

1.  **Graph view**  
    quem chama quem
    
2.  **Timeline view**  
    execução ao longo do tempo
    

```
Astra   █████████████████████
Codex      ███████
Hermes     ███████████
Claude        █████████
```

3.  **Trace view**
    

```
run_123
 ├─ Astra.plan
 ├─ Codex.execute
 │   ├─ read_file
 │   ├─ edit_file
 │   └─ cargo_test
 ├─ Claude.review
 └─ Astra.synthesis
```

Essa terceira é provavelmente a mais útil para auditoria.

Eu inclusive faria o runtime emitir eventos compatíveis com OpenTelemetry:

```
run
 └─ span: supervisor
      ├─ span: codex
      ├─ span: hermes
      └─ span: claude
```

A UI simplesmente transforma os spans em grafo.

### Minha recomendação

Não comece pelo React Flow como produto principal.

Comece por:

```
Agent Cards
+
SVG edges
+
WebSocket
+
Timeline
+
Event stream
```

E só depois migre para React Flow quando tiver:

-   workflows dinâmicos
    
-   dezenas de agentes
    
-   branches
    
-   retries
    
-   nested agents
    
-   drag-and-drop
    
-   edição visual
    

Para **4 agentes**, React Flow é quase overkill.

A interface que eu imagino é algo próximo de um **distributed tracing dashboard para agentes**, não uma ferramenta de BPM. Essa distinção é importante: o usuário está **observando a execução**, não necessariamente desenhando o workflow.

O ChatGPT pode cometer erros. Por isso, lembre-se de conferir informações relevantes.

  

Média

## Embedded Content