use axum::{
    extract::{Path, State},
    Json,
};
use shared::{
    types::{Agent, Permission, RegisterAgentRequest, RegisterAgentResponse},
    AppError,
};
use uuid::Uuid;

use crate::AppState;

pub async fn list_agents(State(state): State<AppState>) -> Result<Json<Vec<Agent>>, AppError> {
    let agents = crate::db::list_agents(&state.pool).await?;
    Ok(Json(agents))
}

pub async fn register_agent(
    State(state): State<AppState>,
    Json(req): Json<RegisterAgentRequest>,
) -> Result<Json<RegisterAgentResponse>, AppError> {
    if req.version.trim().is_empty() {
        return Err(AppError::BadRequest("version must not be empty".into()));
    }
    if req.public_key.is_empty() {
        return Err(AppError::BadRequest("public_key must not be empty".into()));
    }

    let id = Uuid::new_v4();
    let did = Agent::new_did(&id);

    let agent = crate::db::insert_agent(&state.pool, id, &did, &req).await?;
    crate::db::insert_permissions(&state.pool, agent.id, &req.permissions).await?;

    tracing::info!(agent_id = %agent.id, did = %agent.did, "agent registered");

    Ok(Json(RegisterAgentResponse {
        id: agent.id,
        did: agent.did,
        verification_status: agent.verification_status,
    }))
}

#[derive(serde::Serialize)]
pub struct AgentDetail {
    #[serde(flatten)]
    pub agent: Agent,
    pub permissions: Vec<Permission>,
}

#[derive(serde::Deserialize)]
pub struct UpdateTrustScoreRequest {
    pub trust_score: i16,
}

pub async fn update_trust_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTrustScoreRequest>,
) -> Result<Json<Agent>, AppError> {
    if !(0..=100).contains(&req.trust_score) {
        return Err(AppError::BadRequest(
            "trust_score must be between 0 and 100".into(),
        ));
    }

    let agent = crate::db::update_trust_score(&state.pool, id, req.trust_score)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("agent {id} not found")))?;

    Ok(Json(agent))
}

pub async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentDetail>, AppError> {
    let agent = crate::db::get_agent(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("agent {id} not found")))?;
    let permissions = crate::db::get_permissions(&state.pool, id).await?;

    Ok(Json(AgentDetail { agent, permissions }))
}
