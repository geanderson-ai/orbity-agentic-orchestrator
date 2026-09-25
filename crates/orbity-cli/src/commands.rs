//! Command-line argument parser definitions using `clap` (v4 Derive).

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "orbity", author = "Meza Agentic Team", version, about = "Orbity Multi-Agent Agentic Platform CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Database SQLite path
    #[arg(long, global = true, default_value = "orbity.db")]
    pub db_path: PathBuf,

    /// Output format (text or json)
    #[arg(long, global = true, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Inicia uma nova orquestração multi-agente
    Run(RunArgs),

    /// Retoma uma execução de grafo a partir do último checkpoint no SQLite
    Resume(ResumeArgs),

    /// Exibe o status de uma execução
    Status(StatusArgs),

    /// Consulta a trilha de auditoria e verifica integridade criptográfica
    Audit(AuditArgs),

    /// Inicia o servidor Tokio Topcoat full-stack
    Serve(ServeArgs),

    /// Executa preflight health check de ferramentas do sistema e agentes
    Doctor,

    /// Sincroniza arquivos YAML declarativos nas pastas teams/ e agents/
    Sync(SyncArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// O prompt ou instrução da tarefa a ser executada
    pub prompt: String,

    /// Equipe a ser utilizada (arquivo YAML na pasta teams/)
    #[arg(short, long)]
    pub team: Option<String>,

    /// Teto financeiro orçamentário em USD
    #[arg(long)]
    pub budget_usd: Option<f64>,

    /// Modo de sandbox (isolated, allowlist, direct)
    #[arg(long, default_value = "isolated")]
    pub sandbox: String,

    /// Se ativo, desativa interrupções manuais e executa autonomamente
    #[arg(long)]
    pub auto_approve: bool,
}

#[derive(Debug, Args)]
pub struct ResumeArgs {
    /// UUID da execução a ser retomada
    pub run_id: String,

    /// Aprovar o passo pendente do Human-in-the-Loop
    #[arg(long)]
    pub approve: bool,

    /// Rejeitar o passo pendente
    #[arg(long)]
    pub reject: bool,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// UUID da execução
    pub run_id: String,
}

#[derive(Debug, Args)]
pub struct AuditArgs {
    #[command(subcommand)]
    pub subcmd: AuditSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum AuditSubcommand {
    /// Verifica a cadeia criptográfica de hashes SHA-256 de uma execução
    Verify { run_id: String },
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Porta TCP de binding
    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    /// Endereço IP de binding
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Diretório de arquivos de equipes YAML
    #[arg(long, default_value = "examples/teams")]
    pub teams_dir: PathBuf,

    /// Diretório de arquivos de agentes YAML
    #[arg(long, default_value = "examples/agents")]
    pub agents_dir: PathBuf,
}
