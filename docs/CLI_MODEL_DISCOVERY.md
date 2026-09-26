# Guia Canônico de Descoberta de Modelos e Flags das CLIs

Este documento descreve como descobrir, inspecionar e configurar modelos de inteligência artificial em cada uma das 5 CLIs suportadas nativamente pelo **Orbity Multi Agentic Harness**:

1. **Claude Code** (`claude`)
2. **OpenAI Codex** (`codex`)
3. **Pi Coding Agent** (`pi`)
4. **Hermes Agent** (`hermes`)
5. **Antigravity CLI** (`agy`)

Também apresenta a estratégia do Orbity para **evitar obsolescência nos arquivos YAML** através da arquitetura de **Tiers Semânticos** (`fast`, `balanced`, `reasoning`, `latest`).

---

## 1. Comandos de Descoberta de Modelos Disponíveis

Cada CLI possui seu próprio mecanismo para verificar quais modelos estão disponíveis e ativos no ambiente local:

```
┌─────────────┬─────────────────────────────────────┬──────────────────────────────────────────┐
│ CLI Engine  │ Comando de Descoberta em Tempo Real │ Onde Fica o Catálogo / Configuração      │
├─────────────┼─────────────────────────────────────┼──────────────────────────────────────────┤
│ agy         │ agy models                          │ ~/.gemini/antigravity-cli/settings.json  │
│ pi          │ pi --list-models                    │ Catálogo dinâmico + /model na TUI        │
│ hermes      │ hermes model                        │ ~/.hermes/config.yaml                    │
│ claude      │ claude --help (seção --model)       │ ~/.claude.json / ~/.claude/settings.json │
│ codex       │ codex --help (seção --model)        │ ~/.codex/config.toml                     │
└─────────────┴─────────────────────────────────────┴──────────────────────────────────────────┘
```

---

## 2. Detalhamento por CLI

### 2.1. Claude Code CLI (`claude`)
* **Provedor:** Anthropic
* **Comando de Descoberta:**
  ```bash
  claude --help
  ```
  *(ou executar `/model` dentro de uma sessão interativa).*
* **Flag de Modelo:**
  ```bash
  claude -p "prompt" --model <MODEL_ALIAS> --output-format json
  ```
* **Aliases Suportados:**
  - `sonnet`: Aponta automaticamente para a geração mais recente do Sonnet (ex: `claude-3-7-sonnet`, `claude-sonnet-4-6`).
  - `opus`: Modelo de raciocínio profundo.
  - `haiku`: Modelo ultrarrápido e econômico.
* **Flag de Fallback:**
  `--fallback-model <model>` (tenta modelo alternativo se o primário estiver indisponível).
* **Variável de Ambiente:** `CLAUDE_DEFAULT_MODEL`.

---

### 2.2. OpenAI Codex CLI (`codex`)
* **Provedor:** OpenAI
* **Comando de Descoberta:**
  ```bash
  codex --help
  # ou inspecionar o arquivo de configuração
  cat ~/.codex/config.toml
  ```
* **Flag de Modelo:**
  ```bash
  codex exec -m <MODEL> --json "prompt"
  # ou via flag longa
  codex exec --model <MODEL> --json "prompt"
  ```
* **Configuração Inline:**
  `-c model="o3-mini"` permite passar qualquer parâmetro TOML na chamada.
* **Modo Open-Source Local:**
  `--oss` redireciona a inferência para vLLM / Ollama local.
* **Variável de Ambiente:** `CODEX_MODEL`.

---

### 2.3. Pi Coding Agent (`pi`)
* **Provedor:** Multi-provedor (Google, Anthropic, OpenAI, Llama.cpp)
* **Comando de Descoberta:**
  ```bash
  pi --list-models
  # Atualizar o catálogo de modelos
  pi update --models
  ```
* **Flag de Modelo:**
  ```bash
  pi -p "prompt" --mode json --model <pattern> --no-session
  ```
