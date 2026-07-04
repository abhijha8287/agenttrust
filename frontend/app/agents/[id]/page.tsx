'use client';

import { useParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { Nav } from '@/components/Nav';
import { TrustGauge } from '@/components/TrustGauge';
import {
  executeAction,
  getAgent,
  listAudits,
  listExecutions,
  PERMISSION_RESOURCES,
  type Audit,
  type AgentDetail,
  type Execution,
  type PermissionResource,
} from '@/lib/api';

const PERM_COLOR: Record<string, string> = {
  allow: 'var(--success)',
  deny: 'var(--danger)',
  conditional: 'var(--warning)',
  pass: 'var(--success)',
  fail: 'var(--danger)',
  low_confidence: 'var(--warning)',
};
const PERM_BG: Record<string, string> = {
  allow: 'var(--success-dim)',
  deny: 'var(--danger-dim)',
  conditional: 'var(--warning-dim)',
  pass: 'var(--success-dim)',
  fail: 'var(--danger-dim)',
  low_confidence: 'var(--warning-dim)',
};

export default function AgentProfilePage() {
  const params = useParams<{ id: string }>();
  const [agent, setAgent] = useState<AgentDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [executions, setExecutions] = useState<Execution[]>([]);
  const [audits, setAudits] = useState<Audit[]>([]);
  const [resource, setResource] = useState<PermissionResource>('Filesystem');
  const [description, setDescription] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [execError, setExecError] = useState<string | null>(null);

  useEffect(() => {
    function refresh() {
      getAgent(params.id)
        .then(setAgent)
        .catch((e) => setError(e.message));
      listExecutions(params.id)
        .then(setExecutions)
        .catch(() => {});
      listAudits(params.id)
        .then(setAudits)
        .catch(() => {});
    }
    // Audits run in the background (two real judge-model calls take a few
    // seconds), so this polls rather than fetching once — the trust gauge
    // and audit timeline below update on their own once a verdict lands.
    refresh();
    const interval = setInterval(refresh, 4000);
    return () => clearInterval(interval);
  }, [params.id]);

  async function handleExecute(e: React.FormEvent) {
    e.preventDefault();
    setExecError(null);
    if (!description.trim()) {
      setExecError('Describe what the agent is attempting to do.');
      return;
    }
    setSubmitting(true);
    try {
      const result = await executeAction(params.id, resource, description);
      setExecutions((prev) => [result, ...prev]);
      setDescription('');
    } catch (err) {
      setExecError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <>
      <Nav />
      <div className="wrap">
        {error && (
          <div className="card" style={{ borderColor: 'var(--danger)', color: 'var(--danger)' }}>
            Failed to load agent: {error}
          </div>
        )}

        {!error && !agent && <div style={{ color: 'var(--text-muted)' }}>Loading...</div>}

        {agent && (
          <>
            <header
              className="card"
              style={{
                display: 'grid',
                gridTemplateColumns: 'auto 1fr',
                gap: 'var(--sp-5)',
                alignItems: 'center',
                borderRadius: 'var(--radius-lg)',
                marginBottom: 'var(--sp-4)',
              }}
            >
              <TrustGauge score={agent.trust_score} size={140} />
              <div>
                <h1 style={{ fontSize: 30, marginBottom: 'var(--sp-1)' }}>
                  Agent {agent.did.split(':').pop()?.slice(0, 8)}
                </h1>
                <div
                  style={{
                    fontFamily: 'var(--font-mono)',
                    fontSize: 13,
                    color: 'var(--text-faint)',
                    marginBottom: 'var(--sp-2)',
                  }}
                >
                  {agent.did}
                </div>
                <div style={{ display: 'flex', gap: 'var(--sp-2)', flexWrap: 'wrap', marginBottom: 'var(--sp-2)' }}>
                  <span className="badge badge-gold">
                    {agent.verification_status === 'verified' ? '✓ Verified' : agent.verification_status}
                  </span>
                  <span className="badge">v{agent.version}</span>
                </div>
                <div style={{ fontSize: 13, color: 'var(--text-muted)' }}>
                  Owner <strong style={{ color: 'var(--text)' }}>{agent.owner_id}</strong> · Registered{' '}
                  {new Date(agent.created_at).toLocaleDateString()}
                </div>
              </div>
            </header>

            <section style={{ marginBottom: 'var(--sp-6)' }}>
              <div className="section-label">Permissions</div>
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))',
                  gap: 'var(--sp-2)',
                }}
              >
                {agent.permissions.map((p) => (
                  <div
                    key={p.resource}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      background: 'var(--surface)',
                      border: '1px solid var(--border)',
                      borderRadius: 'var(--radius-md)',
                      padding: '8px 16px',
                    }}
                  >
                    <span style={{ fontSize: 13, textTransform: 'capitalize' }}>{p.resource}</span>
                    <span
                      style={{
                        fontSize: 11,
                        fontFamily: 'var(--font-mono)',
                        padding: '2px 8px',
                        borderRadius: 'var(--radius-sm)',
                        color: PERM_COLOR[p.mode] ?? 'var(--text-muted)',
                        background: PERM_BG[p.mode] ?? 'var(--surface-raised)',
                      }}
                    >
                      {p.mode}
                    </span>
                  </div>
                ))}
              </div>
            </section>

            <section style={{ marginBottom: 'var(--sp-6)' }}>
              <div className="section-label">Attempt an Action</div>
              <form
                onSubmit={handleExecute}
                className="card"
                style={{ display: 'flex', gap: 'var(--sp-3)', alignItems: 'flex-end', flexWrap: 'wrap' }}
              >
                <div style={{ flex: '0 0 160px' }}>
                  <label htmlFor="exec-resource">Resource</label>
                  <select
                    id="exec-resource"
                    value={resource}
                    onChange={(e) => setResource(e.target.value as PermissionResource)}
                    style={{ width: '100%' }}
                  >
                    {PERMISSION_RESOURCES.map((r) => (
                      <option key={r} value={r}>
                        {r}
                      </option>
                    ))}
                  </select>
                </div>
                <div style={{ flex: '1 1 260px' }}>
                  <label htmlFor="exec-desc">What is the agent trying to do?</label>
                  <input
                    id="exec-desc"
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    placeholder="e.g. read config.json from the filesystem"
                    style={{ width: '100%' }}
                  />
                </div>
                <button type="submit" className="btn btn-primary" disabled={submitting}>
                  {submitting ? 'Evaluating...' : 'Run'}
                </button>
              </form>
              {execError && (
                <div style={{ color: 'var(--danger)', fontSize: 13, marginTop: 'var(--sp-2)' }}>
                  {execError}
                </div>
              )}
              <p style={{ fontSize: 12, color: 'var(--text-faint)', marginTop: 'var(--sp-2)' }}>
                This goes through the real path: gateway → agent-core →
                policy-engine (which reads this agent&apos;s permissions above)
                → decision recorded below.
              </p>
            </section>

            <section style={{ marginBottom: 'var(--sp-6)' }}>
              <div className="section-label">Recent Executions</div>
              {executions.length === 0 ? (
                <div className="card" style={{ color: 'var(--text-muted)', fontSize: 14 }}>
                  No executions yet — try running an action above.
                </div>
              ) : (
                <div className="card" style={{ padding: 'var(--sp-2)' }}>
                  {executions.map((ex) => (
                    <div
                      key={ex.id}
                      style={{
                        display: 'grid',
                        gridTemplateColumns: '100px 1fr auto auto',
                        gap: 'var(--sp-3)',
                        alignItems: 'center',
                        padding: 'var(--sp-2) var(--sp-3)',
                        borderBottom: '1px solid var(--border)',
                        fontSize: 13,
                      }}
                    >
                      <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--accent)', textTransform: 'capitalize' }}>
                        {ex.resource}
                      </span>
                      <span style={{ color: 'var(--text-muted)' }} title={ex.reason}>
                        {ex.description}
                      </span>
                      <span
                        style={{
                          fontSize: 11,
                          padding: '2px 8px',
                          borderRadius: 999,
                          color: PERM_COLOR[ex.decision] ?? 'var(--text-muted)',
                          background: PERM_BG[ex.decision] ?? 'var(--surface-raised)',
                        }}
                      >
                        {ex.decision}
                      </span>
                      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--text-faint)' }}>
                        {new Date(ex.created_at).toLocaleTimeString()}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </section>

            <section style={{ marginBottom: 'var(--sp-6)' }}>
              <div className="section-label">Audit Timeline</div>
              {audits.length === 0 ? (
                <div className="card" style={{ color: 'var(--text-muted)', fontSize: 14 }}>
                  No audits yet. Allow/conditional executions above are sent
                  to a real 2-judge panel (Anthropic Claude) in the
                  background — this fills in within a few seconds of running
                  one.
                </div>
              ) : (
                <div
                  style={{
                    borderLeft: '2px solid var(--border)',
                    paddingLeft: 'var(--sp-4)',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 'var(--sp-4)',
                  }}
                >
                  {audits.map((a) => (
                    <div key={a.id} style={{ position: 'relative' }}>
                      <div
                        style={{
                          display: 'flex',
                          justifyContent: 'space-between',
                          alignItems: 'baseline',
                          marginBottom: 4,
                        }}
                      >
                        <span style={{ fontSize: 14, fontWeight: 500 }}>
                          Verdict:{' '}
                          <span
                            style={{
                              color: PERM_COLOR[a.verdict] ?? 'var(--text-muted)',
                              textTransform: 'capitalize',
                            }}
                          >
                            {a.verdict.replace('_', ' ')}
                          </span>{' '}
                          <span style={{ color: 'var(--text-faint)', fontWeight: 400 }}>
                            ({a.score_delta >= 0 ? '+' : ''}
                            {a.score_delta} → trust score {a.new_trust_score})
                          </span>
                        </span>
                        <span
                          style={{
                            fontFamily: 'var(--font-mono)',
                            fontSize: 12,
                            color: 'var(--text-faint)',
                          }}
                        >
                          {new Date(a.created_at).toLocaleTimeString()}
                        </span>
                      </div>
                      <div style={{ fontSize: 13, color: 'var(--text-muted)', marginBottom: 4 }}>
                        <strong style={{ color: 'var(--text)' }}>Safety Judge:</strong>{' '}
                        {a.judge_a_reasoning}
                      </div>
                      <div style={{ fontSize: 13, color: 'var(--text-muted)', marginBottom: 4 }}>
                        <strong style={{ color: 'var(--text)' }}>Quality Judge:</strong>{' '}
                        {a.judge_b_reasoning}
                      </div>
                      <div
                        style={{
                          fontFamily: 'var(--font-mono)',
                          fontSize: 12,
                          color: 'var(--text-faint)',
                        }}
                      >
                        hash: {a.audit_hash.slice(0, 16)}...
                      </div>
                    </div>
                  ))}
                </div>
              )}
              <p style={{ fontSize: 12, color: 'var(--text-faint)', marginTop: 'var(--sp-3)' }}>
                On-chain anchoring of this hash (blockchain-service) isn&apos;t
                wired into this frontend yet — the hash above is real and
                computed, just not yet written to Solana from here.
              </p>
            </section>
          </>
        )}
      </div>
    </>
  );
}
