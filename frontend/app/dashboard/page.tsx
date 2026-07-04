'use client';

import Link from 'next/link';
import { useEffect, useState } from 'react';
import { Nav } from '@/components/Nav';
import { TrustGauge } from '@/components/TrustGauge';
import { listAgents, type Agent } from '@/lib/api';

export default function DashboardPage() {
  const [agents, setAgents] = useState<Agent[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listAgents()
      .then(setAgents)
      .catch((e) => setError(e.message));
  }, []);

  return (
    <>
      <Nav current="dashboard" />
      <div className="wrap">
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'baseline',
            marginBottom: 'var(--sp-4)',
          }}
        >
          <h1 style={{ fontSize: 28 }}>Your Agents</h1>
          <Link href="/register" className="btn btn-primary">
            + Register Agent
          </Link>
        </div>

        {error && (
          <div className="card" style={{ borderColor: 'var(--danger)', color: 'var(--danger)' }}>
            Failed to load agents: {error}. Is the gateway running at{' '}
            {process.env.NEXT_PUBLIC_GATEWAY_URL ?? 'http://localhost:8080'}?
          </div>
        )}

        {!error && agents === null && (
          <div className="card" style={{ color: 'var(--text-muted)' }}>
            Loading...
          </div>
        )}

        {agents && agents.length === 0 && (
          <div className="card" style={{ textAlign: 'center', padding: 'var(--sp-6) var(--sp-4)' }}>
            <h3 style={{ fontSize: 17, marginBottom: 'var(--sp-2)', fontFamily: 'var(--font-display)' }}>
              No agents registered yet
            </h3>
            <p style={{ color: 'var(--text-muted)', fontSize: 14 }}>
              Register your first agent to see it here.
            </p>
            <Link href="/register" className="btn btn-primary" style={{ marginTop: 'var(--sp-3)' }}>
              Register an Agent
            </Link>
          </div>
        )}

        {agents && agents.length > 0 && (
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))',
              gap: 'var(--sp-3)',
            }}
          >
            {agents.map((agent) => (
              <Link
                key={agent.id}
                href={`/agents/${agent.id}`}
                className="card"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 'var(--sp-3)',
                  textDecoration: 'none',
                  color: 'inherit',
                }}
              >
                <TrustGauge score={agent.trust_score} size={44} />
                <div style={{ overflow: 'hidden' }}>
                  <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 2 }}>
                    {agent.did.split(':').pop()?.slice(0, 12)}...
                  </div>
                  <div
                    style={{
                      fontFamily: 'var(--font-mono)',
                      fontSize: 11,
                      color: 'var(--text-faint)',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    v{agent.version} · {agent.verification_status}
                  </div>
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
