-- audit-engine owns this schema exclusively (same "no shared DB" rule as
-- identity-service and agent-core). execution_id/agent_id reference rows in
-- agent-core's and identity-service's own databases respectively — plain
-- UUIDs, not foreign keys, since those live in separate databases.

CREATE TABLE audits (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id      UUID NOT NULL,
    agent_id          UUID NOT NULL,
    verdict           TEXT NOT NULL CHECK (verdict IN ('pass', 'fail', 'low_confidence')),
    score_delta       SMALLINT NOT NULL,
    new_trust_score   SMALLINT NOT NULL,
    audit_hash        TEXT NOT NULL,
    judge_a_verdict   TEXT NOT NULL,
    judge_a_reasoning TEXT NOT NULL,
    judge_b_verdict   TEXT NOT NULL,
    judge_b_reasoning TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audits_agent_id ON audits (agent_id);
