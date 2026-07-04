use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use shared::AppError;
use uuid::Uuid;

use crate::db::{ChainAgent, ChainAnchor};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AnchorRequest {
    pub audit_hash: String,
    pub trust_score: i16,
}

async fn ensure_registered_on_chain(
    state: &AppState,
    agent_id: Uuid,
) -> Result<ChainAgent, AppError> {
    if let Some(existing) = crate::db::get_chain_agent(&state.pool, agent_id).await? {
        return Ok(existing);
    }

    let agent_detail: serde_json::Value = state
        .http
        .get(format!("{}/agents/{}", state.identity_service_url, agent_id))
        .header("x-internal-secret", &state.internal_secret)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("identity-service request failed: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("bad identity-service response: {e}")))?;

    let did = agent_detail["did"]
        .as_str()
        .ok_or_else(|| AppError::Internal("no did in identity-service response".into()))?;
    let version = agent_detail["version"]
        .as_str()
        .ok_or_else(|| AppError::Internal("no version in identity-service response".into()))?;

    let args = serde_json::json!({
        "agentUuid": agent_id.to_string(),
        "did": did,
        "version": version,
    })
    .to_string();

    let result = crate::worker::run_chain_worker(&state.contracts_dir, "register", &args)
        .await
        .map_err(AppError::Internal)?;

    let agent_pda = result["agentPda"]
        .as_str()
        .ok_or_else(|| AppError::Internal(format!("no agentPda in register result: {result}")))?;
    let tx = result["txSignature"]
        .as_str()
        .ok_or_else(|| AppError::Internal(format!("no txSignature in register result: {result}")))?;

    tracing::info!(agent_id = %agent_id, agent_pda, tx, "agent registered on-chain");

    crate::db::insert_chain_agent(&state.pool, agent_id, agent_pda, tx)
        .await
        .map_err(AppError::from)
}

pub async fn anchor_execution(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<AnchorRequest>,
) -> Result<Json<ChainAnchor>, AppError> {
    let chain_agent = ensure_registered_on_chain(&state, agent_id).await?;

    let args = serde_json::json!({
        "agentPda": chain_agent.agent_pda,
        "auditHash": req.audit_hash,
        "trustScore": req.trust_score,
    })
    .to_string();

    let result = crate::worker::run_chain_worker(&state.contracts_dir, "anchor", &args)
        .await
        .map_err(AppError::Internal)?;

    let tx_signature = result["txSignature"]
        .as_str()
        .ok_or_else(|| AppError::Internal(format!("no txSignature in anchor result: {result}")))?;

    tracing::info!(agent_id = %agent_id, tx_signature, "audit hash anchored on-chain");

    let anchor = crate::db::insert_chain_anchor(
        &state.pool,
        agent_id,
        &req.audit_hash,
        req.trust_score,
        tx_signature,
    )
    .await?;

    Ok(Json(anchor))
}

pub async fn list_anchors(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<ChainAnchor>>, AppError> {
    let anchors = crate::db::list_chain_anchors(&state.pool, agent_id).await?;
    Ok(Json(anchors))
}
