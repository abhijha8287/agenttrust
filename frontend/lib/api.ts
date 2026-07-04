import { getSession } from './session';

const GATEWAY_URL =
  process.env.NEXT_PUBLIC_GATEWAY_URL ?? 'http://localhost:8080';

export const PERMISSION_RESOURCES = [
  'Filesystem',
  'Terminal',
  'Github',
  'Database',
  'Email',
  'Cloud',
  'Browser',
  'Wallet',
] as const;

export type PermissionResource = (typeof PERMISSION_RESOURCES)[number];
export type PermissionMode = 'Allow' | 'Deny' | 'Conditional';

export interface Agent {
  id: string;
  did: string;
  owner_id: string;
  wallet_address: string | null;
  version: string;
  capabilities: unknown;
  verification_status: string;
  public_key: number[];
  trust_score: number;
  created_at: string;
  updated_at: string;
}

export interface Permission {
  agent_id: string;
  resource: string;
  mode: string;
}

export interface AgentDetail extends Agent {
  permissions: Permission[];
}

export interface RegisterAgentRequest {
  owner_id: string;
  version: string;
  capabilities: unknown;
  public_key: number[];
  permissions: [PermissionResource, PermissionMode][];
}

export interface RegisterAgentResponse {
  id: string;
  did: string;
  verification_status: string;
}

async function authedFetch(path: string, init?: RequestInit): Promise<Response> {
  const { token } = await getSession();
  const res = await fetch(`${GATEWAY_URL}${path}`, {
    ...init,
    headers: {
      ...(init?.headers ?? {}),
      authorization: `Bearer ${token}`,
    },
  });
  return res;
}

export async function listAgents(): Promise<Agent[]> {
  const res = await authedFetch('/agents');
  if (!res.ok) throw new Error(`failed to list agents: ${res.status}`);
  return res.json();
}

export async function getAgent(id: string): Promise<AgentDetail> {
  const res = await authedFetch(`/agents/${id}`);
  if (!res.ok) throw new Error(`failed to fetch agent: ${res.status}`);
  return res.json();
}

export interface Execution {
  id: string;
  agent_id: string;
  resource: string;
  description: string;
  decision: 'allow' | 'deny' | 'conditional';
  reason: string;
  created_at: string;
}

export async function executeAction(
  agentId: string,
  resource: PermissionResource,
  description: string
): Promise<Execution> {
  const res = await authedFetch(`/agents/${agentId}/execute`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ resource, description }),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`failed to execute action (${res.status}): ${text}`);
  }
  return res.json();
}

export async function listExecutions(agentId: string): Promise<Execution[]> {
  const res = await authedFetch(`/agents/${agentId}/executions`);
  if (!res.ok) throw new Error(`failed to list executions: ${res.status}`);
  return res.json();
}

export interface Audit {
  id: string;
  execution_id: string;
  agent_id: string;
  verdict: 'pass' | 'fail' | 'low_confidence';
  score_delta: number;
  new_trust_score: number;
  audit_hash: string;
  judge_a_verdict: string;
  judge_a_reasoning: string;
  judge_b_verdict: string;
  judge_b_reasoning: string;
  created_at: string;
}

export async function listAudits(agentId: string): Promise<Audit[]> {
  const res = await authedFetch(`/agents/${agentId}/audits`);
  if (!res.ok) throw new Error(`failed to list audits: ${res.status}`);
  return res.json();
}

export async function registerAgent(
  req: RegisterAgentRequest
): Promise<RegisterAgentResponse> {
  const res = await authedFetch('/agents', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`failed to register agent (${res.status}): ${text}`);
  }
  return res.json();
}
