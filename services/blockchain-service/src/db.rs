use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ChainAgent {
    pub identity_agent_id: Uuid,
    pub agent_pda: String,
    pub register_tx: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ChainAnchor {
    pub id: Uuid,
    pub identity_agent_id: Uuid,
    pub audit_hash: String,
    pub trust_score: i16,
    pub tx_signature: String,
    pub created_at: DateTime<Utc>,
}

pub async fn get_chain_agent(
    pool: &PgPool,
    identity_agent_id: Uuid,
) -> Result<Option<ChainAgent>, sqlx::Error> {
    sqlx::query_as::<_, ChainAgent>(
        r#"SELECT identity_agent_id, agent_pda, register_tx, created_at
           FROM chain_agents WHERE identity_agent_id = $1"#,
    )
    .bind(identity_agent_id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_chain_agent(
    pool: &PgPool,
    identity_agent_id: Uuid,
    agent_pda: &str,
    register_tx: &str,
) -> Result<ChainAgent, sqlx::Error> {
    sqlx::query_as::<_, ChainAgent>(
        r#"
        INSERT INTO chain_agents (identity_agent_id, agent_pda, register_tx)
        VALUES ($1, $2, $3)
        RETURNING identity_agent_id, agent_pda, register_tx, created_at
        "#,
    )
    .bind(identity_agent_id)
    .bind(agent_pda)
    .bind(register_tx)
    .fetch_one(pool)
    .await
}

pub async fn insert_chain_anchor(
    pool: &PgPool,
    identity_agent_id: Uuid,
    audit_hash: &str,
    trust_score: i16,
    tx_signature: &str,
) -> Result<ChainAnchor, sqlx::Error> {
    sqlx::query_as::<_, ChainAnchor>(
        r#"
        INSERT INTO chain_anchors (identity_agent_id, audit_hash, trust_score, tx_signature)
        VALUES ($1, $2, $3, $4)
        RETURNING id, identity_agent_id, audit_hash, trust_score, tx_signature, created_at
        "#,
    )
    .bind(identity_agent_id)
    .bind(audit_hash)
    .bind(trust_score)
    .bind(tx_signature)
    .fetch_one(pool)
    .await
}

pub async fn list_chain_anchors(
    pool: &PgPool,
    identity_agent_id: Uuid,
) -> Result<Vec<ChainAnchor>, sqlx::Error> {
    sqlx::query_as::<_, ChainAnchor>(
        r#"
        SELECT id, identity_agent_id, audit_hash, trust_score, tx_signature, created_at
        FROM chain_anchors WHERE identity_agent_id = $1 ORDER BY created_at DESC LIMIT 50
        "#,
    )
    .bind(identity_agent_id)
    .fetch_all(pool)
    .await
}
