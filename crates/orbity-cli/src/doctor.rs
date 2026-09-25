//! Preflight health check detecting bwrap and the 5 native CLI agents in PATH.

use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliDetectionResult {
    pub name: &'static str,
    pub installed: bool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PreflightReport {
    pub bwrap_available: bool,
    pub bwrap_path: Option<PathBuf>,
    pub agents: Vec<CliDetectionResult>,
}

impl PreflightReport {
    pub fn is_ready(&self) -> bool {
        // Platform is ready if sandbox is present and at least one CLI agent is installed
        self.bwrap_available && self.agents.iter().any(|a| a.installed)
    }

    pub fn summary_text(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Orbity Preflight Health Check ===\n");
        let bwrap_status = if self.bwrap_available {
            format!("✅ Available ({})", self.bwrap_path.as_ref().map(|p| p.display().to_string()).unwrap_or_default())
        } else {
            "❌ Missing (Bubblewrap /usr/bin/bwrap not found)".to_string()
        };
        out.push_str(&format!("Sandbox Provider (bwrap): {}\n\n", bwrap_status));

        out.push_str("CLI Agents Detection:\n");
        for agent in &self.agents {
            let status = if agent.installed {
                format!("✅ Installed ({}) - Version: {}", agent.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default(), agent.version.as_deref().unwrap_or("unknown"))
            } else {
                "⚠️ Not found in PATH".to_string()
            };
            out.push_str(&format!("  - {:<8}: {}\n", agent.name, status));
        }

        out
    }
}

pub struct PreflightDoctor;

impl PreflightDoctor {
    /// Runs complete preflight check of system tools and CLI agents.
    pub fn check() -> PreflightReport {
        let bwrap_path = Self::find_binary("bwrap");
        let bwrap_available = bwrap_path.is_some();

        let agents = vec![
            Self::check_agent("codex", &["--version", "version", "-v"]),
            Self::check_agent("claude", &["--version", "-v"]),
            Self::check_agent("agy", &["--version", "-v"]),
            Self::check_agent("hermes", &["--version", "-v"]),
            Self::check_agent("pi", &["--version", "-v"]),
        ];

        PreflightReport {
            bwrap_available,
            bwrap_path,
            agents,
        }
    }

    fn find_binary(name: &str) -> Option<PathBuf> {
        let path_var = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    fn check_agent(name: &'static str, version_args: &[&str]) -> CliDetectionResult {
        if let Some(path) = Self::find_binary(name) {
            let mut version = None;
            for arg in version_args {
                if let Ok(output) = Command::new(&path).arg(arg).output() {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !text.is_empty() {
                            version = Some(text);
                            break;
                        }
                    }
                }
            }
            CliDetectionResult {
                name,
                installed: true,
                path: Some(path),
                version,
            }
        } else {
            CliDetectionResult {
                name,
                installed: false,
                path: None,
                version: None,
            }
        }
    }
}
