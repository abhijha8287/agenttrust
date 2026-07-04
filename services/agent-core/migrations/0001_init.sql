-- agent-core owns this schema exclusively (same rule as identity-service:
-- eng review Finding 1, no shared DB across services). agent_id references
-- an agent in identity-service's database — deliberately not a foreign key,
-- since these are two separate databases; cross-service consistency is
-- identity-service's API, not a DB constraint.

CREATE TABLE executions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id    UUID NOT NULL,
    resource    TEXT NOT NULL,
    description TEXT NOT NULL,
    decision    TEXT NOT NULL CHECK (decision IN ('allow', 'deny', 'conditional')),
    reason      TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_executions_agent_id ON executions (agent_id);
