use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use shared::AppError;
use uuid::Uuid;

use crate::db::Execution;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub resource: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct PolicyDecision {
    decision: String,
    reason: String,
}

pub async fn execute_action(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<Execution>, AppError> {
    if req.description.trim().is_empty() {
        return Err(AppError::BadRequest("description must not be empty".into()));
    }

    let resp = state
        .http
        .post(format!("{}/evaluate", state.policy_engine_url))
        .header("x-internal-secret", &state.internal_secret)
        .json(&serde_json::json!({ "agent_id": agent_id, "resource": req.resource }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("policy-engine request failed: {e}")))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::NotFound(format!("agent {agent_id} not found")));
    }
    if resp.status() == reqwest::StatusCode::BAD_REQUEST {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::BadRequest(format!("policy-engine rejected request: {body}")));
    }
    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "policy-engine returned {}",
            resp.status()
        )));
    }

    let policy: PolicyDecision = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("bad policy-engine response: {e}")))?;

    let execution = crate::db::insert_execution(
        &state.pool,
        agent_id,
        &req.resource.to_lowercase(),
        &req.description,
        &policy.decision,
        &policy.reason,
    )
    .await?;

    tracing::info!(
        agent_id = %agent_id,
        resource = %execution.resource,
        decision = %execution.decision,
        "execution evaluated"
    );

    // Auditing a blocked attempt doesn't make sense (nothing was executed to
    // grade), and skipping it saves a real judge-panel API call for the one
    // decision that's already unambiguous. Fired in the background so this
    // endpoint doesn't wait on two LLM round trips before responding —
    // best-effort: if audit-engine is down, the execution still succeeded
    // and just stays unaudited, logged here rather than failing the request.
    if execution.decision != "deny" {
        let http = state.http.clone();
        let audit_engine_url = state.audit_engine_url.clone();
        let internal_secret = state.internal_secret.clone();
        let exec_for_audit = execution.clone();
        tokio::spawn(async move {
            let resp = http
                .post(format!("{audit_engine_url}/audit"))
                .header("x-internal-secret", &internal_secret)
                .json(&serde_json::json!({
                    "execution_id": exec_for_audit.id,
                    "agent_id": exec_for_audit.agent_id,
                    "resource": exec_for_audit.resource,
                    "description": exec_for_audit.description,
                    "decision": exec_for_audit.decision,
                    "reason": exec_for_audit.reason,
                }))
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    tracing::info!(execution_id = %exec_for_audit.id, "execution audited")
                }
                Ok(r) => tracing::warn!(
                    execution_id = %exec_for_audit.id,
                    status = %r.status(),
                    "audit-engine rejected audit request"
                ),
                Err(e) => tracing::warn!(
                    execution_id = %exec_for_audit.id,
                    error = %e,
                    "audit-engine unreachable, execution left unaudited"
                ),
            }
        });
    }

    Ok(Json(execution))
}

pub async fn list_executions(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<Execution>>, AppError> {
    let executions = crate::db::list_executions(&state.pool, agent_id).await?;
    Ok(Json(executions))
}
