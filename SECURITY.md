# Security Policy

## Supported Versions

Orbity follows semantic versioning. Critical security patches are backported to the latest minor release line.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

---

## Reporting a Vulnerability

The Orbity team takes security vulnerabilities seriously. We ask that you report vulnerabilities privately before disclosing them publicly.

### How to Report
- **Email:** Send a report to [security@meza.ai](mailto:security@meza.ai).
- **Details to Include:**
  - Description of the vulnerability and attack vector.
  - Minimal reproducible example or proof-of-concept script.
  - Affected versions and environments (OS, kernel version, bubblewrap version).
  - Potential impact on sandbox confinement, audit trail integrity, or host filesystem.

### Response Timeline
- **Initial Response:** Within 48 hours acknowledging receipt.
- **Triage & Remediation Plan:** Within 5 business days.
- **Coordinated Disclosure:** Security patch release coordinated with reporter.

---

## Security Invariants

Orbity enforces the following core security invariants:
1. **Host Filesystem Isolation:** Sandboxed executions run in unprivileged namespaces with root mounted read-only (`ro-bind`) and temporary writes isolated in `tmpfs`.
2. **Network Confinement:** Host loopback and external network access are blocked by default (`--unshare-net`) unless explicit egress allowlisting is enabled.
3. **Cryptographic Immutability:** Audit records in SQLite are sealed with SHA-256 hash chains to prevent undetected tampering.
4. **Secret Redaction:** Tokens, OpenAI keys, and registered secrets are masked with HMAC-SHA-256 hashes in all public logs and telemetry spans.
