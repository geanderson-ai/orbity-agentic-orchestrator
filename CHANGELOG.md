# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-09-25

### Added
- **Core Architecture:**
  - Multi-crate modular architecture in Rust (`orbity-core`, `orbity-storage`, `orbity-sandbox`, `orbity-graph`, `orbity-agent`, `orbity-telemetry`, `orbity-server`, `orbity-cli`).
  - Event Bus with async broadcast and JSONL streaming sinks.
  - Four-Layer Observability architecture (Runtime Logs, Execution Logs, Audit Trail, OpenTelemetry Spans & Metrics).
- **Execution & Topology Engine:**
  - Kahn's topological sort and DAG DAG validation engine.
  - Tokio parallel fan-out and join barrier fan-in synchronization.
  - Controlled feedback loops with max-iteration guardrails.
  - Shared Blackboard memory context passing between nodes.
- **Security & Sandboxing:**
  - Bubblewrap (`bwrap`) confinement with read-only root mount, isolated tmpfs workspace, unshared network namespaces, and deterministic SIGKILL timeouts.
  - Atomic workspace promotion and snapshot rollback capabilities.
- **Persistence & Auditability:**
  - SQLite 3 with WAL mode, foreign keys, and automatic schema migrations.
  - Cryptographic SHA-256 tamper-evident hash chaining across all execution events.
  - Verification engine detecting payload tampering or out-of-order state mutations.
- **FinOps Guardrails:**
  - Token tracking across Input, Output, Cache Read, and Reasoning tokens.
  - Real-time USD budget capping with tripwire circuit breakers.
- **Server & Web Application:**
  - Tokio Topcoat reactive server with server-rendered views, live progress streaming, and WebSocket push capability.
  - Real-time HTTP dashboard endpoints (`/`, `/health`, `/governance`, `/finops`).
- **CLI & Developer Experience:**
  - `orbity init`: Automatic scaffolding of Orbity workspaces in any folder.
  - `orbity doctor`: Comprehensive preflight diagnostic tool for sandboxes and the 5 AI CLIs (`claude`, `codex`, `pi`, `hermes`, `agy`).
  - `orbity sync`: Declarative YAML reconciliation with SHA-256 hash detection.
  - `orbity run`: Autonomous multi-agent pipeline execution.
  - `orbity resume` & `orbity status`: Human-in-the-Loop governance controls.
  - `orbity audit verify`: Cryptographic verification of audit ledgers.
  - `setup.sh`: Automated, cross-platform bootstrap script with colorized progress logs and `--no-shell-edit` support.
- **Open Source Governance:**
  - Open source licenses (`LICENSE-MIT`, `LICENSE-APACHE`).
  - Community guidelines (`CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`).
  - GitHub Actions CI workflow for format, clippy, and test validation.
