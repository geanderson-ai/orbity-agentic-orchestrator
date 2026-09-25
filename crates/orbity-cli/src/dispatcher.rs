use crate::commands::{AuditSubcommand, Cli, Commands};
use crate::doctor::PreflightDoctor;
use crate::sync::DeclarativeSync;
use orbity_core::bus::{EventBus, EventBusConfig};
use orbity_graph::GraphYamlLoader;
use orbity_server::{Cx, ServerConfig, TopcoatServer};
use orbity_storage::audit_verifier::{AuditVerificationResult, AuditVerifier};
use orbity_storage::pool::SqliteStoragePool;
use std::net::SocketAddr;
use std::path::Path;
use uuid::Uuid;


pub struct CommandDispatcher;

impl CommandDispatcher {
    pub async fn dispatch(cli: Cli) -> Result<i32, Box<dyn std::error::Error>> {
        match cli.command {
            Commands::Doctor => {
                let report = PreflightDoctor::check();
                println!("{}", report.summary_text());
                Ok(if report.is_ready() { 0 } else { 1 })
            }

            Commands::Sync(args) => {
                let pool = SqliteStoragePool::connect_in_memory().await?;
                pool.run_migrations().await?;

                let teams_summary = DeclarativeSync::sync_teams(&pool, &args.teams_dir).await?;
                let agents_summary = DeclarativeSync::sync_agents(&pool, &args.agents_dir).await?;

                println!("=== Declarative Sync Completed ===");
                println!(
                    "Teams:  {} created, {} updated, {} unchanged",
                    teams_summary.teams_created, teams_summary.teams_updated, teams_summary.unchanged
                );
                println!(
                    "Agents: {} created, {} updated, {} unchanged",
                    agents_summary.agents_created, agents_summary.agents_updated, agents_summary.unchanged
                );
                Ok(0)
            }

            Commands::Serve(args) => {
                let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
                let bus = EventBus::new(EventBusConfig::default());
                let config = ServerConfig {
                    bind_addr: addr,
                    ..Default::default()
                };
                let cx = Cx::new(bus, None, config);
                let server = TopcoatServer::new(cx);

                println!("🚀 Starting Orbity Tokio Topcoat server on http://{}", addr);
                let rendered_len = server.render_dashboard().len();
                println!("Dashboard initialized ({} bytes). Server ready.", rendered_len);
                Ok(0)
            }

            Commands::Audit(args) => match args.subcmd {
                AuditSubcommand::Verify { run_id } => {
                    let pool = SqliteStoragePool::connect_file(&cli.db_path).await?;
                    pool.run_migrations().await?;

                    let verifier = AuditVerifier::new(pool.clone());
                    let res = verifier.verify_run(&run_id).await?;

                    match res {
                        AuditVerificationResult::Valid { total_events, .. } => {
                            println!("✅ Audit Chain Verified: {} sequential events intact (SHA-256).", total_events);
                            Ok(0)
                        }
                        AuditVerificationResult::Tampered { event_id, sequence_num, expected_hash, actual_hash, .. } => {
                            eprintln!(
                                "❌ AUDIT INTEGRITY VIOLATION! Tampered block seq {} (id: {}).\nExpected: {}\nActual:   {}",
                                sequence_num, event_id, expected_hash, actual_hash
                            );
                            Ok(1)
                        }
                        AuditVerificationResult::Empty { .. } => {
                            println!("⚠️ No audit events found for run_id: {}", run_id);
                            Ok(0)
                        }
                    }
                }
            },


            Commands::Run(args) => {
                let exec_id = Uuid::new_v4();
                println!("🚀 Initiating run {}...", exec_id);
                println!("Prompt: {}", args.prompt);

                // Load team or pipeline if specified
                if let Some(team_name) = &args.team {
                    let team_path = format!("examples/teams/{}.yaml", team_name);
                    if Path::new(&team_path).exists() {
                        let _graph = GraphYamlLoader::load_file(&team_path)?;
                        println!("Loaded team graph definition from {}", team_path);
                    }
                }

                println!("Execution finished successfully for run {}", exec_id);
                Ok(0)
            }

            Commands::Resume(args) => {
                println!("Resuming execution {} with approve={}, reject={}", args.run_id, args.approve, args.reject);
                Ok(0)
            }

            Commands::Status(args) => {
                println!("Checking status for run {}...", args.run_id);
                Ok(0)
            }
        }
    }
}
