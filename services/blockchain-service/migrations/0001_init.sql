-- blockchain-service owns this schema exclusively (same "no shared DB" rule
-- as the other services). identity_agent_id references a row in
-- identity-service's own database — a plain UUID, not a foreign key.

CREATE TABLE chain_agents (
    identity_agent_id UUID PRIMARY KEY,
    agent_pda         TEXT NOT NULL,
    register_tx       TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE chain_anchors (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_agent_id UUID NOT NULL,
    audit_hash        TEXT NOT NULL,
    trust_score       SMALLINT NOT NULL,
    tx_signature      TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_chain_anchors_agent_id ON chain_anchors (identity_agent_id);
