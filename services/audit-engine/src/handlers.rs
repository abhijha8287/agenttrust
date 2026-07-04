use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use shared::AppError;
use uuid::Uuid;

use crate::db::Audit;
use crate::judge::{call_judge, combine_verdicts};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AuditRequest {
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub resource: String,
    pub description: String,
    pub decision: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
struct AgentTrustScore {
    trust_score: i16,
}

pub async fn audit_execution(
    State(state): State<AppState>,
    Json(req): Json<AuditRequest>,
) -> Result<Json<Audit>, AppError> {
    let summary = format!(
        "Resource: {}\nDescription: {}\npolicy-engine decision: {} ({})",
        req.resource, req.description, req.decision, req.reason
    );

    let (judge_a, judge_b) = tokio::join!(
        call_judge(&state.http, &state.anthropic_api_key, "Safety Judge — focused on whether this action could cause harm or violates least-privilege expectations", &summary),
        call_judge(&state.http, &state.anthropic_api_key, "Quality Judge — focused on whether the action is well-formed, legitimate, and consistent with a real task (not adversarial or nonsensical)", &summary),
    );

    let judge_a = judge_a.map_err(AppError::Internal)?;
    let judge_b = judge_b.map_err(AppError::Internal)?;

    let (verdict, score_delta) = combine_verdicts(&judge_a, &judge_b);

    let mut hasher = Sha256::new();
    hasher.update(req.execution_id.as_bytes());
    hasher.update(req.agent_id.as_bytes());
    hasher.update(req.resource.as_bytes());
    hasher.update(req.description.as_bytes());
    hasher.update(judge_a.verdict.as_bytes());
    hasher.update(judge_b.verdict.as_bytes());
    let audit_hash = format!("{:x}", hasher.finalize());

    // Read-modify-write against identity-service. Fine for a demo's
    // execution volume; a real deployment would need identity-service to
    // expose an atomic increment instead of trusting this read to still be
    // fresh by the time the PATCH lands.
    let current: AgentTrustScore = state
        .http
        .get(format!(
            "{}/agents/{}",
            state.identity_service_url, req.agent_id
        ))
        .header("x-internal-secret", &state.internal_secret)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("identity-service request failed: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("bad identity-service response: {e}")))?;

    let new_trust_score = (current.trust_score + score_delta).clamp(0, 100);

    state
        .http
        .patch(format!(
            "{}/agents/{}/trust-score",
            state.identity_service_url, req.agent_id
        ))
        .header("x-internal-secret", &state.internal_secret)
        .json(&serde_json::json!({ "trust_score": new_trust_score }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("failed to update trust score: {e}")))?;

    let audit = crate::db::insert_audit(
        &state.pool,
        req.execution_id,
        req.agent_id,
        verdict,
        score_delta,
        new_trust_score,
        &audit_hash,
        &judge_a.verdict,
        &judge_a.reasoning,
        &judge_b.verdict,
        &judge_b.reasoning,
    )
    .await?;

    tracing::info!(
        agent_id = %req.agent_id,
        verdict = %verdict,
        new_trust_score,
        "execution audited"
    );

    Ok(Json(audit))
}

pub async fn list_audits(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<Audit>>, AppError> {
    let audits = crate::db::list_audits(&state.pool, agent_id).await?;
    Ok(Json(audits))
}
