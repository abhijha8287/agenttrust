import Link from 'next/link';

export function Nav({ current }: { current?: 'dashboard' | 'register' }) {
  return (
    <nav
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: 'var(--sp-3) var(--sp-4)',
        borderBottom: '1px solid var(--border)',
        maxWidth: 1100,
        margin: '0 auto',
        position: 'sticky',
        top: 0,
        background: 'var(--bg)',
        zIndex: 20,
      }}
    >
      <Link
        href="/"
        style={{
          fontFamily: 'var(--font-display)',
          fontSize: 20,
          fontWeight: 600,
          color: 'var(--text)',
          textDecoration: 'none',
        }}
      >
        Agent<span style={{ color: 'var(--accent)' }}>Trust</span>
      </Link>
      <div
        style={{
          display: 'flex',
          gap: 'var(--sp-5)',
          fontSize: 14,
          color: 'var(--text-muted)',
        }}
      >
        <Link
          href="/dashboard"
          style={{ color: current === 'dashboard' ? 'var(--text)' : 'inherit' }}
        >
          Dashboard
        </Link>
        <Link
          href="/register"
          style={{ color: current === 'register' ? 'var(--text)' : 'inherit' }}
        >
          Register Agent
        </Link>
      </div>
    </nav>
  );
}
