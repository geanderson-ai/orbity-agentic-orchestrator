use orbity_cli::commands::{Cli, Commands, ServeArgs};
use orbity_cli::doctor::PreflightDoctor;
use orbity_cli::sync::DeclarativeSync;
use orbity_storage::pool::SqliteStoragePool;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;


#[tokio::test]
async fn test_cli_preflight_doctor_check() {
    let report = PreflightDoctor::check();
    // Verify report structure
    assert_eq!(report.agents.len(), 5);
    let names: Vec<&str> = report.agents.iter().map(|a| a.name).collect();
    assert_eq!(names, vec!["codex", "claude", "agy", "hermes", "pi"]);

    let text = report.summary_text();
    assert!(text.contains("Orbity Preflight Health Check"));
    assert!(text.contains("codex"));
    assert!(text.contains("claude"));
    assert!(text.contains("agy"));
    assert!(text.contains("hermes"));
    assert!(text.contains("pi"));
}

#[tokio::test]
async fn test_cli_declarative_folder_sync_and_hash_reconciliation() {
    let pool = SqliteStoragePool::connect_in_memory().await.unwrap();
    pool.run_migrations().await.unwrap();

    let temp_base = std::env::temp_dir().join(format!("orbity_sync_test_{}", Uuid::new_v4()));
    let teams_dir = temp_base.join("teams");
    std::fs::create_dir_all(&teams_dir).unwrap();


    // Create a mock team file
    let team_yaml = r#"
name: "finance_team"
start_node: "planner"
nodes:
  - id: "planner"
    type: "orchestrator"
edges: []
"#;
    let file_path = teams_dir.join("finance_team.yaml");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(team_yaml.as_bytes()).unwrap();

    // First sync: team created
    let summary1 = DeclarativeSync::sync_teams(&pool, &teams_dir).await.unwrap();
    assert_eq!(summary1.teams_created, 1);
    assert_eq!(summary1.teams_updated, 0);
    assert_eq!(summary1.unchanged, 0);

    // Second sync without changes: team unchanged
    let summary2 = DeclarativeSync::sync_teams(&pool, &teams_dir).await.unwrap();
    assert_eq!(summary2.teams_created, 0);
    assert_eq!(summary2.teams_updated, 0);
    assert_eq!(summary2.unchanged, 1);

    // Modify file: team updated
    let modified_yaml = format!("{}\n# Comment change\n", team_yaml);
    std::fs::write(&file_path, modified_yaml).unwrap();

    let summary3 = DeclarativeSync::sync_teams(&pool, &teams_dir).await.unwrap();
    assert_eq!(summary3.teams_created, 0);
    assert_eq!(summary3.teams_updated, 1);
    assert_eq!(summary3.unchanged, 0);
}

#[tokio::test]
async fn test_cli_command_dispatch_serve_and_doctor() {
    let doctor_cli = Cli {
        command: Commands::Doctor,
        db_path: "test.db".into(),
        output: "text".into(),
    };

    let res = orbity_cli::CommandDispatcher::dispatch(doctor_cli).await;
    assert!(res.is_ok());

    let serve_cli = Cli {
        command: Commands::Serve(ServeArgs {
            port: 3099,
            host: "127.0.0.1".into(),
        }),
        db_path: "test.db".into(),
        output: "text".into(),
    };

    let serve_res = orbity_cli::CommandDispatcher::dispatch(serve_cli).await;
    assert!(serve_res.is_ok());
    assert_eq!(serve_res.unwrap(), 0);
}
