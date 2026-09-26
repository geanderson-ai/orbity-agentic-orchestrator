use orbity_core::contracts::compute_sha256;
use orbity_storage::dao::agent::{AgentDao, AgentDbRecord};
use orbity_storage::dao::team::{TeamDao, TeamRecord};
use orbity_storage::pool::SqliteStoragePool;

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Storage error: {0}")]
    Storage(#[from] orbity_storage::error::StorageError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Default, Clone)]
pub struct SyncSummary {
    pub teams_created: usize,
    pub teams_updated: usize,
    pub agents_created: usize,
    pub agents_updated: usize,
    pub unchanged: usize,
}

pub struct DeclarativeSync;

impl DeclarativeSync {
    /// Scans a directory of team YAML files (e.g. `examples/forester/teams/`) and syncs to SQLite.
    pub async fn sync_teams(
        pool: &SqliteStoragePool,
        teams_dir: impl AsRef<Path>,
    ) -> Result<SyncSummary, SyncError> {
        let mut summary = SyncSummary::default();
        let team_dao = TeamDao::new(pool.clone());
        let dir = teams_dir.as_ref();

        if !dir.exists() {
            return Ok(summary);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && (path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml"))
            {
                let content = std::fs::read_to_string(&path)?;

                let hash = compute_sha256(&content);
                let team_name = path.file_stem().unwrap().to_string_lossy().to_string();

                let existing = team_dao.get(&team_name).await?;

                if let Some(t) = existing {
                    if t.config_hash == hash {
                        summary.unchanged += 1;
                    } else {
                        let updated = TeamRecord {
                            name: team_name,
                            description: None,
                            config_yaml: content,
                            config_hash: hash,
                            created_at: t.created_at,
                            updated_at: chrono::Utc::now(),
                        };
                        team_dao.upsert(&updated).await?;
                        summary.teams_updated += 1;
                    }
                } else {
                    let new_team = TeamRecord {
                        name: team_name,
                        description: None,
                        config_yaml: content,
                        config_hash: hash,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    };
                    team_dao.upsert(&new_team).await?;
                    summary.teams_created += 1;
                }
            }
        }

        Ok(summary)
    }

    /// Scans a directory of agent YAML files (e.g. `examples/forester/agents/`) and syncs to SQLite.
    pub async fn sync_agents(
        pool: &SqliteStoragePool,
        agents_dir: impl AsRef<Path>,
    ) -> Result<SyncSummary, SyncError> {
        let mut summary = SyncSummary::default();
        let agent_dao = AgentDao::new(pool.clone());
        let dir = agents_dir.as_ref();

        if !dir.exists() {
            return Ok(summary);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && (path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml"))
            {
                let content = std::fs::read_to_string(&path)?;

                let hash = compute_sha256(&content);
                let agent_id = path.file_stem().unwrap().to_string_lossy().to_string();

                let existing = agent_dao.get(&agent_id).await?;

                if let Some(a) = existing {
                    if a.config_hash == hash {
                        summary.unchanged += 1;
                    } else {
                        let updated = AgentDbRecord {
                            id: agent_id.clone(),
                            name: a.name,
                            team_name: a.team_name,
                            state: a.state,
                            orchestrator_model: a.orchestrator_model,
                            prompt_system: a.prompt_system,
                            plan_strategy: a.plan_strategy,
                            plan_json: a.plan_json,
                            finops_budget_usd: a.finops_budget_usd,
                            config_hash: hash,
                            created_at: a.created_at,
                            updated_at: chrono::Utc::now(),
                        };
                        agent_dao.upsert(&updated).await?;
                        summary.agents_updated += 1;
                    }
                } else {
                    let new_agent = AgentDbRecord {
                        id: agent_id.clone(),
                        name: agent_id,
                        team_name: None,
                        state: "Draft".to_string(),
                        orchestrator_model: None,
                        prompt_system: None,
                        plan_strategy: None,
                        plan_json: None,
                        finops_budget_usd: None,
                        config_hash: hash,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    };
                    agent_dao.upsert(&new_agent).await?;
                    summary.agents_created += 1;
                }
            }
        }

        Ok(summary)
    }
}
