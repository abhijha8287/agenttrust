use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Audit {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub verdict: String,
    pub score_delta: i16,
    pub new_trust_score: i16,
    pub audit_hash: String,
    pub judge_a_verdict: String,
    pub judge_a_reasoning: String,
    pub judge_b_verdict: String,
    pub judge_b_reasoning: String,
    pub created_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_audit(
    pool: &PgPool,
    execution_id: Uuid,
    agent_id: Uuid,
    verdict: &str,
    score_delta: i16,
    new_trust_score: i16,
    audit_hash: &str,
    judge_a_verdict: &str,
    judge_a_reasoning: &str,
    judge_b_verdict: &str,
    judge_b_reasoning: &str,
) -> Result<Audit, sqlx::Error> {
    sqlx::query_as::<_, Audit>(
        r#"
        INSERT INTO audits (
            execution_id, agent_id, verdict, score_delta, new_trust_score, audit_hash,
            judge_a_verdict, judge_a_reasoning, judge_b_verdict, judge_b_reasoning
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, execution_id, agent_id, verdict, score_delta, new_trust_score,
                  audit_hash, judge_a_verdict, judge_a_reasoning, judge_b_verdict,
                  judge_b_reasoning, created_at
        "#,
    )
    .bind(execution_id)
    .bind(agent_id)
    .bind(verdict)
    .bind(score_delta)
    .bind(new_trust_score)
    .bind(audit_hash)
    .bind(judge_a_verdict)
    .bind(judge_a_reasoning)
    .bind(judge_b_verdict)
    .bind(judge_b_reasoning)
    .fetch_one(pool)
    .await
}

pub async fn list_audits(pool: &PgPool, agent_id: Uuid) -> Result<Vec<Audit>, sqlx::Error> {
    sqlx::query_as::<_, Audit>(
        r#"
        SELECT id, execution_id, agent_id, verdict, score_delta, new_trust_score,
               audit_hash, judge_a_verdict, judge_a_reasoning, judge_b_verdict,
               judge_b_reasoning, created_at
        FROM audits WHERE agent_id = $1 ORDER BY created_at DESC LIMIT 50
        "#,
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await
}
