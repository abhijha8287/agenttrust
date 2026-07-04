//! AgentTrust on-chain program.
//!
//! Stores the minimum needed to make an agent's trust score and audit
//! history independently verifiable: identity (owner + authority pubkeys,
//! DID, version), the current trust score, verification status, and the
//! hash of the most recent audit report. Full audit reports live in
//! Postgres + IPFS (see audit-engine) — the chain only anchors the hash,
//! per the eng review's "never store large logs on-chain" constraint.
//!
//! `authority` is intentionally just a Pubkey the program checks a signer
//! against. Whether that pubkey belongs to a single keypair or a Squads
//! multisig vault PDA is entirely an off-chain/deploy-time decision — this
//! program doesn't know or care which. That's what keeps the multisig
//! upgrade (eng review, 2026-07-04) a configuration change, not a program
//! rewrite.

use anchor_lang::prelude::*;

declare_id!("DQZdU6jeY2SF1bYXNv9NuEW9JK26ZEaRfNjWG4MoqcSX");

pub const DID_MAX_LEN: usize = 64;
pub const VERSION_MAX_LEN: usize = 16;
pub const MAX_TRUST_SCORE: u16 = 10_000; // scaled 0-10000, matches the design doc's u16 field

#[program]
pub mod agent_trust {
    use super::*;

    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        _agent_id: [u8; 16],
        did: String,
        version: String,
        authority: Pubkey,
    ) -> Result<()> {
        require!(did.len() <= DID_MAX_LEN, AgentTrustError::DidTooLong);
        require!(
            version.len() <= VERSION_MAX_LEN,
            AgentTrustError::VersionTooLong
        );

        let agent = &mut ctx.accounts.agent;
        let now = Clock::get()?.unix_timestamp;

        agent.owner = ctx.accounts.owner.key();
        agent.authority = authority;
        agent.did = did;
        agent.version = version;
        agent.trust_score = 0;
        agent.verification_status = VerificationStatus::Unverified as u8;
        agent.audit_hash = [0u8; 32];
        agent.bump = ctx.bumps.agent;
        agent.created_at = now;
        agent.updated_at = now;

        emit!(AgentRegistered {
            agent: agent.key(),
            owner: agent.owner,
            authority: agent.authority,
            did: agent.did.clone(),
        });

        Ok(())
    }

    pub fn update_trust_score(ctx: Context<AuthorityAction>, new_score: u16) -> Result<()> {
        require!(
            new_score <= MAX_TRUST_SCORE,
            AgentTrustError::InvalidScore
        );

        let agent = &mut ctx.accounts.agent;
        agent.trust_score = new_score;
        agent.updated_at = Clock::get()?.unix_timestamp;

        emit!(TrustScoreUpdated {
            agent: agent.key(),
            new_score,
        });

        Ok(())
    }

    pub fn record_audit_hash(ctx: Context<AuthorityAction>, hash: [u8; 32]) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
        agent.audit_hash = hash;
        agent.updated_at = Clock::get()?.unix_timestamp;

        emit!(AuditHashRecorded {
            agent: agent.key(),
            hash,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(agent_id: [u8; 16])]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = owner,
        space = AgentAccount::LEN,
        seeds = [b"agent", owner.key().as_ref(), &agent_id],
        bump
    )]
    pub agent: Account<'info, AgentAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AuthorityAction<'info> {
    #[account(
        mut,
        constraint = agent.authority == authority.key() @ AgentTrustError::Unauthorized
    )]
    pub agent: Account<'info, AgentAccount>,

    pub authority: Signer<'info>,
}

#[account]
pub struct AgentAccount {
    pub owner: Pubkey,
    pub authority: Pubkey,
    pub did: String,
    pub version: String,
    pub trust_score: u16,
    pub verification_status: u8,
    pub audit_hash: [u8; 32],
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AgentAccount {
    pub const LEN: usize = 8 // discriminator
        + 32 // owner
        + 32 // authority
        + 4 + DID_MAX_LEN // did (String = 4-byte len prefix + content)
        + 4 + VERSION_MAX_LEN // version
        + 2 // trust_score
        + 1 // verification_status
        + 32 // audit_hash
        + 1 // bump
        + 8 // created_at
        + 8; // updated_at
}

#[repr(u8)]
pub enum VerificationStatus {
    Unverified = 0,
    Pending = 1,
    Verified = 2,
}

#[event]
pub struct AgentRegistered {
    pub agent: Pubkey,
    pub owner: Pubkey,
    pub authority: Pubkey,
    pub did: String,
}

#[event]
pub struct TrustScoreUpdated {
    pub agent: Pubkey,
    pub new_score: u16,
}

#[event]
pub struct AuditHashRecorded {
    pub agent: Pubkey,
    pub hash: [u8; 32],
}

#[error_code]
pub enum AgentTrustError {
    #[msg("caller is not this agent's registered authority")]
    Unauthorized,
    #[msg("trust score must be between 0 and 10000")]
    InvalidScore,
    #[msg("did exceeds maximum length of 64 bytes")]
    DidTooLong,
    #[msg("version exceeds maximum length of 16 bytes")]
    VersionTooLong,
}
