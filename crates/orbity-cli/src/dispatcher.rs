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
    pub async fn dispatch(cli: Cli) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        match cli.command {
            Commands::Init(args) => {
                let target_dir = &args.path;
                let created =
                    crate::init::WorkspaceInit::init_project(target_dir, args.name.as_deref())?;
                let canonical = target_dir
                    .canonicalize()
                    .unwrap_or_else(|_| target_dir.clone());
                println!("✨ Initialized Orbity workspace in {}", canonical.display());
                if created.is_empty() {
                    println!("ℹ️  Workspace already contains configuration files. Nothing was overwritten.");
                } else {
                    println!("Created files:");
                    for f in created {
                        println!("  - {}", f.display());
                    }
                }
                println!("\nNext steps:");
                println!("  1. Run `orbity doctor` to check your environment and AI CLIs");
                println!("  2. Run `orbity sync` to synchronize declarative teams into SQLite");
                println!("  3. Run `orbity run \"your task description\"` to launch multi-agent orchestration");
                println!("  4. Run `orbity serve` to start the full-stack Web Dashboard");
                Ok(0)
            }

            Commands::Doctor => {
                let report = PreflightDoctor::check();
                println!("{}", report.summary_text());
                Ok(if report.is_ready() { 0 } else { 1 })
            }

            Commands::Sync(args) => {
                let pool = SqliteStoragePool::connect_file(&cli.db_path).await?;
                pool.run_migrations().await?;

                let teams_path = args.teams_dir.unwrap_or_else(|| {
                    if Path::new("teams").exists() {
                        std::path::PathBuf::from("teams")
                    } else {
                        std::path::PathBuf::from("examples/forester/teams")
                    }
                });

                let agents_path = args.agents_dir.unwrap_or_else(|| {
                    if Path::new("agents").exists() {
                        std::path::PathBuf::from("agents")
                    } else {
                        std::path::PathBuf::from("examples/forester/agents")
                    }
                });

                let teams_summary = DeclarativeSync::sync_teams(&pool, &teams_path).await?;
                let agents_summary = DeclarativeSync::sync_agents(&pool, &agents_path).await?;

                println!("=== Declarative Sync Completed ===");
                println!(
                    "Teams  ({}): {} created, {} updated, {} unchanged",
                    teams_path.display(),
                    teams_summary.teams_created,
                    teams_summary.teams_updated,
                    teams_summary.unchanged
                );
                println!(
                    "Agents ({}): {} created, {} updated, {} unchanged",
                    agents_path.display(),
                    agents_summary.agents_created,
                    agents_summary.agents_updated,
                    agents_summary.unchanged
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

                server.run_server(addr).await?;
                Ok(0)
            }

            Commands::Audit(args) => {
                match args.subcmd {
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
                            AuditVerificationResult::Tampered {
                                event_id,
                                sequence_num,
                                expected_hash,
                                actual_hash,
                                ..
                            } => {
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
                }
            }

            Commands::Run(args) => {
                let exec_id = Uuid::new_v4();
                println!("🚀 Initiating run {}...", exec_id);
                println!("Prompt: {}", args.prompt);

                // Load team or pipeline if specified, or search in teams/ or examples/forester/teams/
                let team_name = args.team.as_deref().unwrap_or("dev_team");
                let search_paths = [
                    format!("teams/{}.yaml", team_name),
                    format!("teams/{}.yml", team_name),
                    format!("{}.yaml", team_name),
                    format!("examples/forester/teams/{}.yaml", team_name),
                    format!("examples/forester/teams/{}.yml", team_name),
                ];

                let mut loaded_graph = None;
                for team_path in &search_paths {
                    if Path::new(team_path).exists() {
                        match GraphYamlLoader::load_file(team_path) {
                            Ok(graph) => {
                                println!(
                                    "Loaded team graph definition from {} (nodes: {})",
                                    team_path,
                                    graph.nodes.len()
                                );
                                loaded_graph = Some(graph);
                                break;
                            }
                            Err(e) => {
                                eprintln!(
                                    "❌ Failed to parse team graph from {}: {}",
                                    team_path, e
                                );
                                return Ok(1);
                            }
                        }
                    }
                }

                let graph = match loaded_graph {
                    Some(g) => g,
                    None => {
                        eprintln!(
                            "❌ Error: Team '{}' was not found in ./teams/ or ./examples/forester/teams/.",
                            team_name
                        );
                        return Ok(1);
                    }
                };

                let blackboard = orbity_graph::blackboard::Blackboard::new();
                blackboard
                    .set_context(
                        "user_prompt",
                        serde_json::Value::String(args.prompt.clone()),
                    )
                    .await;
                let finops = orbity_graph::finops::GraphFinOpsTracker::new(args.budget_usd, None);
                let runner = std::sync::Arc::new(orbity_graph::executor::DefaultNodeRunner);

                let executor =
                    orbity_graph::executor::GraphExecutor::new(graph, blackboard, finops, runner)
                        .with_execution_id(exec_id);

                match executor.execute().await {
                    Ok(()) => {
                        println!("✅ Execution {} finished successfully!", exec_id);
                        Ok(0)
                    }
                    Err(e) => {
                        eprintln!("❌ Execution {} failed: {}", exec_id, e);
                        Ok(1)
                    }
                }
            }

            Commands::Resume(args) => {
                println!(
                    "Resuming execution {} with approve={}, reject={}",
                    args.run_id, args.approve, args.reject
                );
                Ok(0)
            }

            Commands::Status(args) => {
                println!("Checking status for run {}...", args.run_id);
                Ok(0)
            }
        }
    }
}
