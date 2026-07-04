use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Execution {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub resource: String,
    pub description: String,
    pub decision: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

pub async fn insert_execution(
    pool: &PgPool,
    agent_id: Uuid,
    resource: &str,
    description: &str,
    decision: &str,
    reason: &str,
) -> Result<Execution, sqlx::Error> {
    sqlx::query_as::<_, Execution>(
        r#"
        INSERT INTO executions (agent_id, resource, description, decision, reason)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, agent_id, resource, description, decision, reason, created_at
        "#,
    )
    .bind(agent_id)
    .bind(resource)
    .bind(description)
    .bind(decision)
    .bind(reason)
    .fetch_one(pool)
    .await
}

pub async fn list_executions(pool: &PgPool, agent_id: Uuid) -> Result<Vec<Execution>, sqlx::Error> {
    sqlx::query_as::<_, Execution>(
        r#"
        SELECT id, agent_id, resource, description, decision, reason, created_at
        FROM executions WHERE agent_id = $1 ORDER BY created_at DESC LIMIT 50
        "#,
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await
}
