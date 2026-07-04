import Link from 'next/link';
import { Nav } from '@/components/Nav';
import { TrustGauge } from '@/components/TrustGauge';

export default function LandingPage() {
  return (
    <>
      <Nav />
      <div className="wrap">
        <section
          style={{
            padding: 'var(--sp-8) 0 var(--sp-7)',
            display: 'grid',
            gridTemplateColumns: '1.2fr 0.8fr',
            gap: 'var(--sp-6)',
            alignItems: 'center',
          }}
        >
          <div>
            <h1 style={{ fontSize: 52, lineHeight: 1.06, marginBottom: 'var(--sp-3)' }}>
              The trust layer for <em style={{ color: 'var(--accent)', fontStyle: 'normal' }}>autonomous AI agents</em>
            </h1>
            <p style={{ fontSize: 17, color: 'var(--text-muted)', maxWidth: 480, marginBottom: 'var(--sp-4)' }}>
              Cryptographic identity, reputation scoring, and on-chain audit
              trails — real, verifiable trust, not vibes.
            </p>
            <div style={{ display: 'flex', gap: 'var(--sp-3)' }}>
              <Link href="/register" className="btn btn-primary">
                Register an Agent
              </Link>
              <Link href="/dashboard" className="btn btn-ghost">
                View Dashboard
              </Link>
            </div>
          </div>

          <div className="card" style={{ borderRadius: 'var(--radius-xl)' }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                fontFamily: 'var(--font-mono)',
                fontSize: 11,
                color: 'var(--text-faint)',
                marginBottom: 'var(--sp-3)',
              }}
            >
              Illustrative example — not live data
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--sp-3)' }}>
              <TrustGauge score={87} size={72} />
              <div>
                <div style={{ fontSize: 14, fontWeight: 600 }}>CodeXecutor</div>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--text-faint)' }}>
                  did:agenttrust:9f2a7c...c81e
                </div>
              </div>
            </div>
          </div>
        </section>

        <section style={{ padding: 'var(--sp-7) 0' }}>
          <div className="section-label">How it works</div>
          <h2 style={{ fontSize: 28, marginBottom: 'var(--sp-4)', maxWidth: 560 }}>
            Every agent gets a verifiable identity, from registration to
            every action it takes.
          </h2>
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            {[
              {
                n: '01',
                t: 'Register',
                d: 'An agent is issued a DID and a keypair-backed identity, with explicit per-resource permissions set at registration — nothing is allowed by default.',
              },
              {
                n: '02',
                t: 'Execute',
                d: 'Every action the agent takes is evaluated against its permissions and logged.',
              },
              {
                n: '03',
                t: 'Anchor',
                d: 'Trust scores and audit hashes are anchored on Solana — verifiable independently of AgentTrust itself.',
              },
            ].map((s) => (
              <div
                key={s.n}
                style={{
                  display: 'grid',
                  gridTemplateColumns: '64px 1fr',
                  gap: 'var(--sp-4)',
                  padding: 'var(--sp-4) 0',
                  borderTop: '1px solid var(--border)',
                }}
              >
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 24, color: 'var(--accent)', fontWeight: 500 }}>
                  {s.n}
                </div>
                <div>
                  <div style={{ fontSize: 19, marginBottom: 6, fontFamily: 'var(--font-display)' }}>{s.t}</div>
                  <div style={{ fontSize: 14, color: 'var(--text-muted)', maxWidth: 520 }}>{s.d}</div>
                </div>
              </div>
            ))}
            <div style={{ borderBottom: '1px solid var(--border)' }} />
          </div>
        </section>

        <footer style={{ borderTop: '1px solid var(--border)', padding: 'var(--sp-5) 0' }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              fontSize: 13,
              color: 'var(--text-faint)',
            }}
          >
            <span>AgentTrust</span>
            <div style={{ display: 'flex', gap: 'var(--sp-4)' }}>
              <Link href="/dashboard" style={{ color: 'var(--text-faint)' }}>
                Dashboard
              </Link>
              <Link href="/register" style={{ color: 'var(--text-faint)' }}>
                Register
              </Link>
            </div>
          </div>
        </footer>
      </div>
    </>
  );
}
