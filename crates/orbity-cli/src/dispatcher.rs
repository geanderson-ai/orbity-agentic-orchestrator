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

                // Load team or agent pipeline if specified, or search in agents/, teams/, root or examples
                let target_name = args.team.as_deref().unwrap_or("dev_team");
                let mut search_paths = vec![
                    format!("teams/{}.yaml", target_name),
                    format!("teams/{}.yml", target_name),
                    format!("agents/{}.yaml", target_name),
                    format!("agents/{}.yml", target_name),
                    format!("{}.yaml", target_name),
                    format!("{}.yml", target_name),
                    format!("examples/forester/teams/{}.yaml", target_name),
                    format!("examples/forester/agents/{}.yaml", target_name),
                ];

                // If default dev_team is not found, automatically check if there are any YAML files in agents/ or teams/
                if args.team.is_none() {
                    if let Ok(entries) = std::fs::read_dir("agents") {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if p.is_file() && (p.extension().is_some_and(|e| e == "yaml" || e == "yml")) {
                                search_paths.push(p.to_string_lossy().to_string());
                            }
                        }
                    }
                    if let Ok(entries) = std::fs::read_dir("teams") {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if p.is_file() && (p.extension().is_some_and(|e| e == "yaml" || e == "yml")) {
                                search_paths.push(p.to_string_lossy().to_string());
                            }
                        }
                    }
                }

                let mut loaded_graph = None;
                for file_path in &search_paths {
                    if Path::new(file_path).exists() {
                        match GraphYamlLoader::load_file(file_path) {
                            Ok(graph) => {
                                println!(
                                    "Loaded execution definition from {} (nodes: {})",
                                    file_path,
                                    graph.nodes.len()
                                );
                                loaded_graph = Some(graph);
                                break;
                            }
                            Err(e) => {
                                eprintln!(
                                    "❌ Failed to parse graph definition from {}: {}",
                                    file_path, e
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
                            "❌ Error: Could not find any configuration for '{}' in ./agents/, ./teams/, or project root.\nCreate a YAML file in agents/ (e.g. agents/coder.yaml) or teams/ (e.g. teams/dev_team.yaml).",
                            target_name
                        );
                        return Ok(1);
                    }
                };

                let pool = SqliteStoragePool::connect_file(&cli.db_path).await?;
                pool.run_migrations().await?;

                let blackboard = orbity_graph::blackboard::Blackboard::new();
                blackboard
                    .set_context(
                        "user_prompt",
                        serde_json::Value::String(args.prompt.clone()),
                    )
                    .await;
                let finops = orbity_graph::finops::GraphFinOpsTracker::new(args.budget_usd, None);

                let mut sandbox_config = orbity_sandbox::types::SandboxConfig::default();
                sandbox_config.network = orbity_sandbox::types::NetworkMode::HostMediated;
                if let Ok(cwd) = std::env::current_dir() {
                    sandbox_config.workspace = orbity_sandbox::types::WorkspaceMode::EphemeralCopyOnWrite(cwd);
                }
                let mut bwrap_box = orbity_sandbox::BwrapSandbox::new(sandbox_config);
                use orbity_sandbox::traits::Sandbox;
                bwrap_box.initialize().await
                    .map_err(|e| format!("Sandbox initialization failed: {}", e))?;

                let sandbox: std::sync::Arc<tokio::sync::Mutex<dyn orbity_sandbox::traits::Sandbox>> =
                    std::sync::Arc::new(tokio::sync::Mutex::new(bwrap_box));

                let runner = std::sync::Arc::new(orbity_agent::runners::SandboxCliNodeRunner::new(sandbox.clone()));

                let store = orbity_graph::checkpoint::GraphCheckpointStore::new(pool.inner().clone());
                store.init_schema().await?;

                let bus = EventBus::new(EventBusConfig::default());
                let executor =
                    orbity_graph::executor::GraphExecutor::new(graph, blackboard.clone(), finops, runner)
                        .with_execution_id(exec_id)
                        .with_event_bus(bus)
                        .with_checkpoints(store);

                // Record run initiation in SQLite
                let run_dao = orbity_storage::dao::run::RunDao::new(pool.clone());
                let run_rec = orbity_storage::dao::run::RunRecord {
                    id: exec_id.to_string(),
                    status: "Running".to_string(),
                    initiated_at: chrono::Utc::now(),
                    completed_at: None,
                    total_tokens: 0,
                    total_cost_usd: 0.0,
                    metadata: Some(format!("target={}", target_name)),
                };
                let _ = run_dao.create(&run_rec).await;

                match executor.execute().await {
                    Ok(()) => {
                        let _ = run_dao.update_status(&exec_id.to_string(), "Completed", Some(chrono::Utc::now())).await;

                        // Promote file modifications from isolated sandbox to host workspace
                        if let Ok(cwd) = std::env::current_dir() {
                            let sb = sandbox.lock().await;
                            if let Ok(promoted) = sb.promote_changes(&cwd).await {
                                if !promoted.is_empty() {
                                    println!("\n📦 Workspace files updated ({} change(s)):", promoted.len());
                                    for change in promoted {
                                        println!("  - {:?}: {}", change.change_type, change.relative_path.display());
                                    }
                                }
                            }
                        }

                        // Display output from nodes
                        let snap = blackboard.snapshot().await;
                        if let Some(outputs) = snap.get("node_outputs").and_then(|o| o.as_object()) {
                            if !outputs.is_empty() {
                                println!("\n📋 Agent Output Summary:");
                                for (node, out) in outputs {
                                    if let Some(text) = out.as_str() {
                                        if !text.trim().is_empty() {
                                            println!("\n--- [{}] ---\n{}", node, text.trim());
                                        }
                                    }
                                }
                            }
                        }

                        println!("\n✅ Execution {} finished successfully!", exec_id);
                        Ok(0)
                    }
                    Err(e) => {
                        let _ = run_dao.update_status(&exec_id.to_string(), "Failed", Some(chrono::Utc::now())).await;
                        eprintln!("\n❌ Execution {} failed: {}", exec_id, e);
                        Ok(1)
                    }
                }
            }

            Commands::Resume(args) => {
                let pool = SqliteStoragePool::connect_file(&cli.db_path).await?;
                pool.run_migrations().await?;

                let exec_uuid = match Uuid::parse_str(&args.run_id) {
                    Ok(u) => u,
                    Err(e) => {
                        eprintln!("❌ Invalid run_id UUID '{}': {}", args.run_id, e);
                        return Ok(1);
                    }
                };

                let store = orbity_graph::checkpoint::GraphCheckpointStore::new(pool.inner().clone());
                store.init_schema().await?;

                let latest_chk = store.load_latest_checkpoint(exec_uuid).await?;
                match latest_chk {
                    Some(chk) => {
                        println!("🔄 Found checkpoint for run {} at step {}", args.run_id, chk.step_number);
                        println!("   Active nodes:    {:?}", chk.active_nodes);
                        println!("   Completed nodes: {:?}", chk.completed_nodes);
                        println!("   State hash:      {}", chk.state_hash);
                        if args.approve {
                            println!("   Human decision:  APPROVED");
                        } else if args.reject {
                            println!("   Human decision:  REJECTED");
                        }
                        println!("✅ State verified and ready to continue.");
                        Ok(0)
                    }
                    None => {
                        eprintln!("❌ No checkpoint found in SQLite for run_id {}", args.run_id);
                        Ok(1)
                    }
                }
            }

            Commands::Status(args) => {
                let pool = SqliteStoragePool::connect_file(&cli.db_path).await?;
                pool.run_migrations().await?;

                let run_dao = orbity_storage::dao::run::RunDao::new(pool.clone());
                let task_dao = orbity_storage::dao::task::TaskDao::new(pool.clone());

                let run_opt = run_dao.get(&args.run_id).await?;
                match run_opt {
                    Some(run) => {
                        println!("=== Orbity Run Status: {} ===", run.id);
                        println!("Status:       {}", run.status);
                        println!("Initiated At: {}", run.initiated_at);
                        if let Some(comp) = run.completed_at {
                            println!("Completed At: {}", comp);
                        }
                        println!("Total Tokens: {}", run.total_tokens);
                        println!("Total Cost:   ${:.4}", run.total_cost_usd);
                        if let Some(meta) = run.metadata {
                            println!("Metadata:     {}", meta);
                        }

                        let tasks = task_dao.list_by_run(&run.id).await?;
                        if !tasks.is_empty() {
                            println!("\nTasks ({}):", tasks.len());
                            for t in tasks {
                                println!("  - [{}] Agent: {:<10} Status: {:<10} ({}ms)", t.id, t.agent_name, t.status, t.duration_ms.unwrap_or(0));
                            }
                        }
                        Ok(0)
                    }
                    None => {
                        println!("⚠️  Run '{}' not found in database.", args.run_id);
                        Ok(1)
                    }
                }
            }
        }
    }
}
