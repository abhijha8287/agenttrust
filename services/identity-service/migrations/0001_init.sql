-- identity-service owns this schema exclusively (eng review Finding 1,
-- 2026-07-04): no other service reads or writes these tables directly.
-- Cross-service reads go through this service's API or the Redis Stream
-- events it emits.

CREATE TABLE agents (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    did                 TEXT NOT NULL UNIQUE,
    owner_id            UUID NOT NULL,
    wallet_address      TEXT,
    version             TEXT NOT NULL,
    capabilities        JSONB NOT NULL DEFAULT '{}'::jsonb,
    verification_status TEXT NOT NULL DEFAULT 'unverified',
    public_key          BYTEA NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_agents_owner_id ON agents (owner_id);

CREATE TABLE permissions (
    agent_id UUID NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
    resource TEXT NOT NULL,
    mode     TEXT NOT NULL CHECK (mode IN ('allow', 'deny', 'conditional')),
    PRIMARY KEY (agent_id, resource)
);
