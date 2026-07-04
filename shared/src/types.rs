use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum PermissionMode {
    Allow,
    Deny,
    Conditional,
}

/// The 8 permission resources named in the product spec. Filesystem/Terminal/
/// GitHub/Database/Email/Cloud/Browser/Wallet — each agent gets an explicit
/// mode per resource, never an implicit default.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionResource {
    Filesystem,
    Terminal,
    Github,
    Database,
    Email,
    Cloud,
    Browser,
    Wallet,
}

impl PermissionResource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionResource::Filesystem => "filesystem",
            PermissionResource::Terminal => "terminal",
            PermissionResource::Github => "github",
            PermissionResource::Database => "database",
            PermissionResource::Email => "email",
            PermissionResource::Cloud => "cloud",
            PermissionResource::Browser => "browser",
            PermissionResource::Wallet => "wallet",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub did: String,
    pub owner_id: Uuid,
    pub wallet_address: Option<String>,
    pub version: String,
    pub capabilities: serde_json::Value,
    pub verification_status: String,
    pub public_key: Vec<u8>,
    pub trust_score: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub agent_id: Uuid,
    pub resource: String,
    pub mode: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterAgentRequest {
    pub owner_id: Uuid,
    pub version: String,
    pub capabilities: serde_json::Value,
    pub public_key: Vec<u8>,
    /// Explicit permission grants. Any of the 8 resources not listed here
    /// defaults to Deny — permissions are opt-in, never opt-out.
    pub permissions: Vec<(PermissionResource, PermissionMode)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterAgentResponse {
    pub id: Uuid,
    pub did: String,
    pub verification_status: String,
}

fn did_for(id: &Uuid) -> String {
    format!("did:agenttrust:{}", id.simple())
}

impl Agent {
    pub fn new_did(id: &Uuid) -> String {
        did_for(id)
    }
}
