use shared::types::{Agent, Permission, RegisterAgentRequest};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_agent(
    pool: &PgPool,
    id: Uuid,
    did: &str,
    req: &RegisterAgentRequest,
) -> Result<Agent, sqlx::Error> {
    sqlx::query_as::<_, Agent>(
        r#"
        INSERT INTO agents (id, did, owner_id, version, capabilities, public_key)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, did, owner_id, wallet_address, version, capabilities,
                  verification_status, public_key, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(did)
    .bind(req.owner_id)
    .bind(&req.version)
    .bind(&req.capabilities)
    .bind(&req.public_key)
    .fetch_one(pool)
    .await
}

pub async fn insert_permissions(
    pool: &PgPool,
    agent_id: Uuid,
    permissions: &[(shared::types::PermissionResource, shared::types::PermissionMode)],
) -> Result<(), sqlx::Error> {
    for (resource, mode) in permissions {
        let mode_str = match mode {
            shared::types::PermissionMode::Allow => "allow",
            shared::types::PermissionMode::Deny => "deny",
            shared::types::PermissionMode::Conditional => "conditional",
        };
        sqlx::query(
            r#"
            INSERT INTO permissions (agent_id, resource, mode)
            VALUES ($1, $2, $3)
            ON CONFLICT (agent_id, resource) DO UPDATE SET mode = EXCLUDED.mode
            "#,
        )
        .bind(agent_id)
        .bind(resource.as_str())
        .bind(mode_str)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_agent(pool: &PgPool, id: Uuid) -> Result<Option<Agent>, sqlx::Error> {
    sqlx::query_as::<_, Agent>(
        r#"
        SELECT id, did, owner_id, wallet_address, version, capabilities,
               verification_status, public_key, created_at, updated_at
        FROM agents WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_permissions(pool: &PgPool, agent_id: Uuid) -> Result<Vec<Permission>, sqlx::Error> {
    sqlx::query_as::<_, Permission>(
        r#"SELECT agent_id, resource, mode FROM permissions WHERE agent_id = $1"#,
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await
}
