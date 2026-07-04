-- Trust score lives here as the identity-service's local cache of the
-- on-chain value (blockchain-service/policy-engine update it later; the
-- Solana program's own AgentAccount.trust_score is the source of truth once
-- that sync path is built). Starts at 0, matching the program's default on
-- registration.
ALTER TABLE agents ADD COLUMN trust_score SMALLINT NOT NULL DEFAULT 0;
