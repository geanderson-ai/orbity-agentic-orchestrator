//! Data Access Objects (DAOs) for runs, tasks, tokens, teams, and agents.

pub mod agent;
pub mod run;
pub mod task;
pub mod team;
pub mod token_ledger;

pub use agent::{AgentDao, AgentDbRecord};
pub use run::{RunDao, RunRecord};
pub use task::{TaskDao, TaskRecord};
pub use team::{TeamDao, TeamRecord};
pub use token_ledger::{TokenLedgerDao, TokenLedgerRecord, TokenUsageSummary};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::SqliteStoragePool;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_daos_crud_lifecycle() {
        let pool = SqliteStoragePool::connect_in_memory().await.unwrap();

        let run_dao = RunDao::new(pool.clone());
        let task_dao = TaskDao::new(pool.clone());
        let ledger_dao = TokenLedgerDao::new(pool.clone());
        let team_dao = TeamDao::new(pool.clone());
        let agent_dao = AgentDao::new(pool.clone());

        // 1. Team CRUD
        let team = TeamRecord {
            name: "forester".to_string(),
            description: Some("Forester Team".to_string()),
            config_yaml: "version: '1.0'".to_string(),
            config_hash: "abcd1234hash".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        team_dao.upsert(&team).await.unwrap();
        let loaded_team = team_dao.get("forester").await.unwrap().expect("team found");
        assert_eq!(loaded_team.name, "forester");

        // 2. Agent CRUD
        let agent = AgentDbRecord {
            id: "agt_01h89x2k".to_string(),
            name: "Agente01".to_string(),
            team_name: Some("forester".to_string()),
            state: "Idle".to_string(),
            orchestrator_model: Some("agy".to_string()),
            prompt_system: Some("System prompt".to_string()),
            plan_strategy: Some("Sequential".to_string()),
            plan_json: None,
            finops_budget_usd: Some(2.0),
            config_hash: "hash_agt_1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        agent_dao.upsert(&agent).await.unwrap();
        let loaded_agent = agent_dao.get("agt_01h89x2k").await.unwrap().expect("agent found");
        assert_eq!(loaded_agent.name, "Agente01");
        assert_eq!(loaded_agent.state, "Idle");

        agent_dao.update_state("agt_01h89x2k", "Executing").await.unwrap();
        let updated_agent = agent_dao.get("agt_01h89x2k").await.unwrap().unwrap();
        assert_eq!(updated_agent.state, "Executing");

        // 3. Run CRUD
        let run_id = "run-dao-01".to_string();
        let run = RunRecord {
            id: run_id.clone(),
            status: "Running".to_string(),
            initiated_at: Utc::now(),
            completed_at: None,
            total_tokens: 0,
            total_cost_usd: 0.0,
            metadata: Some("{}".to_string()),
        };
        run_dao.create(&run).await.unwrap();
        let loaded_run = run_dao.get(&run_id).await.unwrap().expect("run found");
        assert_eq!(loaded_run.status, "Running");

        // 4. Task CRUD
        let task_id = format!("task-{}", Uuid::new_v4());
        let task = TaskRecord {
            id: task_id.clone(),
            run_id: run_id.clone(),
            parent_task_id: None,
            agent_name: "codex".to_string(),
            status: "Pending".to_string(),
            input_prompt: Some("Write test".to_string()),
            output_result: None,
            duration_ms: None,
            created_at: Utc::now(),
        };
        task_dao.create(&task).await.unwrap();
        task_dao.update_status(&task_id, "Completed", Some("Success"), Some(250)).await.unwrap();
        let loaded_task = task_dao.get(&task_id).await.unwrap().expect("task found");
        assert_eq!(loaded_task.status, "Completed");
        assert_eq!(loaded_task.duration_ms, Some(250));

        // 5. Token Ledger CRUD & Summary
        let ledger1 = TokenLedgerRecord {
            id: Uuid::new_v4().to_string(),
            run_id: run_id.clone(),
            task_id: Some(task_id.clone()),
            agent_name: "codex".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cached_tokens: 20,
            reasoning_tokens: 10,
            cost_usd: 0.005,
            recorded_at: Utc::now(),
        };
        let ledger2 = TokenLedgerRecord {
            id: Uuid::new_v4().to_string(),
            run_id: run_id.clone(),
            task_id: Some(task_id),
            agent_name: "claude".to_string(),
            input_tokens: 200,
            output_tokens: 100,
            cached_tokens: 50,
            reasoning_tokens: 0,
            cost_usd: 0.010,
            recorded_at: Utc::now(),
        };
        ledger_dao.record_usage(&ledger1).await.unwrap();
        ledger_dao.record_usage(&ledger2).await.unwrap();

        let run_summary = ledger_dao.get_run_summary(&run_id).await.unwrap();
        assert_eq!(run_summary.records_count, 2);
        assert_eq!(run_summary.total_tokens, 530);
        assert!((run_summary.total_cost_usd - 0.015).abs() < 1e-6);

        let codex_summary = ledger_dao.get_agent_summary(&run_id, "codex").await.unwrap();
        assert_eq!(codex_summary.records_count, 1);
        assert_eq!(codex_summary.total_tokens, 180);

        // Update run with final tokens and cost
        run_dao.update_tokens_and_cost(&run_id, run_summary.total_tokens, run_summary.total_cost_usd).await.unwrap();
        run_dao.update_status(&run_id, "Completed", Some(Utc::now())).await.unwrap();

        let completed_run = run_dao.get(&run_id).await.unwrap().unwrap();
        assert_eq!(completed_run.status, "Completed");
        assert_eq!(completed_run.total_tokens, 530);
    }
}
