//! Agent CRUD, lifecycle states and database synchronization manager.

use orbity_core::contracts::AgentLifecycleState;
use orbity_storage::dao::agent::{AgentDao, AgentDbRecord};
use orbity_storage::pool::SqliteStoragePool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentManagerError {
    #[error("Storage error: {0}")]
    Storage(#[from] orbity_storage::error::StorageError),
    #[error("Agent not found: {0}")]
    NotFound(String),
}

/// Manages agent persistence, lifecycle state transitions and CRUD operations.
pub struct AgentManager {
    agent_dao: AgentDao,
}

impl AgentManager {
    pub fn new(pool: SqliteStoragePool) -> Self {
        Self {
            agent_dao: AgentDao::new(pool),
        }
    }

    /// Registers or updates an agent in SQLite storage.
    pub async fn create_or_update(&self, agent: &AgentDbRecord) -> Result<(), AgentManagerError> {
        self.agent_dao.upsert(agent).await?;
        Ok(())
    }

    /// Retrieves an agent by its unique identifier.
    pub async fn get_agent(&self, agent_id: &str) -> Result<Option<AgentDbRecord>, AgentManagerError> {
        let opt = self.agent_dao.get(agent_id).await?;
        Ok(opt)
    }

    /// Lists all agents belonging to a specific team.
    pub async fn list_by_team(&self, team_name: &str) -> Result<Vec<AgentDbRecord>, AgentManagerError> {
        let list = self.agent_dao.list_by_team(team_name).await?;
        Ok(list)
    }

    /// Updates agent lifecycle state (e.g. Draft -> Spawning -> Executing -> Completed).
    pub async fn update_state(&self, agent_id: &str, state: AgentLifecycleState) -> Result<(), AgentManagerError> {
        self.agent_dao.update_state(agent_id, &state.to_string()).await?;
        Ok(())
    }

    /// Archives (soft-deletes) an agent.
    pub async fn archive_agent(&self, agent_id: &str) -> Result<(), AgentManagerError> {
        self.agent_dao.update_state(agent_id, "Archived").await?;
        Ok(())
    }
}