* **Sintaxe de Provedor e Raciocínio (*Thinking*):**
  - Prefixo de provedor: `--model openai/gpt-4o` ou `--provider google --model gemini-2.5-pro`
  - Shorthand de thinking: `--model sonnet:high`
  - Flag de thinking dedicada: `--thinking <low|medium|high>`
* **Ciclagem de Modelos:** `--models claude-sonnet,gpt-4o`.

---

### 2.4. Hermes Agent CLI (`hermes`)
* **Provedor:** Nous Research (OpenRouter, Anthropic, Nous Portal)
* **Comando de Descoberta:**
  ```bash
  hermes model
  ```
  *(Abre seletor interativo com busca ao vivo dos modelos configurados).*
* **Flag de Modelo:**
  ```bash
  hermes -m <MODEL> run --json "prompt"
  # ou
  hermes --model <MODEL> run --json "prompt"
  ```
* **Controle de Raciocínio e Métricas:**
  - `--reasoning <none|minimal|low|medium|high|xhigh|max|ultra>`
  - `--usage-file <PATH>`: Gera relatório detalhado de custo e tokens após a execução.
* **Variáveis de Ambiente:** `HERMES_INFERENCE_MODEL` e `HERMES_INFERENCE_PROVIDER`.

---

### 2.5. Antigravity CLI (`agy`)
* **Provedor:** Google DeepMind / Antigravity System
* **Comando de Descoberta em Tempo Real:**
  ```bash
  agy models
  ```
  *Exemplo de saída:*
  ```text
  gemini-3.8-flash-high     Gemini 3.8 Flash (High)
  gemini-3.8-flash-medium   Gemini 3.8 Flash (Medium)
  gemini-3.8-flash-low      Gemini 3.8 Flash (Low)
  gemini-3.1-pro-high       Gemini 3.1 Pro (High)
  claude-sonnet-4-6         Claude Sonnet 4.6 (Thinking)
  gpt-oss-120b-medium       GPT-OSS 120B (Medium)
  ```
* **Flag de Modelo:**
  ```bash
  agy -p "prompt" --output-format json --model <MODEL>
  ```
* **Esforço de Raciocínio:**
  `--effort <low|medium|high|max>`.
* **Modo Operacional:** `--mode <plan|accept-edits>`.

---

## 3. Arquitetura de Tiers Semânticos no Orbity

Para evitar que os arquivos YAML quebrem quando versões específicas de modelos forem descontinuadas pelas provedoras, o Orbity adota a estrutura declarativa:

```yaml
agent:
  name: "codex_coder"
  provider: "codex"
  tier: "balanced"
```

### Matriz de Resolução de Tiers Semânticos

| Tier Semântico | Perfil de Custo / Performance | Resolução Claude | Resolução Codex | Resolução Agy | Resolução Hermes | Resolução Pi |
|---|---|---|---|---|---|---|
| **`fast`** | Baixo custo, alta frequência (lints, testes rápidos) | `haiku` | `gpt-4o-mini` | `gemini-3.8-flash-low` | `openrouter/auto-fast` | `llama-cpp` |
| **`balanced`** | Desenvolvimento, refatoração de código, features | `sonnet` | `gpt-4o` | `gemini-3.8-flash-high` | `anthropic/claude-sonnet-4.6` | `sonnet` |
| **`reasoning`** | Raciocínio profundo, arquitetura e auditoria | `opus` / `thinking` | `o3-mini` | `gemini-3.1-pro-high` | `anthropic/claude-sonnet-4.6` (`--reasoning high`) | `sonnet:high` |
| **`latest`** | Usa o padrão mais recente da instalação local | *(default da CLI)* | *(default da CLI)* | *(default da CLI)* | *(default da CLI)* | *(default da CLI)* |

> Caso o usuário declare um nome exato de modelo (ex: `tier: "o3-mini"` ou `model: "gemini-3.8-flash-high"`), o Orbity identifica o valor específico e injeta a flag canônica da CLI correspondente.
