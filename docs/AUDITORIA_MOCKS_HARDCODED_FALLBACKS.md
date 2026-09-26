# Auditoria de mocks, valores hardcoded e fallbacks

> Atualizado em 26 de setembro de 2026. Esta auditoria cobre o código Rust, o frontend embutido, scripts e arquivos YAML do repositório. Ocorrências repetidas com o mesmo comportamento foram agrupadas.

## 1. Mocks e dados simulados

### 1.1 Expostos no runtime

- **`MockSandbox` faz parte da API pública do crate.** Não está protegido por `cfg(test)` nem por uma feature específica. Respostas não cadastradas retornam sucesso, `exit_code = 0` e `"Mock stdout..."`. Ver [`crates/orbity-sandbox/src/mock.rs`](../crates/orbity-sandbox/src/mock.rs#L18) e [`crates/orbity-sandbox/src/lib.rs`](../crates/orbity-sandbox/src/lib.rs#L6).
- **`DefaultNodeRunner` também é público e sempre simula sucesso.** Usa 100 tokens de entrada, 50 de saída, custo de US$ 0,001 e duração de 10 ms. Ver [`crates/orbity-graph/src/executor.rs`](../crates/orbity-graph/src/executor.rs#L47) e [`crates/orbity-graph/src/lib.rs`](../crates/orbity-graph/src/lib.rs#L15).
- **Dashboard do servidor usa dados simulados.** FinOps, auditoria e os cinco agentes possuem custos, tokens, estados e quantidade de blocos fixos. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L44).
- **`/api/status` mistura métricas reais e dados fictícios.** A consulta ao SQLite é real, mas o orçamento de US$ 20 e os agentes sempre em estado `Ready` são estáticos. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L283).
- **`/api/models` retorna um catálogo estático.** Não há descoberta real das CLIs ou dos modelos instalados. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L327).
- **`/governance` é apenas uma página estática.** A página informa que o gate está ativo sem consultar o estado real. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L332).
- **`/finops` usa números fixos.** A página mostra US$ 1,25 e 45.000 tokens. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L337).
- **O progresso live infere conclusão por contagem de eventos.** Qualquer evento é contado como passo e o fluxo declara sucesso ao atingir `total_steps`, independentemente do resultado real. Ver [`crates/orbity-server/src/live.rs`](../crates/orbity-server/src/live.rs#L94).

### 1.2 Frontend demonstrativo

- **Terminal interativo inteiramente mockado.** Inclui versões de ferramentas, caminhos locais, execuções, hashes, tokens e custos. Ver [`index.html`](../index.html#L1904).
- **Simulador com sequência fixa.** Usa `setInterval` e valores predefinidos de agentes, logs, custos, tokens, hashes e resultado. Ver [`index.html`](../index.html#L1968).
- **Verificação de adulteração apenas visual.** O botão altera o DOM e não consulta a cadeia de auditoria real. Ver [`index.html`](../index.html#L2062).
- **Gates e tarefas estáticos.** Os sete gates são marcados como concluídos; algumas funcionalidades anunciadas não existem nos subcomandos atuais da CLI. Ver [`index.html`](../index.html#L1754).

### 1.3 Restritos a testes

- O E2E do Gate 4 cadastra respostas simuladas para Codex, Claude, Agy, Hermes e Pi via `MockSandbox`. Ver [`crates/orbity-agent/tests/gate_4_multi_agent_e2e.rs`](../crates/orbity-agent/tests/gate_4_multi_agent_e2e.rs#L24).
- `DefaultNodeRunner` é usado nos testes de grafo e resiliência para simular execução e FinOps. Ver [`crates/orbity-graph/tests/graph_engine_tests.rs`](../crates/orbity-graph/tests/graph_engine_tests.rs#L148) e [`crates/orbity-agent/tests/gate_7_hardening_and_resilience.rs`](../crates/orbity-agent/tests/gate_7_hardening_and_resilience.rs#L158).

Esses usos em testes são legítimos. O risco está em os mesmos tipos estarem disponíveis incondicionalmente no build normal.

## 2. Valores hardcoded

### 2.1 Modelos e integrações externas

O resolvedor de tiers mantém no binário os seguintes modelos:

| CLI | Fast | Balanced | Reasoning |
| --- | --- | --- | --- |
| Claude | `haiku` | `sonnet` | `opus` |
| Codex | `gpt-4o-mini` | `gpt-4o` | `o3-mini` |
| Agy | `gemini-3.8-flash-low` | `gemini-3.8-flash-high` | `gemini-3.1-pro-high` |
| Hermes | `openrouter/auto-fast` | `anthropic/claude-sonnet-4.6` | `anthropic/claude-sonnet-4.6` |
| Pi | `llama-cpp` | `sonnet` | `sonnet:high` |

Referências:

- Mapeamento de tiers e modelos: [`crates/orbity-graph/src/model_resolver.rs`](../crates/orbity-graph/src/model_resolver.rs#L71).
- Executáveis e formatos de argumentos das cinco CLIs: [`crates/orbity-agent/src/runners.rs`](../crates/orbity-agent/src/runners.rs#L35).
- Scaffolding ainda grava modelos específicos como `claude-3-7-sonnet` e `gpt-4o`: [`crates/orbity-cli/src/init.rs`](../crates/orbity-cli/src/init.rs#L97).

### 2.2 Orquestração e parser YAML

- Agente simples: CLI `codex`, timeout de 300 s, uma tentativa e orçamento de US$ 5. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L144).
- Agent file: CLI `agy`; passos recebem 300 s, uma tentativa e US$ 2; agente sem pipeline recebe US$ 5. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L199).
- Worker fallback: 300 s, uma tentativa e US$ 1 por nó. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L332).
- Pipeline de equipe: CLI inferida por substring; se não reconhecida, usa `agy`. Cada passo recebe 180 s, uma tentativa e US$ 0,50. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L398).
- Orquestrador padrão `topcoat`, tool timeout de 60 s, Human Gate com mensagem fixa, condição padrão de sucesso e feedback loop com três iterações. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L496).
- Timeout padrão do runner de agente também é 60 s, criando mais de uma fonte de verdade. Ver [`crates/orbity-agent/src/runners.rs`](../crates/orbity-agent/src/runners.rs#L127).

### 2.3 Servidor e infraestrutura

- Bind `127.0.0.1:3000`, limite de 10.000 conexões e live streaming habilitado. Ver [`crates/orbity-server/src/context.rs`](../crates/orbity-server/src/context.rs#L18).
- CLI repete porta 3000 e host `127.0.0.1`; banco padrão `orbity.db`, output `text` e sandbox `isolated`. Ver [`crates/orbity-cli/src/commands.rs`](../crates/orbity-cli/src/commands.rs#L18).
- CORS aceita origens locais por prefixo textual e começa com `http://127.0.0.1:3000`. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L194).
- Caminhos Bubblewrap `/usr/bin/bwrap` e `/usr/local/bin/bwrap`. Ver [`crates/orbity-sandbox/src/bwrap.rs`](../crates/orbity-sandbox/src/bwrap.rs#L37) e [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L258).
- Layout interno `/tmp/workspace`, `PATH`, `LANG=C.UTF-8` e `TERM=xterm-256color`. Ver [`crates/orbity-sandbox/src/bwrap.rs`](../crates/orbity-sandbox/src/bwrap.rs#L148).
- Pool SQLite usa timeout de 5 s e 16 conexões para arquivo; memória usa cinco conexões. Ver [`crates/orbity-storage/src/pool.rs`](../crates/orbity-storage/src/pool.rs#L20).
- Telemetria usa filtro `info,orbity=debug`, scope e versão `0.1.0` fixos. Ver [`crates/orbity-telemetry/src/setup.rs`](../crates/orbity-telemetry/src/setup.rs#L14) e [`crates/orbity-telemetry/src/sink.rs`](../crates/orbity-telemetry/src/sink.rs#L68).
- Versão e stage aparecem repetidos como `0.1.0-beta`, `beta` e `Topcoat 0.9`, em vez de serem derivados de uma fonte única. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L264), [`crates/orbity-cli/src/commands.rs`](../crates/orbity-cli/src/commands.rs#L6) e [`index.html`](../index.html#L1246).

### 2.4 Scaffolding e exemplos

- `orbity init` embute workspace, equipe, agentes, prompts, budgets, retries e timeouts diretamente no binário. Ver [`crates/orbity-cli/src/init.rs`](../crates/orbity-cli/src/init.rs#L34).
- Os YAMLs `examples/forester` contêm políticas, budgets, timeout de 86.400 s, comandos e caminhos temporários específicos. Como a CLI pode carregá-los automaticamente, eles deixam de ser somente exemplos. Ver [`examples/forester/teams/forester.yaml`](../examples/forester/teams/forester.yaml#L23).

## 3. Fallbacks

### 3.1 Segurança e governança

- **Política ausente resulta em aprovação automática.** Ver [`crates/orbity-graph/src/finops.rs`](../crates/orbity-graph/src/finops.rs#L75).
- **Defaults permissivos:** `ApprovalMode::Automatic`, `FallbackAction::Approve` e `ExpensiveModelAction::AutoApprove`. Ver [`crates/orbity-core/src/contracts.rs`](../crates/orbity-core/src/contracts.rs#L124).
- **`EscalateToHuman` em modo automático vira aprovação.** Ver [`crates/orbity-core/src/contracts.rs`](../crates/orbity-core/src/contracts.rs#L210).
- **Exemplos permissivos:** Forester e agente01 usam `fallback_action: approve`; ambos podem ser carregados automaticamente pela CLI. Ver [`examples/forester/teams/forester.yaml`](../examples/forester/teams/forester.yaml#L55) e [`examples/forester/agents/agente01.yaml`](../examples/forester/agents/agente01.yaml#L26).
- **Servidor sem autenticação por padrão.** `auth_token` começa como `None`, deixando APIs e governança sem autenticação. Ver [`crates/orbity-server/src/context.rs`](../crates/orbity-server/src/context.rs#L22).

### 3.2 Integridade e observabilidade

- **Datas inválidas viram `Utc::now()`.** Isso ocorre em audit chain, checkpoints, agentes, equipes, runs e tasks, mascarando corrupção e alterando a semântica histórica. Ver [`crates/orbity-storage/src/audit_chain.rs`](../crates/orbity-storage/src/audit_chain.rs#L278), [`crates/orbity-graph/src/checkpoint.rs`](../crates/orbity-graph/src/checkpoint.rs#L193), [`crates/orbity-storage/src/dao/agent.rs`](../crates/orbity-storage/src/dao/agent.rs#L35), [`crates/orbity-storage/src/dao/team.rs`](../crates/orbity-storage/src/dao/team.rs#L67), [`crates/orbity-storage/src/dao/run.rs`](../crates/orbity-storage/src/dao/run.rs#L54) e [`crates/orbity-storage/src/dao/task.rs`](../crates/orbity-storage/src/dao/task.rs#L56).
- **Checkpoint sem predecessor volta ao hash gênese.** Pode esconder lacunas na cadeia. Ver [`crates/orbity-graph/src/checkpoint.rs`](../crates/orbity-graph/src/checkpoint.rs#L101).
- **Falha ao serializar SSE vira string vazia.** Ver [`crates/orbity-server/src/push.rs`](../crates/orbity-server/src/push.rs#L28).
- **Falha ao serializar nós no hash do checkpoint vira string vazia.** Ver [`crates/orbity-graph/src/checkpoint.rs`](../crates/orbity-graph/src/checkpoint.rs#L74).
- **Falha ao serializar Blackboard vira `null`.** Ver [`crates/orbity-graph/src/blackboard.rs`](../crates/orbity-graph/src/blackboard.rs#L108).
- **Log HTTP sem status code é considerado bem-sucedido.** Ver [`crates/orbity-core/src/execution_log.rs`](../crates/orbity-core/src/execution_log.rs#L329).
- **Timestamp OTLP inválido ou ausente vira zero.** Ver [`crates/orbity-telemetry/src/sink.rs`](../crates/orbity-telemetry/src/sink.rs#L60).

### 3.3 Descoberta e configuração

- `orbity sync` usa automaticamente `examples/forester` quando `teams/` ou `agents/` não existem. Ver [`crates/orbity-cli/src/dispatcher.rs`](../crates/orbity-cli/src/dispatcher.rs#L48).
- `orbity run` usa `dev_team`, procura em oito localizações — incluindo exemplos — e, sem `--team`, adiciona qualquer YAML encontrado em `agents/` ou `teams/`. O primeiro válido vence. Ver [`crates/orbity-cli/src/dispatcher.rs`](../crates/orbity-cli/src/dispatcher.rs#L139).
- O parser YAML tenta cinco formatos diferentes; se todos falharem, retorna apenas o erro do primeiro formato. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L94).
- IDs ausentes são derivados de nomes ou índices e steps ausentes viram uma cadeia sequencial. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L144).
- Pipeline ausente em equipe é reconstruído sequencialmente a partir dos workers. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L332).
- Provider ou runner ausente cai em Codex ou Agy dependendo do formato; provider desconhecido vira `CliType::Custom`. Ver [`crates/orbity-graph/src/yaml_loader.rs`](../crates/orbity-graph/src/yaml_loader.rs#L147).
- Tier `latest/default` não fixa modelo e delega ao default local da CLI. É intencional, mas não determinístico. Ver [`crates/orbity-graph/src/model_resolver.rs`](../crates/orbity-graph/src/model_resolver.rs#L64).

### 3.4 Execução e métricas

- Métricas de tokens ausentes ou incompatíveis viram zero; custo ausente também vira zero. Ver [`crates/orbity-agent/src/tokens.rs`](../crates/orbity-agent/src/tokens.rs#L29).
- Se nenhum JSON reconhecido for encontrado, os tokens são estimados por `caracteres / 4` e o custo permanece zero. Ver [`crates/orbity-agent/src/tokens.rs`](../crates/orbity-agent/src/tokens.rs#L71).
- Prompt ausente usa o nome do agente. Ver [`crates/orbity-agent/src/runners.rs`](../crates/orbity-agent/src/runners.rs#L129).
- Falha em localizar Bubblewrap cai em `/usr/bin/bwrap`; `HOME` e `PATH` ausentes recebem valores fixos. Ver [`crates/orbity-sandbox/src/bwrap.rs`](../crates/orbity-sandbox/src/bwrap.rs#L37).
- Processo terminado sem exit code recebe `-1`. É um sentinel razoável, mas precisa estar documentado no contrato. Ver [`crates/orbity-sandbox/src/bwrap.rs`](../crates/orbity-sandbox/src/bwrap.rs#L234).
- Health check considera SQLite saudável quando nenhum pool foi configurado. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L253).
- Erro na consulta de status ao banco é ignorado e devolve runs, tokens e custo como zero. Ver [`crates/orbity-server/src/server.rs`](../crates/orbity-server/src/server.rs#L283).

## 4. Priorização recomendada

1. Tornar governança *fail-closed*: política ausente e fallback desconhecido devem rejeitar ou solicitar intervenção humana.
2. Remover dados simulados dos endpoints de produção ou marcá-los explicitamente como `demo`.
3. Tratar timestamps, hashes e serialização com erro explícito, sem substituir por `now`, vazio ou `null`.
4. Remover `examples/forester` do caminho automático de execução e sincronização.
5. Substituir os catálogos estáticos de `/api/models` e `ModelTierResolver` por descoberta ou configuração externa.
6. Colocar `MockSandbox` e `DefaultNodeRunner` atrás de `cfg(test)` ou de uma feature `test-utils`.
7. Centralizar versão, timeout, budget, retries, paths e configuração do servidor em uma camada tipada única.

## 5. Resumo por risco

| Categoria | Risco principal |
| --- | --- |
| Mocks no runtime | Estados e métricas fictícias podem ser apresentados como dados reais |
| Modelos hardcoded | Obsolescência rápida e incompatibilidade com CLIs instaladas |
| Defaults permissivos | Execução de comandos sem aprovação explícita |
| Fallbacks silenciosos | Corrupção, erros de serialização e indisponibilidade ficam mascarados |
| Exemplos automáticos | Configurações demonstrativas podem ser executadas como produção |
| Configuração duplicada | Divergência entre CLI, servidor, frontend e documentação |
